//! Global arguments that apply to all commands.

use crate::cli::ToArgs;
use crate::cli::output_format::OutputFormatArg;
use crate::logging::LoggingConfig;
use arbitrary::Arbitrary;
use chrono::Local;
use facet::Facet;
use figue::{self as args};
use std::ffi::OsString;
use std::path::PathBuf;
use std::str::FromStr;
use tracing::level_filters::LevelFilter;

/// Global arguments that apply to all commands.
#[derive(Facet, Arbitrary, Debug, Default, PartialEq)]
pub struct GlobalArgs {
    /// Profile override to use for this command invocation.
    #[facet(args::named)]
    pub profile: Option<String>,

    /// Override the app home directory for this command invocation.
    #[facet(args::named)]
    pub home_dir: Option<PathBuf>,

    /// Override the app cache directory for this command invocation.
    #[facet(args::named)]
    pub cache_dir: Option<PathBuf>,

    /// Enable debug logging, including backtraces on panics.
    #[facet(args::named, default)]
    pub debug: bool,

    /// Hide Veilid internal logs when present.
    #[facet(args::named, default)]
    pub no_veilid_logs: bool,

    /// Log level filter directive.
    #[facet(args::named)]
    pub log_filter: Option<String>,

    /// Write structured ndjson logs to this file or directory.
    #[facet(args::named)]
    pub log_file: Option<PathBuf>,

    /// Output format for command responses: auto, text, json.
    #[facet(args::named)]
    pub output_format: Option<OutputFormatArg>,
}

impl GlobalArgs {
    /// Get the logging configuration from CLI arguments.
    ///
    /// # Errors
    ///
    /// This function will return an error if the log filter string is invalid.
    pub fn logging_config(&self) -> eyre::Result<LoggingConfig> {
        let effective_level = match (self.debug, &self.log_filter) {
            (true, _) => LevelFilter::DEBUG,
            (false, Some(filter)) => LevelFilter::from_str(filter)?,
            (false, None) => LevelFilter::INFO,
        };

        Ok(LoggingConfig {
            default_directive: effective_level.into(),
            json_log_path: match &self.log_file {
                None => None,
                Some(path) if path.is_dir() => {
                    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
                    let filename = format!("log_{timestamp}.ndjson");
                    Some(path.join(filename))
                }
                Some(path) => Some(path.clone()),
            },
            show_veilid_internal_logs: !self.no_veilid_logs,
        })
    }
}

impl ToArgs for GlobalArgs {
    fn to_args(&self) -> Vec<OsString> {
        let mut args = Vec::new();
        if let Some(profile) = &self.profile {
            args.push("--profile".into());
            args.push(profile.into());
        }
        if let Some(path) = &self.home_dir {
            args.push("--home-dir".into());
            args.push(path.as_os_str().into());
        }
        if let Some(path) = &self.cache_dir {
            args.push("--cache-dir".into());
            args.push(path.as_os_str().into());
        }
        if self.debug {
            args.push("--debug".into());
        }
        if self.no_veilid_logs {
            args.push("--no-veilid-logs".into());
        }
        if let Some(filter) = &self.log_filter {
            args.push("--log-filter".into());
            args.push(filter.into());
        }
        if let Some(path) = &self.log_file {
            args.push("--log-file".into());
            args.push(path.as_os_str().into());
        }
        if let Some(output_format) = &self.output_format {
            args.push("--output-format".into());
            args.push(output_format.as_cli_token().into());
        }
        args
    }
}
