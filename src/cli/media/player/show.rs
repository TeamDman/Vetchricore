use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::media::player::catalog::canonical_media_player_key;
use crate::cli::media::player::catalog::display_name_for_key;
use crate::cli::media::player::catalog::support_for_key;
use arbitrary::Arbitrary;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use figue as args;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct MediaPlayerShowArgs {
    #[facet(args::positional)]
    pub key: String,
}

impl MediaPlayerShowArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<()> {
        let key = canonical_media_player_key(&self.key);
        let Some(configured) = app_state::media_player(context.profile_home(), &key)? else {
            bail!("Media player '{}' is not configured.", key);
        };
        let default_key = app_state::default_media_player(context.profile_home())?;

        println!("Name: {}", display_name_for_key(&key));
        println!("Key: {key}");
        println!(
            "Support: {}",
            if support_for_key(&key) {
                "supported"
            } else {
                "not supported"
            }
        );
        println!("Configured path: {}", configured.path.display());

        if default_key.as_deref() == Some(&key) {
            println!("Default: yes");
        } else {
            println!("Default: no");
        }

        Ok(())
    }
}

impl ToArgs for MediaPlayerShowArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        vec![self.key.clone().into()]
    }
}
