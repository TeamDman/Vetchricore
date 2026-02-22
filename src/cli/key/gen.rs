use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::global_args::GlobalArgs;
use crate::cli::veilid_runtime::{printing_update_callback, start_api_for_profile};
use arbitrary::Arbitrary;
use eyre::{Result, bail};
use facet::Facet;
use veilid_core::CRYPTO_KIND_VLD0;

#[derive(Facet, Arbitrary, Debug, PartialEq, Default)]
pub struct KeyGenArgs;

impl KeyGenArgs {
    pub async fn invoke(self, global: &GlobalArgs) -> Result<()> {
        let profile = app_state::resolve_profile(global)?;
        if app_state::load_keypair(&profile)?.is_some() {
            bail!("You already have a keypair.");
        }

        let api = start_api_for_profile(&profile, false, printing_update_callback(false)).await?;
        let crypto = api.crypto()?;
        let vcrypto = crypto
            .get_async(CRYPTO_KIND_VLD0)
            .ok_or_else(|| eyre::eyre!("VLD0 cryptosystem unavailable"))?;
        let keypair = vcrypto.generate_keypair().await;
        api.shutdown().await;

        app_state::store_keypair(&profile, &keypair)?;
        println!("Public key: {}", keypair.key());
        println!("Private key: this value is hidden");

        Ok(())
    }
}

impl ToArgs for KeyGenArgs {}
