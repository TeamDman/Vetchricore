use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::global_args::GlobalArgs;
use crate::cli::veilid_runtime::start_api_for_profile;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;
use veilid_core::VeilidUpdate;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct RouteRemoveArgs {
    #[facet(args::positional)]
    pub name: String,
}

impl RouteRemoveArgs {
    /// # Errors
    ///
    /// Returns an error if the route does not exist, network readiness cannot be reached,
    /// DHT cleanup fails, or route identity persistence update fails.
    pub async fn invoke(self, global: &GlobalArgs) -> Result<()> {
        let profile = app_state::resolve_profile(global)?;
        let identity = app_state::local_route_identity(&profile, &self.name)?
            .ok_or_else(|| eyre::eyre!("Route '{}' does not exist.", self.name))?;

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
        wait_for_public_internet_ready(&api, &public_internet_ready).await?;

        let router = api.routing_context()?.with_default_safety()?;
        let opened = router
            .open_dht_record(identity.record_key.clone(), Some(identity.keypair.clone()))
            .await
            .is_ok();

        if opened {
            router.close_dht_record(identity.record_key.clone()).await?;
            router
                .delete_dht_record(identity.record_key.clone())
                .await?;
        }

        api.shutdown().await;

        app_state::remove_local_route_identity(&profile, &self.name)?;
        println!("Route '{}' has been removed.", self.name);
        Ok(())
    }
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
                eyre::bail!(
                    "Timed out waiting for public internet readiness; retry when network attachment improves."
                );
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    }
    Ok(())
}

impl ToArgs for RouteRemoveArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        vec![self.name.clone().into()]
    }
}
