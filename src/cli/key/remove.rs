use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use std::io::Write;

#[derive(Facet, Arbitrary, Debug, PartialEq, Default)]
pub struct KeyRemoveArgs;

impl KeyRemoveArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<()> {
        let mut stdout = std::io::stdout();
        write!(stdout, "Are you sure? y/N: ")?;
        stdout.flush()?;
        let mut answer = String::new();
        std::io::stdin().read_line(&mut answer)?;
        if !answer.trim().eq_ignore_ascii_case("y") {
            return Ok(());
        }

        write!(
            stdout,
            "Are you sure you're sure? Type \"Yes, I'm sure.\" to proceed: "
        )?;
        stdout.flush()?;
        answer.clear();
        std::io::stdin().read_line(&mut answer)?;
        if answer.trim() != "Yes, I'm sure." {
            return Ok(());
        }

        app_state::remove_keypair(context.profile_home())?;
        println!("Key has been removed.");

        Ok(())
    }
}

impl ToArgs for KeyRemoveArgs {}
