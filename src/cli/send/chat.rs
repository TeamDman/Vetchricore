use crate::cli::Cli;
use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::key::KeyArgs;
use crate::cli::key::KeyCommand;
use crate::cli::key::key_gen::KeyGenArgs;
use crate::cli::known_user::KnownUserArgs;
use crate::cli::known_user::KnownUserCommand;
use crate::cli::known_user::add::KnownUserAddArgs;
use crate::cli::veilid_runtime::start_api_for_profile;
use arbitrary::Arbitrary;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use figue as args;
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;
use veilid_core::RecordKey;
use veilid_core::RouteId;
use veilid_core::Target;
use veilid_core::VeilidUpdate;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct SendChatArgs {
    #[facet(args::named)]
    pub message: Option<String>,
    #[facet(args::named)]
    pub retry: Option<usize>,
}

impl SendChatArgs {
    #[expect(
        clippy::too_many_lines,
        reason = "chat flow combines validation, setup, and interactive send loop"
    )]
    pub async fn invoke(self, context: &InvokeContext, known_user: &str) -> Result<()> {

        let retry_attempts = self.retry.unwrap_or(1);
        if retry_attempts == 0 {
            bail!("--retry must be greater than 0.");
        }

        let profile_home = context.profile_home();
        let my_keypair = app_state::load_keypair(profile_home)?.ok_or_else(|| {
            eyre::eyre!(
                "You have no key. Run '{}' first.",
                Cli::display_invocation(&crate::cli::Command::Key(KeyArgs {
                    command: KeyCommand::Gen(KeyGenArgs),
                }))
            )
        })?;
        let known_user_key = app_state::known_user_public_key(profile_home, known_user)?
            .ok_or_else(|| {
                eyre::eyre!(
                    "Known user '{}' not found. Add them with '{}'.",
                    known_user,
                    Cli::display_invocation(&crate::cli::Command::KnownUser(KnownUserArgs {
                        command: KnownUserCommand::Add(KnownUserAddArgs {
                            name: "<name>".to_owned(),
                            pubkey: "<pubkey>".to_owned(),
                        }),
                    }))
                )
            })?;

        let keys = app_state::route_keys_for_known_user(profile_home, known_user)?;
        if keys.is_empty() {
            bail!("No route record keys configured for {}.", known_user);
        }

        let public_internet_ready = Arc::new(AtomicBool::new(false));
        let callback = {
            let public_internet_ready = Arc::clone(&public_internet_ready);
            Arc::new(move |update: VeilidUpdate| {
                if let VeilidUpdate::Attachment(attachment) = update {
                    public_internet_ready
                        .store(attachment.public_internet_ready, Ordering::Release);
                }
            }) as crate::cli::veilid_runtime::UpdateCallback
        };

        let api = start_api_for_profile(profile_home, true, callback).await?;
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
                    api.shutdown().await;
                    bail!(
                        "Timed out waiting for public internet readiness; retry when network attachment improves."
                    );
                }
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
        }

        let router = api.routing_context()?.with_default_safety()?;
        let mut cached_route_id: Option<RouteId> = None;

        if let Some(message) = self.message {
            let payload = format!("{}|{}", my_keypair.key(), message);
            send_payload_with_route_retry(
                &api,
                &router,
                &keys,
                payload.into_bytes(),
                retry_attempts,
                &mut cached_route_id,
            )
            .await?;
            println!("Message sent.");
        } else {
            loop {
                let input_task = tokio::task::spawn_blocking(move || -> std::io::Result<String> {
                    let mut out = std::io::stdout();
                    write!(out, "CHAT> ")?;
                    out.flush()?;
                    let mut line = String::new();
                    let _ = std::io::stdin().read_line(&mut line)?;
                    Ok(line)
                });

                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {
                        println!("Ctrl+C detected.");
                        break;
                    }
                    line = input_task => {
                        let line = line.wrap_err("failed reading chat input")??;
                        let text = line.trim_end_matches(['\r', '\n']).to_owned();
                        if text.is_empty() {
                            continue;
                        }
                        let payload = format!("{}|{}", my_keypair.key(), text);
                        send_payload_with_route_retry(
                            &api,
                            &router,
                            &keys,
                            payload.into_bytes(),
                            retry_attempts,
                            &mut cached_route_id,
                        )
                        .await?;
                    }
                }
            }
        }

        if let Some(route_id) = cached_route_id {
            let _ = api.release_private_route(route_id);
        }

        api.shutdown().await;
        let _ = known_user_key;
        Ok(())
    }
}

