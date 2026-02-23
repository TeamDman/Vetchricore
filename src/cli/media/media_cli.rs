use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::media::player::MediaPlayerArgs;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;
use std::ffi::OsString;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct MediaArgs {
    #[facet(args::subcommand)]
    pub command: MediaCommand,
}

#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum MediaCommand {
    Player(MediaPlayerArgs),
}

impl MediaArgs {
    /// # Errors
    ///
    /// Returns an error if the selected media subcommand fails.
    pub async fn invoke(self, context: &InvokeContext) -> Result<()> {
        match self.command {
            MediaCommand::Player(args) => args.invoke(context).await?,
        }
        Ok(())
    }
}

impl ToArgs for MediaArgs {
    fn to_args(&self) -> Vec<OsString> {
        let mut args = Vec::new();
        match &self.command {
            MediaCommand::Player(player_args) => {
                args.push("player".into());
                args.extend(player_args.to_args());
            }
        }
        args
    }
}
