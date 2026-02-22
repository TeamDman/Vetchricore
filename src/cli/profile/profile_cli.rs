use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::profile::add::ProfileAddArgs;
use crate::cli::profile::list::ProfileListArgs;
use crate::cli::profile::remove::ProfileRemoveArgs;
use crate::cli::profile::show::ProfileShowArgs;
use crate::cli::profile::use_profile::ProfileUseArgs;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;
use std::ffi::OsString;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct ProfileArgs {
    #[facet(args::subcommand)]
    pub command: ProfileCommand,
}

#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum ProfileCommand {
    Add(ProfileAddArgs),
    List(ProfileListArgs),
    Use(ProfileUseArgs),
    Remove(ProfileRemoveArgs),
    Show(ProfileShowArgs),
}

impl ProfileArgs {
    /// # Errors
    ///
    /// Returns an error if the selected profile subcommand fails.
    pub async fn invoke(self, context: &InvokeContext) -> Result<()> {
        match self.command {
            ProfileCommand::Add(args) => args.invoke(context).await?,
            ProfileCommand::List(args) => args.invoke(context).await?,
            ProfileCommand::Use(args) => args.invoke(context).await?,
            ProfileCommand::Remove(args) => args.invoke(context).await?,
            ProfileCommand::Show(args) => args.invoke(context).await?,
        }
        Ok(())
    }
}

impl ToArgs for ProfileArgs {
    fn to_args(&self) -> Vec<OsString> {
        let mut args = Vec::new();
        match &self.command {
            ProfileCommand::Add(add_args) => {
                args.push("add".into());
                args.extend(add_args.to_args());
            }
            ProfileCommand::List(list_args) => {
                args.push("list".into());
                args.extend(list_args.to_args());
            }
            ProfileCommand::Use(use_args) => {
                args.push("use".into());
                args.extend(use_args.to_args());
            }
            ProfileCommand::Remove(remove_args) => {
                args.push("remove".into());
                args.extend(remove_args.to_args());
            }
            ProfileCommand::Show(show_args) => {
                args.push("show".into());
                args.extend(show_args.to_args());
            }
        }
        args
    }
}
