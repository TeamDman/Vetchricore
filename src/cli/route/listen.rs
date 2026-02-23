use crate::cli::Cli;
use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::route::RouteArgs;
use crate::cli::route::RouteCommand;
use crate::cli::route::add::RouteAddArgs;
use crate::cli::veilid_runtime::start_api_for_profile;
use arbitrary::Arbitrary;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use figue as args;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;
use veilid_core::CRYPTO_KIND_VLD0;
use veilid_core::DHTSchema;
use veilid_core::RouteBlob;
use veilid_core::RouteId;
use veilid_core::VeilidUpdate;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct RouteListenArgs {
    #[facet(args::positional)]
    pub name: String,

    #[facet(args::named)]
    pub count: Option<usize>,
}

const ROUTE_ALLOCATE_MAX_ATTEMPTS: usize = 20;
const ROUTE_ALLOCATE_RETRY_DELAY: Duration = Duration::from_secs(1);

impl RouteListenArgs {
    /// # Errors
    ///
    /// Returns an error if the route is not found or listening fails.
    pub async fn invoke(self, context: &InvokeContext) -> Result<()> {
        if matches!(self.count, Some(0)) {
            bail!("--count must be greater than 0.");
        }
        listen_on_named_route(context, &self.name, self.count).await
    }
}

/// Listen for incoming messages using an existing named route identity.
///
/// # Errors
///
/// Returns an error if the profile/route identity cannot be loaded,
/// attachment readiness is not reached, or Veilid operations fail.
#[expect(
    clippy::too_many_lines,
    reason = "listen flow includes setup, callback wiring, and runtime loop"
)]
pub async fn listen_on_named_route(
    context: &InvokeContext,
    route_name: &str,
    message_count_limit: Option<usize>,
) -> Result<()> {
    let profile_home = context.profile_home();
    let identity = app_state::local_route_identity(profile_home, route_name)?.ok_or_else(|| {
        eyre::eyre!(
            "Route '{}' does not exist. Create it with '{}'.",
            route_name,
            Cli::display_invocation(&RouteArgs {
                command: RouteCommand::Add(RouteAddArgs {
                    name: route_name.to_owned(),
                    listen: true,
                }),
            })
        )
    })?;

    let public_internet_ready = Arc::new(AtomicBool::new(false));
    let known_user_map = Arc::new(Mutex::new(
        app_state::list_known_users(profile_home)?
            .into_iter()
            .map(|entry| (entry.pubkey.to_string(), entry.name))
            .collect::<std::collections::HashMap<_, _>>(),
    ));
    let dead_routes = Arc::new(Mutex::new(HashSet::<RouteId>::new()));
    let printed_messages = Arc::new(AtomicUsize::new(0));
    let callback = route_update_callback(
        Arc::clone(&public_internet_ready),
        Arc::clone(&known_user_map),
        Arc::clone(&dead_routes),
        Arc::clone(&printed_messages),
    );

    let api = start_api_for_profile(profile_home, true, callback).await?;
    wait_for_public_internet_ready(&api, &public_internet_ready).await?;

    let router = api.routing_context()?.with_default_safety()?;
    if router
        .open_dht_record(identity.record_key.clone(), Some(identity.keypair.clone()))
        .await
        .is_err()
    {
        let _ = router
            .create_dht_record(
                CRYPTO_KIND_VLD0,
                DHTSchema::dflt(1)?,
                Some(identity.keypair.clone()),
            )
            .await?;
    }

    let mut route_blob = allocate_private_route_with_retry(
        &api,
        ROUTE_ALLOCATE_MAX_ATTEMPTS,
        ROUTE_ALLOCATE_RETRY_DELAY,
    )
    .await?;
    router
        .set_dht_value(
            identity.record_key.clone(),
            0,
            route_blob.blob.clone(),
            None,
        )
        .await?;

    println!(
        "Created route information and stored it under record key {} for {}",
        identity.record_key,
        profile_home.profile()
    );
    println!("Listening for messages.");

    loop {
        if let Some(limit) = message_count_limit
            && printed_messages.load(Ordering::Acquire) >= limit
        {
            break;
        }

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                break;
            }
            () = tokio::time::sleep(Duration::from_millis(250)) => {
                let should_rotate = {
                    let mut guard = dead_routes
                        .lock()
                        .map_err(|_poison| eyre::eyre!("dead route state lock poisoned"))?;
                    guard.remove(&route_blob.route_id)
                };

                if should_rotate {
                    route_blob = allocate_private_route_with_retry(
                        &api,
                        ROUTE_ALLOCATE_MAX_ATTEMPTS,
                        ROUTE_ALLOCATE_RETRY_DELAY,
                    )
                    .await?;
                    router
                        .set_dht_value(
                            identity.record_key.clone(),
                            0,
                            route_blob.blob.clone(),
                            None,
                        )
                        .await?;
                    println!("Route changed; republished route information.");
                }
            }
        }
    }

    let _ = router
        .set_dht_value(identity.record_key.clone(), 0, Vec::new(), None)
        .await;
    println!("Stopped listening; route record marked offline (empty route data).");

    let _ = api.release_private_route(route_blob.route_id);
    let _ = router.close_dht_record(identity.record_key.clone()).await;
    api.shutdown().await;
    Ok(())
}

