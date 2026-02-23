use crate::cli::Cli;
use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::app_state::LocalRouteIdentity;
use crate::cli::response::CliResponse;
use crate::cli::route::RouteArgs;
use crate::cli::route::RouteCommand;
use crate::cli::route::listen::RouteListenArgs;
use crate::cli::route::listen::listen_on_named_route;
use crate::cli::route::listen::wait_for_public_internet_ready;
use crate::cli::veilid_runtime::printing_update_callback;
use crate::cli::veilid_runtime::start_api_for_profile;
use arbitrary::Arbitrary;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use figue as args;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use veilid_core::CRYPTO_KIND_VLD0;
use veilid_core::DHTSchema;
use veilid_core::VeilidUpdate;

#[derive(Facet, Arbitrary, Debug, PartialEq, Default)]
pub struct RouteAddArgs {
    #[facet(args::positional)]
    pub name: String,

    #[facet(args::named, default)]
    pub listen: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct RouteAddResponse {
    name: String,
    record_key: String,
    profile: String,
    initialized_offline: bool,
}

impl fmt::Display for RouteAddResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Created route identity '{}' with record key {} for {}",
            self.name, self.record_key, self.profile
        )?;
        if self.initialized_offline {
            write!(f, "Initialized route record in offline state (empty route data).")
        } else {
            Ok(())
        }
    }
}

impl RouteAddArgs {
    pub async fn invoke(self, context: &InvokeContext) -> Result<CliResponse> {
        let profile_home = context.profile_home();
        if app_state::local_route_identity(profile_home, &self.name)?.is_some() {
            bail!(
                "Route '{}' already exists. Use '{}' to reuse it.",
                self.name,
                Cli::display_invocation(&RouteArgs {
                    command: RouteCommand::Listen(RouteListenArgs {
                        name: self.name.clone(),
                        count: None,
                    }),
                })
            );
        }

        let api =
            start_api_for_profile(profile_home, false, printing_update_callback(false)).await?;
        let crypto = api.crypto()?;
        let vcrypto = crypto
            .get_async(CRYPTO_KIND_VLD0)
            .ok_or_else(|| eyre::eyre!("VLD0 cryptosystem unavailable"))?;
        let route_keypair = vcrypto.generate_keypair().await;
        let record_encryption_key = vcrypto.random_shared_secret().await;
        let record_key = api.get_dht_record_key(
            DHTSchema::dflt(1)?,
            route_keypair.key().clone(),
            Some(record_encryption_key),
        )?;

        api.shutdown().await;

        let identity = LocalRouteIdentity {
            name: self.name.clone(),
            keypair: route_keypair,
            record_key,
        };
        app_state::add_local_route_identity(
            profile_home,
            &identity.name,
            &identity.keypair,
            &identity.record_key,
        )?;

        let record_key_text = identity.record_key.to_string();

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

        router
            .set_dht_value(identity.record_key.clone(), 0, Vec::new(), None)
            .await?;
        let _ = router.close_dht_record(identity.record_key.clone()).await;
        api.shutdown().await;

        if self.listen {
            listen_on_named_route(context, &self.name, None).await?;
            return Ok(CliResponse::empty());
        }

        CliResponse::from_facet(RouteAddResponse {
            name: self.name,
            record_key: record_key_text,
            profile: profile_home.profile().to_owned(),
            initialized_offline: true,
        })
    }
}

impl ToArgs for RouteAddArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        let mut args = vec![self.name.clone().into()];
        if self.listen {
            args.push("--listen".into());
        }
        args
    }
}
