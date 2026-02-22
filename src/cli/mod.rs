pub mod app_state;
pub mod friend;
pub mod global_args;
pub mod key;
pub mod profile;
pub mod route;
pub mod send;
pub mod veilid_runtime;

use crate::cli::friend::FriendArgs;
use crate::cli::global_args::GlobalArgs;
use crate::cli::key::KeyArgs;
use crate::cli::profile::ProfileArgs;
use crate::cli::route::RouteArgs;
use crate::cli::send::SendArgs;
use arbitrary::Arbitrary;
use eyre::Context;
use facet::Facet;
use figue::FigueBuiltins;
use figue::{self as args};
use std::ffi::OsString;

/// Trait for converting CLI structures to command line arguments.
pub trait ToArgs {
    /// Convert the CLI structure to command line arguments.
    fn to_args(&self) -> Vec<OsString> {
        Vec::new()
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
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .wrap_err("Failed to build tokio runtime")?;
        runtime.block_on(async move { self.command.invoke(&self.global).await })?;
        Ok(())
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
    /// Friend management commands.
    Friend(FriendArgs),
    /// Key management commands.
    Key(KeyArgs),
    /// Route management commands.
    Route(RouteArgs),
    /// Sending commands.
    Send(SendArgs),
}

impl Command {
    /// # Errors
    ///
    /// This function will return an error if the subcommand fails.
    pub async fn invoke(self, global: &GlobalArgs) -> eyre::Result<()> {
        match self {
            Command::Profile(args) => args.invoke(global).await,
            Command::Friend(args) => args.invoke(global).await,
            Command::Key(args) => args.invoke(global).await,
            Command::Route(args) => args.invoke(global).await,
            Command::Send(args) => args.invoke(global).await,
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
            Command::Friend(friend_args) => {
                args.push("friend".into());
                args.extend(friend_args.to_args());
            }
            Command::Key(key_args) => {
                args.push("key".into());
                args.extend(key_args.to_args());
            }
            Command::Route(route_args) => {
                args.push("route".into());
                args.extend(route_args.to_args());
            }
            Command::Send(send_args) => {
                args.push("send".into());
                args.extend(send_args.to_args());
            }
        }
        args
    }
}
