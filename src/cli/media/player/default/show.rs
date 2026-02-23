use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::media::player::catalog::display_name_for_key;
use arbitrary::Arbitrary;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::fmt;

#[derive(Facet, Arbitrary, Debug, PartialEq, Default)]
pub struct MediaPlayerDefaultShowArgs;

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct MediaPlayerDefaultShowResponse {
    name: String,
    key: String,
    configured_path: Option<String>,
}

impl fmt::Display for MediaPlayerDefaultShowResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(path) = &self.configured_path {
            write!(
                f,
                "Default media player: {} ({}) {}",
                self.name, self.key, path
            )
        } else {
            write!(f, "Default media player: {} ({})", self.name, self.key)
        }
    }
}

impl MediaPlayerDefaultShowArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<MediaPlayerDefaultShowResponse> {
        let Some(key) = app_state::default_media_player(context.profile_home())? else {
            bail!("No default media player is set.");
        };

        let configured_path =
            app_state::media_player(context.profile_home(), &key)?.map(|player| player.path);

        Ok(MediaPlayerDefaultShowResponse {
            name: display_name_for_key(&key),
            key,
            configured_path: configured_path.map(|path| path.display().to_string()),
        })
    }
}

impl ToArgs for MediaPlayerDefaultShowArgs {}
