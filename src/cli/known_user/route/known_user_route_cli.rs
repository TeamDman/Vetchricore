use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::known_user::route::add::KnownUserRouteAddArgs;
use crate::cli::known_user::route::list::KnownUserRouteListArgs;
use crate::cli::known_user::route::remove::KnownUserRouteRemoveArgs;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;
use std::ffi::OsString;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct KnownUserRouteArgs {
    #[facet(args::subcommand)]
    pub command: KnownUserRouteCommand,
}

#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum KnownUserRouteCommand {
    Add(KnownUserRouteAddArgs),
    List(KnownUserRouteListArgs),
    Remove(KnownUserRouteRemoveArgs),
}

impl KnownUserRouteArgs {
    /// # Errors
    ///
    /// Returns an error if the selected known-user route subcommand fails.
    pub async fn invoke(self, context: &InvokeContext) -> Result<()> {
        match self.command {
            KnownUserRouteCommand::Add(args) => args.invoke(context).await?,
            KnownUserRouteCommand::List(args) => args.invoke(context).await?,
            KnownUserRouteCommand::Remove(args) => args.invoke(context).await?,
        }
        Ok(())
    }
}

impl ToArgs for KnownUserRouteArgs {
    fn to_args(&self) -> Vec<OsString> {
        let mut args = Vec::new();
        match &self.command {
            KnownUserRouteCommand::Add(add_args) => {
                args.push("add".into());
                args.extend(add_args.to_args());
            }
            KnownUserRouteCommand::List(list_args) => {
                args.push("list".into());
                args.extend(list_args.to_args());
            }
            KnownUserRouteCommand::Remove(remove_args) => {
                args.push("remove".into());
                args.extend(remove_args.to_args());
            }
        }
        args
    }
}
