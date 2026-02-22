use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::global_args::GlobalArgs;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct FriendRenameArgs {
    #[facet(args::positional)]
    pub old_name: String,
    #[facet(args::positional)]
    pub new_name: String,
}

impl FriendRenameArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, global: &GlobalArgs) -> Result<()> {
        let profile = app_state::resolve_profile(global)?;
        app_state::rename_friend(&profile, &self.old_name, &self.new_name)?;
        println!("{} has been renamed to {}.", self.old_name, self.new_name);
        Ok(())
    }
}

impl ToArgs for FriendRenameArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        vec![self.old_name.clone().into(), self.new_name.clone().into()]
    }
}
