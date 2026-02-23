use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::response::CliResponse;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;
use std::fmt;
use std::io::Write;
use veilid_core::RecordKey;

#[derive(Facet, Arbitrary, Debug, PartialEq, Default)]
pub struct KnownUserRouteRemoveArgs {
    #[facet(args::named)]
    pub known_user: Option<String>,
    #[facet(args::named)]
    pub record_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct KnownUserRouteRemoveResponse {
    message: String,
}

impl fmt::Display for KnownUserRouteRemoveResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl KnownUserRouteRemoveArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<CliResponse> {
        let record_key = self
            .record_id
            .as_ref()
            .map(|value| value.parse::<RecordKey>())
            .transpose()?;

        let matches = app_state::list_known_user_route_keys(
            context.profile_home(),
            self.known_user.as_deref(),
        )?
        .into_iter()
        .filter(|entry| match &record_key {
            Some(target_key) => entry.record_key == *target_key,
            None => true,
        })
        .collect::<Vec<_>>();

        if matches.is_empty() {
            return Ok(KnownUserRouteRemoveResponse {
                message: "No matching known-user routes found.".to_owned(),
            }
            .into());
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
                return Ok(KnownUserRouteRemoveResponse {
                    message: "Aborted known-user route removal.".to_owned(),
                }
                .into());
            }
        }

        let removed = app_state::remove_known_user_route_keys(
            context.profile_home(),
            self.known_user.as_deref(),
            record_key.as_ref(),
        )?;
        Ok(KnownUserRouteRemoveResponse {
            message: format!("Removed {removed} known-user route(s)."),
        }
        .into())
    }
}

impl ToArgs for KnownUserRouteRemoveArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        let mut args = Vec::new();
        if let Some(known_user) = &self.known_user {
            args.push("--known-user".into());
            args.push(known_user.clone().into());
        }
        if let Some(record_id) = &self.record_id {
            args.push("--record-id".into());
            args.push(record_id.clone().into());
        }
        args
    }
}

