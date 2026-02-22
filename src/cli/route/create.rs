use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::app_state::LocalRouteIdentity;
use crate::cli::global_args::GlobalArgs;
use crate::cli::route::listen::listen_on_named_route;
use crate::cli::veilid_runtime::printing_update_callback;
use crate::cli::veilid_runtime::start_api_for_profile;
use arbitrary::Arbitrary;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use figue as args;
use veilid_core::CRYPTO_KIND_VLD0;
use veilid_core::DHTSchema;

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

        let api = start_api_for_profile(&profile, false, printing_update_callback(false)).await?;
        let crypto = api.crypto()?;
        let vcrypto = crypto
            .get_async(CRYPTO_KIND_VLD0)
            .ok_or_else(|| eyre::eyre!("VLD0 cryptosystem unavailable"))?;
        let route_keypair = vcrypto.generate_keypair().await;
        let record_key =
            api.get_dht_record_key(DHTSchema::dflt(1)?, route_keypair.key().clone(), None)?;

        api.shutdown().await;

        let identity = LocalRouteIdentity {
            name: self.name.clone(),
            keypair: route_keypair,
            record_key,
        };
        app_state::add_local_route_identity(
            &profile,
            &identity.name,
            &identity.keypair,
            &identity.record_key,
        )?;

        println!(
            "Created route identity '{}' with record key {} for {}",
            self.name, identity.record_key, profile
        );

        if self.listen {
            listen_on_named_route(global, &self.name).await?;
        }

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
