use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::media::player::catalog::display_name_for_key;
use crate::cli::media::player::catalog::support_for_key;
use crate::cli::response::CliResponse;
use arbitrary::Arbitrary;
use color_eyre::owo_colors::OwoColorize;
use eyre::Result;
use facet::Facet;
use std::fmt;
use tracing::debug;

#[derive(Facet, Arbitrary, Debug, PartialEq, Default)]
pub struct MediaPlayerListArgs;

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
struct MediaPlayerView {
    key: String,
    name: String,
    supported: bool,
    is_default: bool,
    configured_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct MediaPlayerListResponse {
    views: Vec<MediaPlayerView>,
}

impl fmt::Display for MediaPlayerListResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.views.is_empty() {
            return f.write_str("No configured media players found.");
        }

        for (index, view) in self.views.iter().enumerate() {
            let status = if view.supported {
                "supported".green().to_string()
            } else {
                "not supported".yellow().to_string()
            };
            let default_marker = if view.is_default {
                format!(" {}", "(default)".cyan())
            } else {
                String::new()
            };

            if index > 0 {
                writeln!(f)?;
            }

            write!(
                f,
                "{} ({status}) {}{default_marker}",
                view.name,
                view.configured_path.bright_black()
            )?;
        }

        Ok(())
    }
}

impl MediaPlayerListArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<CliResponse> {
        debug!("listing media players");

        let configured = app_state::list_media_players(context.profile_home())?;
        let default_key = app_state::default_media_player(context.profile_home())?;
        debug!(
            configured_count = configured.len(),
            has_default = default_key.is_some(),
            "loaded media player preferences"
        );

        let mut views = configured
            .into_iter()
            .map(|player| {
                let key = player.key;
                MediaPlayerView {
                    key: key.clone(),
                    name: display_name_for_key(&key),
                    supported: support_for_key(&key),
                    is_default: false,
                    configured_path: player.path.display().to_string(),
                }
            })
            .collect::<Vec<_>>();

        if let Some(default_key) = default_key {
            for view in &mut views {
                if view.key == default_key {
                    view.is_default = true;
                }
            }
        }

        views.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.key.cmp(&b.key)));
        debug!(
            media_player_count = views.len(),
            "prepared media player list output"
        );

        Ok(MediaPlayerListResponse { views }.into())
    }
}

impl ToArgs for MediaPlayerListArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> { Vec::new() }
}

