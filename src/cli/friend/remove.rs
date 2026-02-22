use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::global_args::GlobalArgs;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct FriendRemoveArgs {
    #[facet(args::positional)]
    pub name: String,
}

impl FriendRemoveArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, global: &GlobalArgs) -> Result<()> {
        let profile = app_state::resolve_profile(global)?;
        app_state::remove_friend(&profile, &self.name)?;
        println!("{} has been unfriended.", self.name);
        Ok(())
    }
}

impl ToArgs for FriendRemoveArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        vec![self.name.clone().into()]
    }
}
