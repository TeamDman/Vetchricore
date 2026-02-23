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
pub struct KnownUserRenameArgs {
    #[facet(args::positional)]
    pub old_name: String,
    #[facet(args::positional)]
    pub new_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct KnownUserRenameResponse {
    old_name: String,
    new_name: String,
}

impl fmt::Display for KnownUserRenameResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} has been renamed to {}.", self.old_name, self.new_name)
    }
}

impl KnownUserRenameArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<CliResponse> {
        app_state::rename_known_user(context.profile_home(), &self.old_name, &self.new_name)?;
        CliResponse::from_facet(KnownUserRenameResponse {
            old_name: self.old_name,
            new_name: self.new_name,
        })
    }
}

impl ToArgs for KnownUserRenameArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        vec![self.old_name.clone().into(), self.new_name.clone().into()]
    }
}
