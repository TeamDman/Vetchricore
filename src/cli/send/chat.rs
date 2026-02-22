use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::global_args::GlobalArgs;
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
    #[facet(args::positional)]
    pub to: String,
    #[facet(args::positional)]
    pub friend: String,
    #[facet(args::named)]
    pub message: Option<String>,
}

impl SendChatArgs {
    pub async fn invoke(self, global: &GlobalArgs) -> Result<()> {
        if self.to != "to" {
            bail!("Usage: send chat to <friend> [--message <text>]");
        }

        let profile = app_state::resolve_profile(global)?;
        let my_keypair = app_state::load_keypair(&profile)?
            .ok_or_else(|| eyre::eyre!("You have no key. Run 'vetchricore key gen' first."))?;
        let friend_key = app_state::friend_public_key(&profile, &self.friend)?.ok_or_else(|| {
            eyre::eyre!(
                "Friend '{}' not found. Add them with 'vetchricore friend add <name> <pubkey>'.",
                self.friend
            )
        })?;

        let keys = app_state::route_keys_for_friend(&profile, &self.friend)?;
        if keys.is_empty() {
            bail!("No route record keys configured for {}.", self.friend);
        }

        let public_internet_ready = Arc::new(AtomicBool::new(false));
        let callback = {
            let public_internet_ready = Arc::clone(&public_internet_ready);
            Arc::new(move |update: VeilidUpdate| {
                if let VeilidUpdate::Attachment(attachment) = update
                    && attachment.public_internet_ready
                {
                    public_internet_ready.store(true, Ordering::Release);
                }
            }) as crate::cli::veilid_runtime::UpdateCallback
        };

        let api = start_api_for_profile(&profile, true, callback).await?;
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

        let mut route_id = acquire_best_route(&api, &router, &keys).await?;

        if let Some(message) = self.message {
            let payload = format!("{}|{}", my_keypair.key(), message);
            send_payload_with_retry(&api, &router, &keys, &mut route_id, payload.into_bytes())
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
                        send_payload_with_retry(&api, &router, &keys, &mut route_id, payload.into_bytes())
                            .await?;
                    }
                }
            }
        }

        let _ = api.release_private_route(route_id);
        api.shutdown().await;
        let _ = friend_key;
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

async fn send_payload_with_retry(
    api: &veilid_core::VeilidAPI,
    router: &veilid_core::RoutingContext,
    keys: &[RecordKey],
    route_id: &mut RouteId,
    payload: Vec<u8>,
) -> Result<()> {
    let first_try = router
        .app_message(Target::RouteId(route_id.clone()), payload.clone())
        .await;
    if first_try.is_ok() {
        return Ok(());
    }

    println!("Route send failed; reacquiring route information and retrying...");
    let _ = api.release_private_route(route_id.clone());
    *route_id = acquire_best_route(api, router, keys).await?;

    router
        .app_message(Target::RouteId(route_id.clone()), payload)
        .await?;
    Ok(())
}

impl ToArgs for SendChatArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        let mut args = vec![self.to.clone().into(), self.friend.clone().into()];
        if let Some(message) = &self.message {
            args.push("--message".into());
            args.push(message.clone().into());
        }
        args
    }
}
