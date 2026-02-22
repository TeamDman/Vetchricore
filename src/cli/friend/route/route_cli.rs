use crate::cli::ToArgs;
use crate::cli::friend::route::add::FriendRouteAddArgs;
use crate::cli::friend::route::list::FriendRouteListArgs;
use crate::cli::friend::route::remove::FriendRouteRemoveArgs;
use crate::cli::global_args::GlobalArgs;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;
use std::ffi::OsString;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct FriendRouteArgs {
    #[facet(args::subcommand)]
    pub command: FriendRouteCommand,
}

#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum FriendRouteCommand {
    Add(FriendRouteAddArgs),
    List(FriendRouteListArgs),
    Remove(FriendRouteRemoveArgs),
}

impl FriendRouteArgs {
    /// # Errors
    ///
    /// Returns an error if the selected friend route subcommand fails.
    pub async fn invoke(self, global: &GlobalArgs) -> Result<()> {
        match self.command {
            FriendRouteCommand::Add(args) => args.invoke(global).await?,
            FriendRouteCommand::List(args) => args.invoke(global).await?,
            FriendRouteCommand::Remove(args) => args.invoke(global).await?,
        }
        Ok(())
    }
}

impl ToArgs for FriendRouteArgs {
    fn to_args(&self) -> Vec<OsString> {
        let mut args = Vec::new();
        match &self.command {
            FriendRouteCommand::Add(add_args) => {
                args.push("add".into());
                args.extend(add_args.to_args());
            }
            FriendRouteCommand::List(list_args) => {
                args.push("list".into());
                args.extend(list_args.to_args());
            }
            FriendRouteCommand::Remove(remove_args) => {
                args.push("remove".into());
                args.extend(remove_args.to_args());
            }
        }
        args
    }
}
