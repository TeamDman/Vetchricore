use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::key::key_gen::KeyGenArgs;
use crate::cli::key::remove::KeyRemoveArgs;
use crate::cli::key::show::KeyShowArgs;
use crate::cli::response::CliResponse;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;
use std::ffi::OsString;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct KeyArgs {
    #[facet(args::subcommand)]
    pub command: KeyCommand,
}

#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum KeyCommand {
    Gen(KeyGenArgs),
    Show(KeyShowArgs),
    Remove(KeyRemoveArgs),
}

impl KeyArgs {
    /// # Errors
    ///
    /// Returns an error if the selected key subcommand fails.
    pub async fn invoke(self, context: &InvokeContext) -> Result<CliResponse> {
        match self.command {
            KeyCommand::Gen(args) => args.invoke(context).await,
            KeyCommand::Show(args) => args.invoke(context).await,
            KeyCommand::Remove(args) => args.invoke(context).await,
        }
    }
}

impl ToArgs for KeyArgs {
    fn to_args(&self) -> Vec<OsString> {
        let mut args = Vec::new();
        match &self.command {
            KeyCommand::Gen(gen_args) => {
                args.push("gen".into());
                args.extend(gen_args.to_args());
            }
            KeyCommand::Show(show_args) => {
                args.push("show".into());
                args.extend(show_args.to_args());
            }
            KeyCommand::Remove(remove_args) => {
                args.push("remove".into());
                args.extend(remove_args.to_args());
            }
        }
        args
    }
}
