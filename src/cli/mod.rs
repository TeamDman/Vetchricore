pub mod app_state;
pub mod global_args;
pub mod key;
pub mod media;
pub mod known_user;
pub mod output_format;
pub mod profile;
pub mod route;
pub mod send;
pub mod test;
pub mod veilid_runtime;

use crate::cli::global_args::GlobalArgs;
use crate::cli::key::KeyArgs;
use crate::cli::media::MediaArgs;
use crate::cli::known_user::KnownUserArgs;
use crate::cli::output_format::OutputFormatArg;
use crate::cli::profile::ProfileArgs;
use crate::cli::route::RouteArgs;
use crate::cli::send::SendArgs;
use crate::cli::test::TestArgs;
use crate::paths::APP_HOME;
use crate::paths::AppHome;
use crate::paths::CACHE_DIR;
use crate::paths::CacheHome;
use arbitrary::Arbitrary;
use eyre::Context;
use facet::Facet;
use figue::FigueBuiltins;
use figue::{self as args};
use std::ffi::OsString;
use tracing::Instrument;

/// Trait for converting CLI structures to command line arguments.
pub trait ToArgs {
    /// Convert the CLI structure to command line arguments.
    fn to_args(&self) -> Vec<OsString> {
        Vec::new()
    }
}

#[derive(Clone, Debug)]
#[expect(
    clippy::struct_field_names,
    reason = "context fields are intentionally grouped by *_home semantics"
)]
pub struct InvokeContext {
    app_home: AppHome,
    cache_home: CacheHome,
    profile_home: app_state::ProfileHome,
    output_format: Option<OutputFormatArg>,
}

impl InvokeContext {
    /// # Errors
    ///
    /// Returns an error if profile/home resolution fails.
    pub fn resolve(global: &GlobalArgs) -> eyre::Result<Self> {
        let app_home = global
            .home_dir
            .as_ref()
            .map_or_else(|| APP_HOME.clone(), |path| AppHome(path.clone()));
        let cache_home = global
            .cache_dir
            .as_ref()
            .map_or_else(|| CACHE_DIR.clone(), |path| CacheHome(path.clone()));
        let profile_home = app_state::resolve_profile_home(&app_home, global.profile.as_deref())?;
        let output_format = global.output_format;

        Ok(Self {
            app_home,
            cache_home,
            profile_home,
            output_format,
        })
    }

    #[must_use]
    pub fn app_home(&self) -> &AppHome {
        &self.app_home
    }

    #[must_use]
    pub fn cache_home(&self) -> &CacheHome {
        &self.cache_home
    }

    #[must_use]
    pub fn profile_home(&self) -> &app_state::ProfileHome {
        &self.profile_home
    }

    #[must_use]
    pub fn output_format(&self) -> Option<OutputFormatArg> {
        self.output_format
    }
}

// Blanket implementation for references
impl<T: ToArgs> ToArgs for &T {
    fn to_args(&self) -> Vec<OsString> {
        (*self).to_args()
    }
}

/// A demonstration command line utility.
#[derive(Facet, Arbitrary, Debug)]
pub struct Cli {
    /// Global arguments (`debug`, `no_veilid_logs`, `log_filter`, `log_file`).
    #[facet(flatten)]
    pub global: GlobalArgs,

    /// Standard CLI options (help, version, completions).
    #[facet(flatten)]
    #[arbitrary(default)]
    pub builtins: FigueBuiltins,

    /// The command to run.
    #[facet(args::subcommand)]
    pub command: Command,
}

impl PartialEq for Cli {
    fn eq(&self, other: &Self) -> bool {
        // Ignore builtins in comparison since FigueBuiltins doesn't implement PartialEq
        self.global == other.global && self.command == other.command
    }
}

impl Cli {
    /// # Errors
    ///
    /// This function will return an error if the tokio runtime cannot be built or if the command fails.
    pub fn invoke(self) -> eyre::Result<()> {
        let context = InvokeContext::resolve(&self.global)?;
        let command_display = Self::display_invocation(&self.command);
        let profile = context.profile_home().profile().to_owned();
        let app_home = context.app_home().display().to_string();
        let cache_home = context.cache_home().display().to_string();
        let profile_home = context.profile_home().profile_dir().display().to_string();
        let span = tracing::info_span!(
            "invoke_command",
            command = %command_display,
            profile = %profile,
            app_home = %app_home,
            cache_home = %cache_home,
            profile_home = %profile_home,
        );
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .wrap_err("Failed to build tokio runtime")?;
        runtime.block_on(async move { self.command.invoke(&context).instrument(span).await })?;
        Ok(())
    }

    #[must_use]
    pub fn display_invocation(command: &impl ToArgs) -> String {
        command
            .to_args()
            .iter()
            .map(|arg| arg.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl ToArgs for Cli {
    fn to_args(&self) -> Vec<OsString> {
        let mut args = Vec::new();
        args.extend(self.global.to_args());
        args.extend(self.command.to_args());
        args
    }
}

/// A demonstration command line utility.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum Command {
    /// Profile management commands.
    Profile(ProfileArgs),
    /// Known-user management commands.
    KnownUser(KnownUserArgs),
    /// Key management commands.
    Key(KeyArgs),
    /// Route management commands.
    Route(RouteArgs),
    /// Media management commands.
    Media(MediaArgs),
    /// Sending commands.
    Send(SendArgs),
    /// Test utility commands.
    Test(TestArgs),
}

impl Command {
    /// # Errors
    ///
    /// This function will return an error if the subcommand fails.
    pub async fn invoke(self, context: &InvokeContext) -> eyre::Result<()> {
        match self {
            Command::Profile(args) => args.invoke(context).await,
            Command::KnownUser(args) => args.invoke(context).await,
            Command::Key(args) => args.invoke(context).await,
            Command::Route(args) => args.invoke(context).await,
            Command::Media(args) => args.invoke(context).await,
            Command::Send(args) => args.invoke(context).await,
            Command::Test(args) => args.invoke(context).await,
        }
    }
}

impl ToArgs for Command {
    fn to_args(&self) -> Vec<OsString> {
        let mut args = Vec::new();
        match self {
            Command::Profile(profile_args) => {
                args.push("profile".into());
                args.extend(profile_args.to_args());
            }
            Command::KnownUser(known_user_args) => {
                args.push("known-user".into());
                args.extend(known_user_args.to_args());
            }
            Command::Key(key_args) => {
                args.push("key".into());
                args.extend(key_args.to_args());
            }
            Command::Route(route_args) => {
                args.push("route".into());
                args.extend(route_args.to_args());
            }
            Command::Media(media_args) => {
                args.push("media".into());
                args.extend(media_args.to_args());
            }
            Command::Send(send_args) => {
                args.push("send".into());
                args.extend(send_args.to_args());
            }
            Command::Test(test_args) => {
                args.push("test".into());
                args.extend(test_args.to_args());
            }
        }
        args
    }
}
