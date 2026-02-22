use crate::cli::ToArgs;
use crate::cli::friend::add::FriendAddArgs;
use crate::cli::friend::list::FriendListArgs;
use crate::cli::friend::remove::FriendRemoveArgs;
use crate::cli::friend::rename::FriendRenameArgs;
use crate::cli::friend::route::FriendRouteArgs;
use crate::cli::global_args::GlobalArgs;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;
use std::ffi::OsString;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct FriendArgs {
    #[facet(args::subcommand)]
    pub command: FriendCommand,
}

#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum FriendCommand {
    List(FriendListArgs),
    Add(FriendAddArgs),
    Rename(FriendRenameArgs),
    Remove(FriendRemoveArgs),
    Route(FriendRouteArgs),
}

impl FriendArgs {
    /// # Errors
    ///
    /// Returns an error if the selected friend subcommand fails.
    pub async fn invoke(self, global: &GlobalArgs) -> Result<()> {
        match self.command {
            FriendCommand::List(args) => args.invoke(global).await?,
            FriendCommand::Add(args) => args.invoke(global).await?,
            FriendCommand::Rename(args) => args.invoke(global).await?,
            FriendCommand::Remove(args) => args.invoke(global).await?,
            FriendCommand::Route(args) => args.invoke(global).await?,
        }
        Ok(())
    }
}

impl ToArgs for FriendArgs {
    fn to_args(&self) -> Vec<OsString> {
        let mut args = Vec::new();
        match &self.command {
            FriendCommand::List(list_args) => {
                args.push("list".into());
                args.extend(list_args.to_args());
            }
            FriendCommand::Add(add_args) => {
                args.push("add".into());
                args.extend(add_args.to_args());
            }
            FriendCommand::Rename(rename_args) => {
                args.push("rename".into());
                args.extend(rename_args.to_args());
            }
            FriendCommand::Remove(remove_args) => {
                args.push("remove".into());
                args.extend(remove_args.to_args());
            }
            FriendCommand::Route(route_args) => {
                args.push("route".into());
                args.extend(route_args.to_args());
            }
        }
        args
    }
}
