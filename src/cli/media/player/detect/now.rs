use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::media::player::catalog::detect_media_players_by_walk;
use crate::cli::media::player::catalog::detect_media_players_on_path;
use crate::cli::media::player::catalog::display_name_for_key;
use crate::cli::media::player::catalog::support_for_key;
use crate::cli::media::player::output_format::OutputFormatArg;
use crate::cli::media::player::output_format::OutputFormat;
use arbitrary::Arbitrary;
use color_eyre::owo_colors::OwoColorize;
use eyre::Result;
use facet::Facet;
use figue as args;
use std::collections::BTreeSet;
use std::io::IsTerminal;
use std::io::Write;
use tracing::debug;

#[derive(Facet, Arbitrary, Debug, PartialEq, Default)]
pub struct MediaPlayerDetectNowArgs {
    /// Output format: auto, text, json.
    #[facet(args::named)]
    pub output_format: Option<String>,

    /// Recursively walk filesystem for media players: yes|no|true|false|ask.
    #[facet(args::named)]
    pub walk: Option<String>,

    /// Timeout for filesystem walk (e.g. 25s, 2m).
    #[facet(args::named)]
    pub walk_timeout: Option<String>,

    /// Optional walk roots; use ';' or ',' as separators.
    #[facet(args::named)]
    pub walk_roots: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
struct DetectOutputItem {
    key: String,
    name: String,
    supported: bool,
    path: String,
    newly_added: bool,
}

impl MediaPlayerDetectNowArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<()> {
        let output_format = self
            .output_format
            .as_deref()
            .unwrap_or("auto")
            .parse::<OutputFormatArg>()?
            .resolve();
        let should_walk = self.should_walk()?;
        let walk_timeout = self.walk_timeout_duration()?;
        let walk_roots = self.walk_roots()?;
        debug!(
            output_format = ?output_format,
            should_walk,
            walk_timeout_ms = walk_timeout.as_millis(),
            walk_roots_count = walk_roots.len(),
            "running media player detection"
        );

        let configured = app_state::list_media_players(context.profile_home())?;
        let existing = configured
            .into_iter()
            .map(|entry| (entry.key, entry.path))
            .collect::<BTreeSet<_>>();

        let mut dedup = BTreeSet::<(String, std::path::PathBuf)>::new();
        for player in detect_media_players_on_path()? {
            dedup.insert((player.key, player.path));
        }
        debug!(
            detected_after_path = dedup.len(),
            "completed PATH media player detection"
        );
        if should_walk {
            for player in detect_media_players_by_walk(walk_timeout, &walk_roots).await? {
                dedup.insert((player.key, player.path));
            }
            debug!(
                detected_after_walk = dedup.len(),
                "completed filesystem walk media player detection"
            );
        }

        let mut output = Vec::new();

        for (key, path) in dedup {
            let was_existing = existing.contains(&(key.clone(), path.clone()));
            app_state::upsert_media_player(context.profile_home(), &key, &path)?;

            output.push(DetectOutputItem {
                key: key.clone(),
                name: display_name_for_key(&key),
                supported: support_for_key(&key),
                path: path.display().to_string(),
                newly_added: !was_existing,
            });
        }
        debug!(
            output_count = output.len(),
            "prepared media player detection output"
        );

        output.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.path.cmp(&b.path)));

        match output_format {
            OutputFormat::Text => {
                if output.is_empty() {
                    println!("No media players were detected on PATH.");
                } else {
                    for item in &output {
                        let status = if item.supported {
                            "supported".green().to_string()
                        } else {
                            "not supported".yellow().to_string()
                        };
                        let marker = if item.newly_added {
                            "(added)".cyan().to_string()
                        } else {
                            "(already configured)".bright_black().to_string()
                        };
                        println!(
                            "{} ({status}) {} {}",
                            item.name,
                            item.path.bright_black(),
                            marker
                        );
                    }
                }
            }
            OutputFormat::Json => {
                let json = if std::io::stdout().is_terminal() {
                    facet_json::to_string_pretty(&output)?
                } else {
                    facet_json::to_string(&output)?
                };
                println!("{json}");
            }
        }
        Ok(())
    }

    fn should_walk(&self) -> Result<bool> {
        match self
            .walk
            .as_deref()
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            None | Some("ask") => {
                if !std::io::stdout().is_terminal() {
                    return Ok(false);
                }
                let mut stdout = std::io::stdout();
                write!(
                    stdout,
                    "Walk filesystem recursively for media players? [y/N]: "
                )?;
                stdout.flush()?;
                let mut answer = String::new();
                std::io::stdin().read_line(&mut answer)?;
                Ok(matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes"))
            }
            Some("yes") | Some("true") => Ok(true),
            Some("no") | Some("false") => Ok(false),
            Some(other) => eyre::bail!(
                "Unsupported --walk value '{}'. Use yes, no, true, false, or ask.",
                other
            ),
        }
    }

    fn walk_timeout_duration(&self) -> Result<std::time::Duration> {
        let timeout = self.walk_timeout.as_deref().unwrap_or("25s");
        humantime::parse_duration(timeout)
            .map_err(|err| eyre::eyre!("Invalid --walk-timeout value '{}': {}", timeout, err))
    }

    fn walk_roots(&self) -> Result<Vec<std::path::PathBuf>> {
        let mut roots = Vec::new();
        let Some(raw_roots) = &self.walk_roots else {
            return Ok(roots);
        };

        for value in raw_roots.split([';', ',']) {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                continue;
            }
            let root = std::path::PathBuf::from(trimmed);
            if !root.exists() {
                eyre::bail!("Walk root '{}' does not exist.", root.display());
            }
            roots.push(root);
        }

        Ok(roots)
    }
}

impl ToArgs for MediaPlayerDetectNowArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        let mut args = Vec::new();
        if let Some(output_format) = &self.output_format {
            args.push("--output-format".into());
            args.push(output_format.into());
        }
        if let Some(walk) = &self.walk {
            args.push("--walk".into());
            args.push(walk.into());
        }
        if let Some(walk_timeout) = &self.walk_timeout {
            args.push("--walk-timeout".into());
            args.push(walk_timeout.into());
        }
        if let Some(walk_roots) = &self.walk_roots {
            args.push("--walk-roots".into());
            args.push(walk_roots.into());
        }
        args
    }
}
