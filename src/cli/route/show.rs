use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use arbitrary::Arbitrary;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use figue as args;
use std::fmt;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct RouteShowArgs {
    #[facet(args::positional)]
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct RouteShowResponse {
    name: String,
    record_key: String,
    public_key: String,
}

impl fmt::Display for RouteShowResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Route: {}", self.name)?;
        writeln!(f, "Record key: {}", self.record_key)?;
        write!(f, "Public key: {}", self.public_key)
    }
}

impl RouteShowArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<RouteShowResponse> {
        let Some(route) = app_state::local_route_identity(context.profile_home(), &self.name)?
        else {
            bail!("Route '{}' does not exist.", self.name);
        };

        Ok(RouteShowResponse {
            name: route.name,
            record_key: route.record_key.to_string(),
            public_key: route.keypair.key().to_string(),
        })
    }
}

impl ToArgs for RouteShowArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        vec![self.name.clone().into()]
    }
}
