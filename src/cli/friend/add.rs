use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::global_args::GlobalArgs;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;
use veilid_core::PublicKey;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct FriendAddArgs {
    #[facet(args::positional)]
    pub name: String,
    #[facet(args::positional)]
    pub pubkey: String,
}

impl FriendAddArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, global: &GlobalArgs) -> Result<()> {
        let profile = app_state::resolve_profile(global)?;
        let pubkey = self.pubkey.parse::<PublicKey>()?;
        app_state::add_friend(&profile, &self.name, pubkey)?;
        println!("You have added {} as a friend.", self.name);
        Ok(())
    }
}

impl ToArgs for FriendAddArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        vec![self.name.clone().into(), self.pubkey.clone().into()]
    }
}
