use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::global_args::GlobalArgs;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;

#[derive(Facet, Arbitrary, Debug, PartialEq, Default)]
pub struct RouteListArgs;

impl RouteListArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, global: &GlobalArgs) -> Result<()> {
        let profile = app_state::resolve_profile(global)?;
        let routes = app_state::list_local_route_identities(&profile)?;

        if routes.is_empty() {
            println!("No routes have been created.");
            return Ok(());
        }

        for route in routes {
            println!("{} ({})", route.name, route.record_key);
        }
        Ok(())
    }
}

impl ToArgs for RouteListArgs {}
