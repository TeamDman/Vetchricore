use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::global_args::GlobalArgs;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;
use veilid_core::RecordKey;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct FriendRouteAddArgs {
    #[facet(args::named)]
    pub friend: String,
    #[facet(args::named)]
    pub record_id: String,
}

impl FriendRouteAddArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, global: &GlobalArgs) -> Result<()> {
        let profile = app_state::resolve_profile(global)?;
        let key = self.record_id.parse::<RecordKey>()?;
        app_state::add_route_key(&profile, &self.friend, &key)?;
        println!("Added a route to {} for {}.", self.friend, profile);
        Ok(())
    }
}

impl ToArgs for FriendRouteAddArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        vec![
            "--friend".into(),
            self.friend.clone().into(),
            "--record-id".into(),
            self.record_id.clone().into(),
        ]
    }
}