fn route_update_callback(
    public_internet_ready: Arc<AtomicBool>,
    known_user_map: Arc<Mutex<std::collections::HashMap<String, String>>>,
    dead_routes: Arc<Mutex<HashSet<RouteId>>>,
    printed_messages: Arc<AtomicUsize>,
) -> crate::cli::veilid_runtime::UpdateCallback {
    Arc::new(move |update: VeilidUpdate| match update {
        VeilidUpdate::Attachment(attachment) => {
            public_internet_ready.store(attachment.public_internet_ready, Ordering::Release);
        }
        VeilidUpdate::AppMessage(message) => {
            let text = String::from_utf8_lossy(message.message()).to_string();
            if let Some((pubkey, body)) = text.split_once('|') {
                if let Ok(guard) = known_user_map.lock() {
                    if let Some(name) = guard.get(pubkey) {
                        println!("{name}> {body}");
                    } else {
                        println!("{pubkey}> {body}");
                    }
                } else {
                    println!("{pubkey}> {body}");
                }
            } else {
                println!("INCOMING> {text}");
            }
            printed_messages.fetch_add(1, Ordering::AcqRel);
        }
        VeilidUpdate::RouteChange(change) => {
            if let Ok(mut guard) = dead_routes.lock() {
                for route_id in &change.dead_routes {
                    guard.insert(route_id.clone());
                }
            }
        }
        _ => {}
    })
}

pub async fn wait_for_public_internet_ready(
    api: &veilid_core::VeilidAPI,
    public_internet_ready: &AtomicBool,
) -> Result<()> {
    if !public_internet_ready.load(Ordering::Acquire) {
        let state = api.get_state().await?;
        if state.attachment.public_internet_ready {
            public_internet_ready.store(true, Ordering::Release);
        }
    }

    if !public_internet_ready.load(Ordering::Acquire) {
        println!("Waiting for public internet readiness...");
        let start = Instant::now();
        let timeout = Duration::from_secs(120);
        while !public_internet_ready.load(Ordering::Acquire) {
            if start.elapsed() >= timeout {
                bail!(
                    "Timed out waiting for public internet readiness; retry when network attachment improves."
                );
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    }
    Ok(())
}

async fn allocate_private_route_with_retry(
    api: &veilid_core::VeilidAPI,
    max_attempts: usize,
    retry_delay: Duration,
) -> Result<RouteBlob> {
    for attempt in 1..=max_attempts {
        match api.new_private_route().await {
            Ok(route_blob) => return Ok(route_blob),
            Err(error) => {
                if is_try_again_route_allocation_error(&error) && attempt < max_attempts {
                    println!(
                        "Route allocation attempt {} of {} failed transiently; retrying in {}s...",
                        attempt,
                        max_attempts,
                        retry_delay.as_secs()
                    );
                    tokio::time::sleep(retry_delay).await;
                    continue;
                }
                return Err(error.into());
            }
        }
    }

    bail!(
        "Unable to allocate private route after {} attempts.",
        max_attempts
    )
}

fn is_try_again_route_allocation_error(error: &veilid_core::VeilidAPIError) -> bool {
    let text = error.to_string();
    text.contains("TryAgain:")
        && (text.contains("allocated route failed to test")
            || text.contains("allocated route could not be tested"))
}

impl ToArgs for RouteListenArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        let mut args: Vec<std::ffi::OsString> = vec![self.name.clone().into()];
        if let Some(count) = self.count {
            args.push("--count".into());
            args.push(count.to_string().into());
        }
        args
    }
}
