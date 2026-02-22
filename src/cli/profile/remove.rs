use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::global_args::GlobalArgs;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;
use std::io;
use std::io::Write;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct ProfileRemoveArgs {
    #[facet(args::named, default)]
    pub yes: bool,

    #[facet(args::positional)]
    pub name: String,
}

impl ProfileRemoveArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, _global: &GlobalArgs) -> Result<()> {
        app_state::ensure_initialized()?;
        if !self.yes && !confirm_remove(&self.name)? {
            println!("Aborted profile removal.");
            return Ok(());
        }
        app_state::remove_profile(&self.name)?;
        println!("{} has been destroyed.", self.name);
        Ok(())
    }
}

fn confirm_remove(name: &str) -> Result<bool> {
    print!("Remove profile '{name}'? [y/N]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let response = input.trim().to_ascii_lowercase();
    Ok(matches!(response.as_str(), "y" | "yes"))
}

impl ToArgs for ProfileRemoveArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        let mut args = Vec::new();
        if self.yes {
            args.push("--yes".into());
        }
        args.push(self.name.clone().into());
        args
    }
}
