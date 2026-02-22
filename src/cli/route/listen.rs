use crate::cli::ToArgs;
use crate::cli::global_args::GlobalArgs;
use crate::cli::route::create::listen_on_named_route;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct RouteListenArgs {
    #[facet(args::positional)]
    pub name: String,
}

impl RouteListenArgs {
    /// # Errors
    ///
    /// Returns an error if the route is not found or listening fails.
    pub async fn invoke(self, global: &GlobalArgs) -> Result<()> {
        listen_on_named_route(global, &self.name).await
    }
}

impl ToArgs for RouteListenArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        vec![self.name.clone().into()]
    }
}
