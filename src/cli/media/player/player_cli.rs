use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::media::player::add::MediaPlayerAddArgs;
use crate::cli::media::player::default::MediaPlayerDefaultArgs;
use crate::cli::media::player::detect::MediaPlayerDetectArgs;
use crate::cli::media::player::list::MediaPlayerListArgs;
use crate::cli::media::player::show::MediaPlayerShowArgs;
use crate::cli::response::CliResponse;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;
use std::ffi::OsString;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct MediaPlayerArgs {
    #[facet(args::subcommand)]
    pub command: MediaPlayerCommand,
}

#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum MediaPlayerCommand {
    List(MediaPlayerListArgs),
    Add(MediaPlayerAddArgs),
    New(MediaPlayerAddArgs),
    Set(MediaPlayerAddArgs),
    Update(MediaPlayerAddArgs),
    Create(MediaPlayerAddArgs),
    Show(MediaPlayerShowArgs),
    Default(MediaPlayerDefaultArgs),
    Detect(MediaPlayerDetectArgs),
    Discover(MediaPlayerDetectArgs),
}

impl MediaPlayerArgs {
    /// # Errors
    ///
    /// Returns an error if the selected media-player subcommand fails.
    pub async fn invoke(self, context: &InvokeContext) -> Result<CliResponse> {
        match self.command {
            MediaPlayerCommand::List(args) => args.invoke(context).await,
            MediaPlayerCommand::Add(args) => args.invoke(context).await,
            MediaPlayerCommand::New(args) => args.invoke(context).await,
            MediaPlayerCommand::Set(args) => args.invoke(context).await,
            MediaPlayerCommand::Update(args) => args.invoke(context).await,
            MediaPlayerCommand::Create(args) => args.invoke(context).await,
            MediaPlayerCommand::Show(args) => args.invoke(context).await,
            MediaPlayerCommand::Default(args) => args.invoke(context).await,
            MediaPlayerCommand::Detect(args) => args.invoke(context).await,
            MediaPlayerCommand::Discover(args) => args.invoke(context).await,
        }
    }
}

impl ToArgs for MediaPlayerArgs {
    fn to_args(&self) -> Vec<OsString> {
        let mut args = Vec::new();
        match &self.command {
            MediaPlayerCommand::List(list_args) => {
                args.push("list".into());
                args.extend(list_args.to_args());
            }
            MediaPlayerCommand::Add(add_args) => {
                args.push("add".into());
                args.extend(add_args.to_args());
            }
            MediaPlayerCommand::New(new_args) => {
                args.push("new".into());
                args.extend(new_args.to_args());
            }
            MediaPlayerCommand::Set(set_args) => {
                args.push("set".into());
                args.extend(set_args.to_args());
            }
            MediaPlayerCommand::Update(update_args) => {
                args.push("update".into());
                args.extend(update_args.to_args());
            }
            MediaPlayerCommand::Create(create_args) => {
                args.push("create".into());
                args.extend(create_args.to_args());
            }
            MediaPlayerCommand::Show(show_args) => {
                args.push("show".into());
                args.extend(show_args.to_args());
            }
            MediaPlayerCommand::Default(default_args) => {
                args.push("default".into());
                args.extend(default_args.to_args());
            }
            MediaPlayerCommand::Detect(detect_args) => {
                args.push("detect".into());
                args.extend(detect_args.to_args());
            }
            MediaPlayerCommand::Discover(detect_args) => {
                args.push("discover".into());
                args.extend(detect_args.to_args());
            }
        }
        args
    }
}
