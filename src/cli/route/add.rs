use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::global_args::GlobalArgs;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;
use veilid_core::RecordKey;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct RouteAddArgs {
    #[facet(args::named)]
    pub known_user: String,
    #[facet(args::named)]
    pub record_key: String,
}

impl RouteAddArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, global: &GlobalArgs) -> Result<()> {
        let profile = app_state::resolve_profile(global)?;
        let key = self.record_key.parse::<RecordKey>()?;
        app_state::add_route_key(&profile, &self.known_user, &key)?;
        println!("Added a route to {} for {}.", self.known_user, profile);
        Ok(())
    }
}

impl ToArgs for RouteAddArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        vec![
            "--known-user".into(),
            self.known_user.clone().into(),
            "--record-key".into(),
            self.record_key.clone().into(),
        ]
    }
}
