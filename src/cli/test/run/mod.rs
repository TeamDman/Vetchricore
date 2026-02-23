pub mod e2e_chat;

use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;
use std::ffi::OsString;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct TestRunArgs {
    #[facet(args::subcommand)]
    pub command: TestRunCommand,
}

#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum TestRunCommand {
    E2eChat(e2e_chat::E2eChatArgs),
}

impl TestRunArgs {
    /// # Errors
    ///
    /// Returns an error if the selected test run scenario fails.
    pub async fn invoke(self, context: &InvokeContext) -> Result<()> {
        match self.command {
            TestRunCommand::E2eChat(args) => args.invoke(context).await?,
        }
        Ok(())
    }
}

impl ToArgs for TestRunArgs {
    fn to_args(&self) -> Vec<OsString> {
        let mut args = Vec::new();
        match &self.command {
            TestRunCommand::E2eChat(e2e_chat_args) => {
                args.push("e2e-chat".into());
                args.extend(e2e_chat_args.to_args());
            }
        }
        args
    }
}
