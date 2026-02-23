use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::response::CliResponse;
use crate::cli::send::chat::SendChatArgs;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;
use std::ffi::OsString;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct SendArgs {
    #[facet(args::positional)]
    pub known_user: String,
    #[facet(args::subcommand)]
    pub command: SendCommand,
}

#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum SendCommand {
    Chat(SendChatArgs),
}

impl SendArgs {
    /// # Errors
    ///
    /// Returns an error if the selected send subcommand fails.
    pub async fn invoke(self, context: &InvokeContext) -> Result<CliResponse> {
        match self.command {
            SendCommand::Chat(args) => args.invoke(context, &self.known_user).await?,
        }
        Ok(CliResponse::empty())
    }
}

impl ToArgs for SendArgs {
    fn to_args(&self) -> Vec<OsString> {
        let mut args = vec![self.known_user.clone().into()];
        match &self.command {
            SendCommand::Chat(chat_args) => {
                args.push("chat".into());
                args.extend(chat_args.to_args());
            }
        }
        args
    }
}
