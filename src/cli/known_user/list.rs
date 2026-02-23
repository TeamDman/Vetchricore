use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::response::CliResponse;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use std::fmt;

#[derive(Facet, Arbitrary, Debug, PartialEq, Default)]
pub struct KnownUserListArgs;

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct KnownUserListItem {
    name: String,
    pubkey: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct KnownUserListResponse {
    known_users: Vec<KnownUserListItem>,
}

impl fmt::Display for KnownUserListResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.known_users.is_empty() {
            return f.write_str("You have no known users. A new dawn awaits.");
        }

        for (index, known_user) in self.known_users.iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
            }
            write!(f, "{} ({})", known_user.name, known_user.pubkey)?;
        }
        Ok(())
    }
}

impl KnownUserListArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<CliResponse> {
        let known_users = app_state::list_known_users(context.profile_home())?;
        let response = KnownUserListResponse {
            known_users: known_users
                .into_iter()
                .map(|known_user| KnownUserListItem {
                    name: known_user.name,
                    pubkey: known_user.pubkey.to_string(),
                })
                .collect(),
        };
        CliResponse::from_facet(response)
    }
}

impl ToArgs for KnownUserListArgs {}
