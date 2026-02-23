use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;

#[derive(Facet, Arbitrary, Debug, PartialEq, Default)]
pub struct KnownUserRouteListArgs {
    #[facet(args::named)]
    pub known_user: Option<String>,
}

impl KnownUserRouteListArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<()> {
        let routes = app_state::list_known_user_route_keys(
            context.profile_home(),
            self.known_user.as_deref(),
        )?;

        if routes.is_empty() {
            println!("No known-user routes have been added.");
            return Ok(());
        }

        for route in routes {
            println!("{} ({})", route.known_user, route.record_key);
        }
        Ok(())
    }
}

impl ToArgs for KnownUserRouteListArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        let mut args = Vec::new();
        if let Some(known_user) = &self.known_user {
            args.push("--known-user".into());
            args.push(known_user.clone().into());
        }
        args
    }
}
