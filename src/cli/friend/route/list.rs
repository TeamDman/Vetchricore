use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::global_args::GlobalArgs;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;

#[derive(Facet, Arbitrary, Debug, PartialEq, Default)]
pub struct FriendRouteListArgs {
    #[facet(args::named)]
    pub friend: Option<String>,
}

impl FriendRouteListArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, global: &GlobalArgs) -> Result<()> {
        let profile = app_state::resolve_profile(global)?;
        let routes = app_state::list_friend_route_keys(&profile, self.friend.as_deref())?;

        if routes.is_empty() {
            println!("No friend routes have been added.");
            return Ok(());
        }

        for route in routes {
            println!("{} ({})", route.friend, route.record_key);
        }
        Ok(())
    }
}

impl ToArgs for FriendRouteListArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        let mut args = Vec::new();
        if let Some(friend) = &self.friend {
            args.push("--friend".into());
            args.push(friend.clone().into());
        }
        args
    }
}
