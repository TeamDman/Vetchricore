use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::known_user::add::KnownUserAddArgs;
use crate::cli::known_user::list::KnownUserListArgs;
use crate::cli::known_user::remove::KnownUserRemoveArgs;
use crate::cli::known_user::rename::KnownUserRenameArgs;
use crate::cli::known_user::route::KnownUserRouteArgs;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;
use std::ffi::OsString;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct KnownUserArgs {
    #[facet(args::subcommand)]
    pub command: KnownUserCommand,
}

#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum KnownUserCommand {
    List(KnownUserListArgs),
    Add(KnownUserAddArgs),
    New(KnownUserAddArgs),
    Create(KnownUserAddArgs),
    Rename(KnownUserRenameArgs),
    Remove(KnownUserRemoveArgs),
    Route(KnownUserRouteArgs),
}

impl KnownUserArgs {
    /// # Errors
    ///
    /// Returns an error if the selected known-user subcommand fails.
    pub async fn invoke(self, context: &InvokeContext) -> Result<()> {
        match self.command {
            KnownUserCommand::List(args) => args.invoke(context).await?,
            KnownUserCommand::Add(args) => args.invoke(context).await?,
            KnownUserCommand::New(args) => args.invoke(context).await?,
            KnownUserCommand::Create(args) => args.invoke(context).await?,
            KnownUserCommand::Rename(args) => args.invoke(context).await?,
            KnownUserCommand::Remove(args) => args.invoke(context).await?,
            KnownUserCommand::Route(args) => args.invoke(context).await?,
        }
        Ok(())
    }
}

impl ToArgs for KnownUserArgs {
    fn to_args(&self) -> Vec<OsString> {
        let mut args = Vec::new();
        match &self.command {
            KnownUserCommand::List(list_args) => {
                args.push("list".into());
                args.extend(list_args.to_args());
            }
            KnownUserCommand::Add(add_args) => {
                args.push("add".into());
                args.extend(add_args.to_args());
            }
            KnownUserCommand::New(new_args) => {
                args.push("new".into());
                args.extend(new_args.to_args());
            }
            KnownUserCommand::Create(create_args) => {
                args.push("create".into());
                args.extend(create_args.to_args());
            }
            KnownUserCommand::Rename(rename_args) => {
                args.push("rename".into());
                args.extend(rename_args.to_args());
            }
            KnownUserCommand::Remove(remove_args) => {
                args.push("remove".into());
                args.extend(remove_args.to_args());
            }
            KnownUserCommand::Route(route_args) => {
                args.push("route".into());
                args.extend(route_args.to_args());
            }
        }
        args
    }
}
