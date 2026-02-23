use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::media::player::default::set::MediaPlayerDefaultSetArgs;
use crate::cli::media::player::default::show::MediaPlayerDefaultShowArgs;
use crate::cli::response::CliResponse;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;
use std::ffi::OsString;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct MediaPlayerDefaultArgs {
    #[facet(args::subcommand)]
    pub command: MediaPlayerDefaultCommand,
}

#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum MediaPlayerDefaultCommand {
    Set(MediaPlayerDefaultSetArgs),
    Show(MediaPlayerDefaultShowArgs),
}

impl MediaPlayerDefaultArgs {
    /// # Errors
    ///
    /// Returns an error if the selected default subcommand fails.
    pub async fn invoke(self, context: &InvokeContext) -> Result<CliResponse> {
        match self.command {
            MediaPlayerDefaultCommand::Set(args) => args.invoke(context).await,
            MediaPlayerDefaultCommand::Show(args) => args.invoke(context).await,
        }
    }
}

impl ToArgs for MediaPlayerDefaultArgs {
    fn to_args(&self) -> Vec<OsString> {
        let mut args = Vec::new();
        match &self.command {
            MediaPlayerDefaultCommand::Set(set_args) => {
                args.push("set".into());
                args.extend(set_args.to_args());
            }
            MediaPlayerDefaultCommand::Show(show_args) => {
                args.push("show".into());
                args.extend(show_args.to_args());
            }
        }
        args
    }
}
