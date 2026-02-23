use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::response::CliResponse;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use std::fmt;
use std::io::Write;

#[derive(Facet, Arbitrary, Debug, PartialEq, Default)]
pub struct KeyRemoveArgs;

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct KeyRemoveResponse {
    message: String,
}

impl fmt::Display for KeyRemoveResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl KeyRemoveArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<CliResponse> {
        let mut stdout = std::io::stdout();
        write!(stdout, "Are you sure? y/N: ")?;
        stdout.flush()?;
        let mut answer = String::new();
        std::io::stdin().read_line(&mut answer)?;
        if !answer.trim().eq_ignore_ascii_case("y") {
            return CliResponse::from_facet(KeyRemoveResponse {
                message: "Aborted key removal.".to_owned(),
            });
        }

        write!(
            stdout,
            "Are you sure you're sure? Type \"Yes, I'm sure.\" to proceed: "
        )?;
        stdout.flush()?;
        answer.clear();
        std::io::stdin().read_line(&mut answer)?;
        if answer.trim() != "Yes, I'm sure." {
            return CliResponse::from_facet(KeyRemoveResponse {
                message: "Aborted key removal.".to_owned(),
            });
        }

        app_state::remove_keypair(context.profile_home())?;
        CliResponse::from_facet(KeyRemoveResponse {
            message: "Key has been removed.".to_owned(),
        })
    }
}

impl ToArgs for KeyRemoveArgs {}
