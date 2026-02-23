use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::veilid_runtime::printing_update_callback;
use crate::cli::veilid_runtime::start_api_for_profile;
use arbitrary::Arbitrary;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::fmt;
use veilid_core::CRYPTO_KIND_VLD0;

#[derive(Facet, Arbitrary, Debug, PartialEq, Default)]
pub struct KeyGenArgs;

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct KeyGenResponse {
    public_key: String,
    private_key_hidden: bool,
}

impl fmt::Display for KeyGenResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Public key: {}", self.public_key)?;
        if self.private_key_hidden {
            write!(f, "Private key: this value is hidden")
        } else {
            write!(f, "Private key: <unavailable>")
        }
    }
}

impl KeyGenArgs {
    pub async fn invoke(self, context: &InvokeContext) -> Result<KeyGenResponse> {
        let profile_home = context.profile_home();
        if app_state::load_keypair(profile_home)?.is_some() {
            bail!("You already have a keypair.");
        }

        let api =
            start_api_for_profile(profile_home, false, printing_update_callback(false)).await?;
        let crypto = api.crypto()?;
        let vcrypto = crypto
            .get_async(CRYPTO_KIND_VLD0)
            .ok_or_else(|| eyre::eyre!("VLD0 cryptosystem unavailable"))?;
        let keypair = vcrypto.generate_keypair().await;
        api.shutdown().await;

        app_state::store_keypair(profile_home, &keypair)?;
        Ok(KeyGenResponse {
            public_key: keypair.key().to_string(),
            private_key_hidden: true,
        })
    }
}

impl ToArgs for KeyGenArgs {}
