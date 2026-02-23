use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::response::CliResponse;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;
use std::fmt;

#[derive(Facet, Arbitrary, Debug, PartialEq, Default)]
pub struct KnownUserRouteListArgs {
    #[facet(args::named)]
    pub known_user: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct KnownUserRouteListItem {
    known_user: String,
    record_key: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct KnownUserRouteListResponse {
    routes: Vec<KnownUserRouteListItem>,
}

impl fmt::Display for KnownUserRouteListResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.routes.is_empty() {
            return f.write_str("No known-user routes have been added.");
        }

        for (index, route) in self.routes.iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
            }
            write!(f, "{} ({})", route.known_user, route.record_key)?;
        }
        Ok(())
    }
}

impl KnownUserRouteListArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<CliResponse> {
        let routes = app_state::list_known_user_route_keys(
            context.profile_home(),
            self.known_user.as_deref(),
        )?;

        let response = KnownUserRouteListResponse {
            routes: routes
                .into_iter()
                .map(|route| KnownUserRouteListItem {
                    known_user: route.known_user,
                    record_key: route.record_key.to_string(),
                })
                .collect(),
        };

        Ok(response.into())
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

