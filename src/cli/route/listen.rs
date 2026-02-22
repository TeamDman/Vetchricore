use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::global_args::GlobalArgs;
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
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;
use veilid_core::CRYPTO_KIND_VLD0;
use veilid_core::DHTSchema;
use veilid_core::RouteId;
use veilid_core::VeilidUpdate;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct RouteListenArgs {
    #[facet(args::positional)]
    pub name: String,
}

impl RouteListenArgs {
    /// # Errors
    ///
    /// Returns an error if the route is not found or listening fails.
    pub async fn invoke(self, global: &GlobalArgs) -> Result<()> {
        listen_on_named_route(global, &self.name).await
    }
}

/// Listen for incoming messages using an existing named route identity.
///
/// # Errors
///
/// Returns an error if the profile/route identity cannot be loaded,
/// attachment readiness is not reached, or Veilid operations fail.
pub async fn listen_on_named_route(global: &GlobalArgs, route_name: &str) -> Result<()> {
    let profile = app_state::resolve_profile(global)?;
    let mut identity = app_state::local_route_identity(&profile, route_name)?.ok_or_else(|| {
        eyre::eyre!(
            "Route '{}' does not exist. Create it with 'vetchricore route create {} --listen'.",
            route_name,
            route_name
        )
    })?;

    let public_internet_ready = Arc::new(AtomicBool::new(false));
    let friend_map = Arc::new(Mutex::new(
        app_state::list_friends(&profile)?
            .into_iter()
            .map(|f| (f.pubkey.to_string(), f.name))
            .collect::<std::collections::HashMap<_, _>>(),
    ));
    let dead_routes = Arc::new(Mutex::new(HashSet::<RouteId>::new()));
    let callback = route_update_callback(
        Arc::clone(&public_internet_ready),
        Arc::clone(&friend_map),
        Arc::clone(&dead_routes),
    );

    let api = start_api_for_profile(&profile, true, callback).await?;
    wait_for_public_internet_ready(&api, &public_internet_ready).await?;

    let router = api.routing_context()?.with_default_safety()?;
    if router
        .open_dht_record(identity.record_key.clone(), Some(identity.keypair.clone()))
        .await
        .is_err()
    {
        let descriptor = router
            .create_dht_record(
                CRYPTO_KIND_VLD0,
                DHTSchema::dflt(1)?,
                Some(identity.keypair.clone()),
            )
            .await?;
        let created_record_key = descriptor.key();
        if created_record_key != identity.record_key {
            app_state::remove_local_route_identity(&profile, &identity.name)?;
            app_state::add_local_route_identity(
                &profile,
                &identity.name,
                &identity.keypair,
                &created_record_key,
            )?;
            identity.record_key = created_record_key;
        }
    }

    let mut route_blob = api.new_private_route().await?;
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
        identity.record_key, profile
    );
    println!("Listening for messages.");

    loop {
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
                    route_blob = api.new_private_route().await?;
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

    let _ = api.release_private_route(route_blob.route_id);
    let _ = router.close_dht_record(identity.record_key.clone()).await;
    let _ = router.delete_dht_record(identity.record_key).await;
    api.shutdown().await;
    Ok(())
}

fn route_update_callback(
    public_internet_ready: Arc<AtomicBool>,
    friend_map: Arc<Mutex<std::collections::HashMap<String, String>>>,
    dead_routes: Arc<Mutex<HashSet<RouteId>>>,
) -> crate::cli::veilid_runtime::UpdateCallback {
    Arc::new(move |update: VeilidUpdate| match update {
        VeilidUpdate::Attachment(attachment) => {
            if attachment.public_internet_ready {
                public_internet_ready.store(true, Ordering::Release);
            }
        }
        VeilidUpdate::AppMessage(message) => {
            let text = String::from_utf8_lossy(message.message()).to_string();
            if let Some((pubkey, body)) = text.split_once('|') {
                if let Ok(guard) = friend_map.lock() {
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

async fn wait_for_public_internet_ready(
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

impl ToArgs for RouteListenArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        vec![self.name.clone().into()]
    }
}
