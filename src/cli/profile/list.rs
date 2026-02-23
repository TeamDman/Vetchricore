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
pub struct ProfileListArgs {
    #[facet(args::named, default)]
    pub detailed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct ProfileListItem {
    name: String,
    active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
#[repr(u8)]
pub enum ProfileListResponse {
    Compact { profiles: Vec<ProfileListItem> },
    Detailed { text: String },
}

impl fmt::Display for ProfileListResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Compact { profiles } => {
                for (index, profile) in profiles.iter().enumerate() {
                    if index > 0 {
                        writeln!(f)?;
                    }
                    if profile.active {
                        write!(f, "{} (active)", profile.name)?;
                    } else {
                        write!(f, "{}", profile.name)?;
                    }
                }
                Ok(())
            }
            Self::Detailed { text } => f.write_str(text),
        }
    }
}

impl ProfileListArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<ProfileListResponse> {
        app_state::ensure_initialized(context.app_home())?;
        let active = app_state::current_active_profile(context.app_home())?;
        let profiles = app_state::list_profiles(context.app_home())?;

        if self.detailed {
            let mut blocks = Vec::new();
            for profile in profiles {
                let profile_home = app_state::profile_home(context.app_home(), &profile)?;
                blocks.push(format_detailed_profile(&profile_home, profile == active)?);
            }
            return Ok(ProfileListResponse::Detailed {
                text: blocks.join("\n\n"),
            });
        }

        let response = ProfileListResponse::Compact {
            profiles: profiles
                .into_iter()
                .map(|name| ProfileListItem {
                    active: name == active,
                    name,
                })
                .collect(),
        };

        Ok(response)
    }
}

impl ToArgs for ProfileListArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        if self.detailed {
            vec!["--detailed".into()]
        } else {
            Vec::new()
        }
    }
}
