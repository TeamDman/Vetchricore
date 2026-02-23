use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::response::CliResponse;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;
use std::fmt;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct ProfileAddArgs {
    #[facet(args::positional)]
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct ProfileAddResponse {
    name: String,
}

impl fmt::Display for ProfileAddResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Profile '{}' has been created.", self.name)
    }
}

impl ProfileAddArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<CliResponse> {
        app_state::ensure_initialized(context.app_home())?;
        app_state::create_profile(context.app_home(), &self.name)?;
        CliResponse::from_facet(ProfileAddResponse { name: self.name })
    }
}

impl ToArgs for ProfileAddArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        vec![self.name.clone().into()]
    }
}
