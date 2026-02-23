use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;
use std::fmt;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct KnownUserRemoveArgs {
    #[facet(args::positional)]
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct KnownUserRemoveResponse {
    name: String,
}

impl fmt::Display for KnownUserRemoveResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} is no longer a known user.", self.name)
    }
}

impl KnownUserRemoveArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<KnownUserRemoveResponse> {
        app_state::remove_known_user(context.profile_home(), &self.name)?;
        Ok(KnownUserRemoveResponse { name: self.name })
    }
}

impl ToArgs for KnownUserRemoveArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        vec![self.name.clone().into()]
    }
}
