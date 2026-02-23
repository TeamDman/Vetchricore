use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;
use veilid_core::RecordKey;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct KnownUserRouteAddArgs {
    #[facet(args::named)]
    pub known_user: String,
    #[facet(args::named)]
    pub record_key: String,
}

impl KnownUserRouteAddArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<()> {
        let key = self.record_key.parse::<RecordKey>()?;
        app_state::add_route_key(context.profile_home(), &self.known_user, &key)?;
        println!(
            "Added a route to {} for {}.",
            self.known_user,
            context.profile_home().profile()
        );
        Ok(())
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
