use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::global_args::GlobalArgs;
use arbitrary::Arbitrary;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use figue as args;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct RouteShowArgs {
    #[facet(args::positional)]
    pub name: String,
}

impl RouteShowArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, global: &GlobalArgs) -> Result<()> {
        let profile = app_state::resolve_profile(global)?;
        let Some(route) = app_state::local_route_identity(&profile, &self.name)? else {
            bail!("Route '{}' does not exist.", self.name);
        };

        println!("Route: {}", route.name);
        println!("Record key: {}", route.record_key);
        println!("Public key: {}", route.keypair.key());
        Ok(())
    }
}

impl ToArgs for RouteShowArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        vec![self.name.clone().into()]
    }
}
