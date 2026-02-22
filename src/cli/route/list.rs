use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
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
    pub async fn invoke(self, context: &InvokeContext) -> Result<()> {
        let routes = app_state::list_local_route_identities(context.profile_home())?;

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
