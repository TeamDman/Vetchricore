use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::response::CliResponse;
use crate::cli::test::run::TestRunArgs;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;
use std::ffi::OsString;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct TestArgs {
    #[facet(args::subcommand)]
    pub command: TestCommand,
}

#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum TestCommand {
    Run(TestRunArgs),
}

impl TestArgs {
    /// # Errors
    ///
    /// Returns an error if the selected test subcommand fails.
    pub async fn invoke(self, context: &InvokeContext) -> Result<CliResponse> {
        match self.command {
            TestCommand::Run(args) => args.invoke(context).await,
        }
    }
}

impl ToArgs for TestArgs {
    fn to_args(&self) -> Vec<OsString> {
        let mut args = Vec::new();
        match &self.command {
            TestCommand::Run(run_args) => {
                args.push("run".into());
                args.extend(run_args.to_args());
            }
        }
        args
    }
}
