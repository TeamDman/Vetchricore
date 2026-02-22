use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::profile::details::print_detailed_profile;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;

#[derive(Facet, Arbitrary, Debug, PartialEq, Default)]
pub struct ProfileListArgs {
    #[facet(args::named, default)]
    pub detailed: bool,
}

impl ProfileListArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<()> {
        app_state::ensure_initialized(context.app_home())?;
        let active = app_state::current_active_profile(context.app_home())?;
        for profile in app_state::list_profiles(context.app_home())? {
            let profile_home = app_state::profile_home(context.app_home(), &profile)?;
            if self.detailed {
                print_detailed_profile(&profile_home, profile == active)?;
                println!();
            } else if profile == active {
                println!("{profile} (active)");
            } else {
                println!("{profile}");
            }
        }
        Ok(())
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
