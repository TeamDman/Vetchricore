use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;

#[derive(Facet, Arbitrary, Debug, PartialEq, Default)]
pub struct KnownUserListArgs;

impl KnownUserListArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<()> {
        let known_users = app_state::list_known_users(context.profile_home())?;
        if known_users.is_empty() {
            println!("You have no known users. A new dawn awaits.");
            return Ok(());
        }

        for known_user in known_users {
            println!("{} ({})", known_user.name, known_user.pubkey);
        }
        Ok(())
    }
}

impl ToArgs for KnownUserListArgs {}
