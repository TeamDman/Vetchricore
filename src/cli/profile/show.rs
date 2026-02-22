use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::profile::details::print_detailed_profile;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;

#[derive(Facet, Arbitrary, Debug, PartialEq, Default)]
pub struct ProfileShowArgs {
    #[facet(args::named, default)]
    pub detailed: bool,
}

impl ProfileShowArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<()> {
        let profile_home = context.profile_home();
        if self.detailed {
            let active = app_state::current_active_profile(context.app_home())?;
            print_detailed_profile(profile_home, profile_home.profile() == active)?;
        } else {
            println!("You are using {}.", profile_home.profile());
        }
        Ok(())
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
