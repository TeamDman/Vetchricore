use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::media::player::catalog::canonical_media_player_key;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct MediaPlayerDefaultSetArgs {
    #[facet(args::positional)]
    pub key: String,
}

impl MediaPlayerDefaultSetArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<()> {
        let key = canonical_media_player_key(&self.key);
        app_state::set_default_media_player(context.profile_home(), &key)?;
        println!("Default media player set to '{key}'.");
        Ok(())
    }
}

impl ToArgs for MediaPlayerDefaultSetArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        vec![self.key.clone().into()]
    }
}
