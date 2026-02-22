use crate::cli::ToArgs;
use crate::cli::global_args::GlobalArgs;
use crate::cli::route::create::RouteCreateArgs;
use crate::cli::route::list::RouteListArgs;
use crate::cli::route::listen::RouteListenArgs;
use crate::cli::route::remove::RouteRemoveArgs;
use crate::cli::route::show::RouteShowArgs;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;
use std::ffi::OsString;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct RouteArgs {
    #[facet(args::subcommand)]
    pub command: RouteCommand,
}

#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum RouteCommand {
    Create(RouteCreateArgs),
    Listen(RouteListenArgs),
    List(RouteListArgs),
    Show(RouteShowArgs),
    Remove(RouteRemoveArgs),
}

impl RouteArgs {
    /// # Errors
    ///
    /// Returns an error if the selected route subcommand fails.
    pub async fn invoke(self, global: &GlobalArgs) -> Result<()> {
        match self.command {
            RouteCommand::Create(args) => args.invoke(global).await?,
            RouteCommand::Listen(args) => args.invoke(global).await?,
            RouteCommand::List(args) => args.invoke(global).await?,
            RouteCommand::Show(args) => args.invoke(global).await?,
            RouteCommand::Remove(args) => args.invoke(global).await?,
        }
        Ok(())
    }
}

impl ToArgs for RouteArgs {
    fn to_args(&self) -> Vec<OsString> {
        let mut args = Vec::new();
        match &self.command {
            RouteCommand::Create(create_args) => {
                args.push("create".into());
                args.extend(create_args.to_args());
            }
            RouteCommand::Listen(listen_args) => {
                args.push("listen".into());
                args.extend(listen_args.to_args());
            }
            RouteCommand::List(list_args) => {
                args.push("list".into());
                args.extend(list_args.to_args());
            }
            RouteCommand::Show(show_args) => {
                args.push("show".into());
                args.extend(show_args.to_args());
            }
            RouteCommand::Remove(remove_args) => {
                args.push("remove".into());
                args.extend(remove_args.to_args());
            }
        }
        args
    }
}
