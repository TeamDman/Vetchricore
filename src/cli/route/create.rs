use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::app_state::LocalRouteIdentity;
use crate::cli::global_args::GlobalArgs;
use crate::cli::veilid_runtime::start_api_for_profile;
use arbitrary::Arbitrary;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use figue as args;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;
use veilid_core::CRYPTO_KIND_VLD0;
use veilid_core::DHTSchema;
use veilid_core::VeilidUpdate;

#[derive(Facet, Arbitrary, Debug, PartialEq, Default)]
pub struct RouteCreateArgs {
    #[facet(args::positional)]
    pub name: String,

    #[facet(args::named, default)]
    pub listen: bool,
}

impl RouteCreateArgs {
    pub async fn invoke(self, global: &GlobalArgs) -> Result<()> {
        let profile = app_state::resolve_profile(global)?;
        if app_state::local_route_identity(&profile, &self.name)?.is_some() {
            bail!(
                "Route '{}' already exists. Use 'vetchricore route listen {}' to reuse it.",
                self.name,
                self.name
            );
        }

        let public_internet_ready = Arc::new(AtomicBool::new(false));
        let friend_map = Arc::new(Mutex::new(
            app_state::list_friends(&profile)?
                .into_iter()
                .map(|f| (f.pubkey.to_string(), f.name))
                .collect::<std::collections::HashMap<_, _>>(),
        ));

        let callback =
            route_update_callback(Arc::clone(&public_internet_ready), Arc::clone(&friend_map));

        let api = start_api_for_profile(&profile, true, callback).await?;
        wait_for_public_internet_ready(&api, &public_internet_ready).await?;

        let router = api.routing_context()?.with_default_safety()?;
        let crypto = api.crypto()?;
        let vcrypto = crypto
            .get_async(CRYPTO_KIND_VLD0)
            .ok_or_else(|| eyre::eyre!("VLD0 cryptosystem unavailable"))?;
        let route_keypair = vcrypto.generate_keypair().await;

        let record = router
            .create_dht_record(
                CRYPTO_KIND_VLD0,
                DHTSchema::dflt(1)?,
                Some(route_keypair.clone()),
            )
            .await?;

        let identity = LocalRouteIdentity {
            name: self.name.clone(),
            keypair: route_keypair,
            record_key: record.key().clone(),
        };
        app_state::add_local_route_identity(
            &profile,
            &identity.name,
            &identity.keypair,
            &identity.record_key,
        )?;

        let route_blob = api.new_private_route().await?;
        router
            .set_dht_value(record.key().clone(), 0, route_blob.blob.clone(), None)
            .await?;

        println!(
            "Created route information and stored it under record key {} for {}",
            record.key(),
            profile
        );

        if self.listen {
            let route_blob = api.new_private_route().await?;
            router
                .set_dht_value(record.key().clone(), 0, route_blob.blob.clone(), None)
                .await?;
            println!("Listening for messages.");
            tokio::signal::ctrl_c().await?;

            let _ = router
                .set_dht_value(record.key().clone(), 0, Vec::new(), None)
                .await;
            let _ = api.release_private_route(route_blob.route_id);
        }

        let _ = router.close_dht_record(record.key().clone()).await;
        api.shutdown().await;

        Ok(())
    }
}

impl ToArgs for RouteCreateArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        let mut args = vec![self.name.clone().into()];
        if self.listen {
            args.push("--listen".into());
        }
        args
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
    let identity = app_state::local_route_identity(&profile, route_name)?.ok_or_else(|| {
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
    let callback = route_update_callback(Arc::clone(&public_internet_ready), friend_map);

    let api = start_api_for_profile(&profile, true, callback).await?;
    wait_for_public_internet_ready(&api, &public_internet_ready).await?;

    let router = api.routing_context()?.with_default_safety()?;
    let _ = router
        .open_dht_record(identity.record_key.clone(), Some(identity.keypair.clone()))
        .await?;

    let route_blob = api.new_private_route().await?;
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
    tokio::signal::ctrl_c().await?;

    let _ = router
        .set_dht_value(identity.record_key.clone(), 0, Vec::new(), None)
        .await;
    let _ = api.release_private_route(route_blob.route_id);
    let _ = router.close_dht_record(identity.record_key).await;
    api.shutdown().await;
    Ok(())
}

fn route_update_callback(
    public_internet_ready: Arc<AtomicBool>,
    friend_map: Arc<Mutex<std::collections::HashMap<String, String>>>,
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
