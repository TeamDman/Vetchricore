use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::global_args::GlobalArgs;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;

#[derive(Facet, Arbitrary, Debug, PartialEq, Default)]
pub struct ProfileShowArgs;

impl ProfileShowArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, global: &GlobalArgs) -> Result<()> {
        let profile = app_state::resolve_profile(global)?;
        println!("You are using {profile}.");
        Ok(())
    }
}

impl ToArgs for ProfileShowArgs {}
