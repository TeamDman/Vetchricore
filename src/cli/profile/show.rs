use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::profile::details::format_detailed_profile;
use crate::cli::response::CliResponse;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;
use std::fmt;

#[derive(Facet, Arbitrary, Debug, PartialEq, Default)]
pub struct ProfileShowArgs {
    #[facet(args::named, default)]
    pub detailed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct ProfileShowResponse {
    profile: String,
}

impl fmt::Display for ProfileShowResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "You are using {}.", self.profile)
    }
}

impl ProfileShowArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<CliResponse> {
        let profile_home = context.profile_home();
        if self.detailed {
            let active = app_state::current_active_profile(context.app_home())?;
            return Ok(format_detailed_profile(
                profile_home,
                profile_home.profile() == active,
            )?
            .into());
        } else {
            return Ok(ProfileShowResponse {
                profile: profile_home.profile().to_owned(),
            }
            .into());
        }
    }
}

impl ToArgs for ProfileShowArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        if self.detailed {
            vec!["--detailed".into()]
        } else {
            Vec::new()
        }
    }
}

