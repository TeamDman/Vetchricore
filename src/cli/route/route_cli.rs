use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::response::CliResponse;
use crate::cli::route::add::RouteAddArgs;
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
    Add(RouteAddArgs),
    New(RouteAddArgs),
    Create(RouteAddArgs),
    Listen(RouteListenArgs),
    List(RouteListArgs),
    Show(RouteShowArgs),
    Remove(RouteRemoveArgs),
}

impl RouteArgs {
    /// # Errors
    ///
    /// Returns an error if the selected route subcommand fails.
    pub async fn invoke(self, context: &InvokeContext) -> Result<CliResponse> {
        Ok(match self.command {
            RouteCommand::Add(args) => args.invoke(context).await?.into(),
            RouteCommand::New(args) => args.invoke(context).await?.into(),
            RouteCommand::Create(args) => args.invoke(context).await?.into(),
            RouteCommand::Listen(args) => {
                args.invoke(context).await?;
                CliResponse::empty()
            }
            RouteCommand::List(args) => args.invoke(context).await?.into(),
            RouteCommand::Show(args) => args.invoke(context).await?.into(),
            RouteCommand::Remove(args) => args.invoke(context).await?.into(),
        })
    }
}

impl ToArgs for RouteArgs {
    fn to_args(&self) -> Vec<OsString> {
        let mut args = Vec::new();
        match &self.command {
            RouteCommand::Add(add_args) => {
                args.push("add".into());
                args.extend(add_args.to_args());
            }
            RouteCommand::New(new_args) => {
                args.push("new".into());
                args.extend(new_args.to_args());
            }
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
