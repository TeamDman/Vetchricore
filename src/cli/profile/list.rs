use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::global_args::GlobalArgs;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;

#[derive(Facet, Arbitrary, Debug, PartialEq, Default)]
pub struct ProfileListArgs;

impl ProfileListArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, _global: &GlobalArgs) -> Result<()> {
        app_state::ensure_initialized()?;
        let active = app_state::current_active_profile()?;
        for profile in app_state::list_profiles()? {
            if profile == active {
                println!("{profile} (active)");
            } else {
                println!("{profile}");
            }
        }
        Ok(())
    }
}

impl ToArgs for ProfileListArgs {}
