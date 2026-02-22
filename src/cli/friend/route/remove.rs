use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::global_args::GlobalArgs;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;
use std::io::Write;
use veilid_core::RecordKey;

#[derive(Facet, Arbitrary, Debug, PartialEq, Default)]
pub struct FriendRouteRemoveArgs {
    #[facet(args::named)]
    pub friend: Option<String>,
    #[facet(args::named)]
    pub record_id: Option<String>,
}

impl FriendRouteRemoveArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, global: &GlobalArgs) -> Result<()> {
        let profile = app_state::resolve_profile(global)?;
        let record_key = self
            .record_id
            .as_ref()
            .map(|value| value.parse::<RecordKey>())
            .transpose()?;

        let matches = app_state::list_friend_route_keys(&profile, self.friend.as_deref())?
            .into_iter()
            .filter(|entry| match &record_key {
                Some(target_key) => entry.record_key == *target_key,
                None => true,
            })
            .collect::<Vec<_>>();

        if matches.is_empty() {
            println!("No matching friend routes found.");
            return Ok(());
        }

        if matches.len() > 1 {
            let mut stdout = std::io::stdout();
            write!(
                stdout,
                "This will remove {} routes. Continue? y/N: ",
                matches.len()
            )?;
            stdout.flush()?;
            let mut answer = String::new();
            std::io::stdin().read_line(&mut answer)?;
            if !answer.trim().eq_ignore_ascii_case("y") {
                return Ok(());
            }
        }

        let removed = app_state::remove_friend_route_keys(
            &profile,
            self.friend.as_deref(),
            record_key.as_ref(),
        )?;
        println!("Removed {removed} friend route(s).");
        Ok(())
    }
}

impl ToArgs for FriendRouteRemoveArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        let mut args = Vec::new();
        if let Some(friend) = &self.friend {
            args.push("--friend".into());
            args.push(friend.clone().into());
        }
        if let Some(record_id) = &self.record_id {
            args.push("--record-id".into());
            args.push(record_id.clone().into());
        }
        args
    }
}
