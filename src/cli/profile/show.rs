use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::profile::details::format_detailed_profile;
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
#[repr(u8)]
pub enum ProfileShowResponse {
    Summary { profile: String },
    Detailed { text: String },
}

impl fmt::Display for ProfileShowResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Summary { profile } => write!(f, "You are using {}.", profile),
            Self::Detailed { text } => f.write_str(text),
        }
    }
}

impl ProfileShowArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<ProfileShowResponse> {
        let profile_home = context.profile_home();
        if self.detailed {
            let active = app_state::current_active_profile(context.app_home())?;
            return Ok(ProfileShowResponse::Detailed {
                text: format_detailed_profile(
                profile_home,
                profile_home.profile() == active,
            )?,
            });
        } else {
            return Ok(ProfileShowResponse::Summary {
                profile: profile_home.profile().to_owned(),
            });
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

