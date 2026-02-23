use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::response::CliResponse;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;
use std::fmt;
use veilid_core::RecordKey;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct KnownUserRouteAddArgs {
    #[facet(args::named)]
    pub known_user: String,
    #[facet(args::named)]
    pub record_key: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct KnownUserRouteAddResponse {
    known_user: String,
    profile: String,
}

impl fmt::Display for KnownUserRouteAddResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Added a route to {} for {}.",
            self.known_user, self.profile
        )
    }
}

impl KnownUserRouteAddArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<CliResponse> {
        let key = self.record_key.parse::<RecordKey>()?;
        app_state::add_route_key(context.profile_home(), &self.known_user, &key)?;
        CliResponse::from_facet(KnownUserRouteAddResponse {
            known_user: self.known_user,
            profile: context.profile_home().profile().to_owned(),
        })
    }
}

impl ToArgs for KnownUserRouteAddArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        vec![
            "--known-user".into(),
            self.known_user.clone().into(),
            "--record-key".into(),
            self.record_key.clone().into(),
        ]
    }
}