async fn acquire_route(
    api: &veilid_core::VeilidAPI,
    router: &veilid_core::RoutingContext,
    key: &RecordKey,
) -> Result<Option<RouteId>> {
    let _ = router.open_dht_record(key.clone(), None).await?;
    let value = router.get_dht_value(key.clone(), 0, true).await?;
    let _ = router.close_dht_record(key.clone()).await;

    let Some(value) = value else {
        return Ok(None);
    };

    let data = value.data();
    if data.is_empty() {
        return Ok(None);
    }

    let route_id = api.import_remote_private_route(data.to_vec())?;
    Ok(Some(route_id))
}

async fn acquire_best_route(
    api: &veilid_core::VeilidAPI,
    router: &veilid_core::RoutingContext,
    keys: &[RecordKey],
) -> Result<RouteId> {
    for (index, key) in keys.iter().enumerate() {
        println!("Trying route record key {} of {}.", index + 1, keys.len());
        if let Some(route_id) = acquire_route(api, router, key).await? {
            println!("Acquired route information.");
            return Ok(route_id);
        }
    }

    bail!("Unable to acquire a route from any configured record key.")
}

async fn send_payload_with_route_retry(
    api: &veilid_core::VeilidAPI,
    router: &veilid_core::RoutingContext,
    keys: &[RecordKey],
    payload: Vec<u8>,
    max_attempts: usize,
    cached_route_id: &mut Option<RouteId>,
) -> Result<()> {
    for attempt in 1..=max_attempts {
        println!("Send attempt {attempt} of {max_attempts}.");

        if cached_route_id.is_none() {
            let route_id = match acquire_best_route(api, router, keys).await {
                Ok(route_id) => route_id,
                Err(error) => {
                    if should_retry_route_acquire(&error) && attempt < max_attempts {
                        println!(
                            "Route record key unavailable; retrying in 1s (attempt {attempt} of {max_attempts})."
                        );
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        continue;
                    }
                    return Err(error);
                }
            };
            *cached_route_id = Some(route_id);
        }

        let Some(route_id) = cached_route_id.clone() else {
            continue;
        };

        match router
            .app_message(Target::RouteId(route_id.clone()), payload.clone())
            .await
        {
            Ok(()) => return Ok(()),
            Err(error) => {
                let error = eyre::Report::from(error);
                if should_reacquire_route_after_send_error(&error) {
                    println!("Cached route appears stale; reacquiring route information.");
                    let _ = api.release_private_route(route_id.clone());
                    *cached_route_id = None;

                    if attempt < max_attempts {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        continue;
                    }
                }

                return Err(error);
            }
        }
    }

    bail!(
        "Unable to send message after {} attempts because route record keys remained unavailable.",
        max_attempts
    )
}

fn should_retry_route_acquire(error: &eyre::Report) -> bool {
    error
        .chain()
        .any(|cause| cause.to_string().contains("Key not found"))
}

fn should_reacquire_route_after_send_error(error: &eyre::Report) -> bool {
    error.chain().any(|cause| {
        let text = cause.to_string();
        text.contains("Route id does not exist")
            || text.contains("private route could not be found")
            || text.contains("Key not found")
    })
}

impl ToArgs for SendChatArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        let mut args = vec![];
        if let Some(message) = &self.message {
            args.push("--message".into());
            args.push(message.clone().into());
        }
        if let Some(retry) = self.retry {
            args.push("--retry".into());
            args.push(retry.to_string().into());
        }
        args
    }
}
