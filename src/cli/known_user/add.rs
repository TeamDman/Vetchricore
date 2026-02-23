use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::response::CliResponse;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;
use std::fmt;
use veilid_core::PublicKey;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct KnownUserAddArgs {
    #[facet(args::positional)]
    pub name: String,
    #[facet(args::positional)]
    pub pubkey: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct KnownUserAddResponse {
    name: String,
}

impl fmt::Display for KnownUserAddResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "You have added {} as a known user.", self.name)
    }
}

impl KnownUserAddArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<CliResponse> {
        let profile_home = context.profile_home();
        let pubkey = self.pubkey.parse::<PublicKey>()?;
        app_state::add_known_user(profile_home, &self.name, pubkey)?;
        Ok(KnownUserAddResponse { name: self.name }.into())
    }
}

impl ToArgs for KnownUserAddArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        vec![self.name.clone().into(), self.pubkey.clone().into()]
    }
}

