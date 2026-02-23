use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::media::player::catalog::display_name_for_key;
use arbitrary::Arbitrary;
use eyre::Result;
use eyre::bail;
use facet::Facet;

#[derive(Facet, Arbitrary, Debug, PartialEq, Default)]
pub struct MediaPlayerDefaultShowArgs;

impl MediaPlayerDefaultShowArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<()> {
        let Some(key) = app_state::default_media_player(context.profile_home())? else {
            bail!("No default media player is set.");
        };

        if let Some(player) = app_state::media_player(context.profile_home(), &key)? {
            println!(
                "Default media player: {} ({}) {}",
                display_name_for_key(&key),
                key,
                player.path.display()
            );
        } else {
            println!("Default media player: {} ({})", display_name_for_key(&key), key);
        }
        Ok(())
    }
}

impl ToArgs for MediaPlayerDefaultShowArgs {}
