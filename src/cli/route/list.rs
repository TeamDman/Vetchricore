use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::response::CliResponse;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use std::fmt;

#[derive(Facet, Arbitrary, Debug, PartialEq, Default)]
pub struct RouteListArgs;

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct RouteListItem {
    name: String,
    record_key: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct RouteListResponse {
    routes: Vec<RouteListItem>,
}

impl fmt::Display for RouteListResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.routes.is_empty() {
            return f.write_str("No routes have been created.");
        }

        for (index, route) in self.routes.iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
            }
            write!(f, "{} ({})", route.name, route.record_key)?;
        }
        Ok(())
    }
}

impl RouteListArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<CliResponse> {
        let routes = app_state::list_local_route_identities(context.profile_home())?;
        let response = RouteListResponse {
            routes: routes
                .into_iter()
                .map(|route| RouteListItem {
                    name: route.name,
                    record_key: route.record_key.to_string(),
                })
                .collect(),
        };
        CliResponse::from_facet(response)
    }
}

impl ToArgs for RouteListArgs {}
