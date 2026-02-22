use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
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
    pub async fn invoke(self, context: &InvokeContext) -> Result<()> {
        let key = self.record_id.parse::<RecordKey>()?;
        app_state::add_route_key(context.profile_home(), &self.friend, &key)?;
        println!(
            "Added a route to {} for {}.",
            self.friend,
            context.profile_home().profile()
        );
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
