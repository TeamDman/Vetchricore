use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::global_args::GlobalArgs;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct ProfileRemoveArgs {
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
        app_state::remove_profile(&self.name)?;
        println!("{} has been destroyed.", self.name);
        Ok(())
    }
}

impl ToArgs for ProfileRemoveArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        vec![self.name.clone().into()]
    }
}
