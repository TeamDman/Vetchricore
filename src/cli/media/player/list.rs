use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::media::player::catalog::display_name_for_key;
use crate::cli::media::player::catalog::support_for_key;
use crate::cli::output_format::OutputFormat;
use crate::cli::output_format::OutputFormatArg;
use arbitrary::Arbitrary;
use color_eyre::owo_colors::OwoColorize;
use eyre::Result;
use facet::Facet;
use std::io::IsTerminal;
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

impl MediaPlayerListArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<()> {
        let output_format = context
            .output_format()
            .unwrap_or(OutputFormatArg::Auto)
            .resolve();
        debug!(
            output_format = ?output_format,
            "listing media players"
        );

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

        match output_format {
            OutputFormat::Text => print_text(&views),
            OutputFormat::Json => {
                let is_terminal = std::io::stdout().is_terminal();
                let json = if is_terminal {
                    facet_json::to_string_pretty(&views)?
                } else {
                    facet_json::to_string(&views)?
                };
                println!("{json}");
            }
        }

        Ok(())
    }
}

fn print_text(views: &[MediaPlayerView]) {
    if views.is_empty() {
        println!("No configured media players found.");
        return;
    }

    for view in views {
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

        println!(
            "{} ({status}) {}{default_marker}",
            view.name,
            view.configured_path.bright_black()
        );
    }
}

impl ToArgs for MediaPlayerListArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> { Vec::new() }
}
