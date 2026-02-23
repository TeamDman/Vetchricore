use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::media::player::detect::now::MediaPlayerDetectNowArgs;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;
use std::ffi::OsString;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct MediaPlayerDetectArgs {
    #[facet(args::subcommand)]
    pub command: MediaPlayerDetectCommand,
}

#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum MediaPlayerDetectCommand {
    Now(MediaPlayerDetectNowArgs),
}

impl MediaPlayerDetectArgs {
    /// # Errors
    ///
    /// Returns an error if the selected detect subcommand fails.
    pub async fn invoke(self, context: &InvokeContext) -> Result<()> {
        match self.command {
            MediaPlayerDetectCommand::Now(args) => args.invoke(context).await?,
        }
        Ok(())
    }
}

impl ToArgs for MediaPlayerDetectArgs {
    fn to_args(&self) -> Vec<OsString> {
        let mut args = Vec::new();
        match &self.command {
            MediaPlayerDetectCommand::Now(now_args) => {
                args.push("now".into());
                args.extend(now_args.to_args());
            }
        }
        args
    }
}
