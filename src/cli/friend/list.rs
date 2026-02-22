use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;

#[derive(Facet, Arbitrary, Debug, PartialEq, Default)]
pub struct FriendListArgs;

impl FriendListArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<()> {
        let friends = app_state::list_friends(context.profile_home())?;
        if friends.is_empty() {
            println!("You have no friends. A new dawn awaits.");
            return Ok(());
        }

        for friend in friends {
            println!("{} ({})", friend.name, friend.pubkey);
        }
        Ok(())
    }
}

impl ToArgs for FriendListArgs {}
