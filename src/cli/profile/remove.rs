use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;
use std::io;
use std::io::Write;
use std::fmt;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct ProfileRemoveArgs {
    #[facet(args::named, default)]
    pub yes: bool,

    #[facet(args::positional)]
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct ProfileRemoveResponse {
    message: String,
}

impl fmt::Display for ProfileRemoveResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl ProfileRemoveArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<ProfileRemoveResponse> {
        app_state::ensure_initialized(context.app_home())?;
        if !self.yes && !confirm_remove(&self.name)? {
            return Ok(ProfileRemoveResponse {
                message: "Aborted profile removal.".to_owned(),
            });
        }
        app_state::remove_profile(context.app_home(), &self.name)?;
        Ok(ProfileRemoveResponse {
            message: format!("{} has been destroyed.", self.name),
        })
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

