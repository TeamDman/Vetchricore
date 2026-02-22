use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::key::key_gen::KeyGenArgs;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;
use std::io::Write;

#[derive(Facet, Arbitrary, Debug, PartialEq, Default)]
pub struct KeyShowArgs {
    #[facet(args::named, default)]
    pub reveal: bool,
}

impl KeyShowArgs {
    pub async fn invoke(self, context: &InvokeContext) -> Result<()> {
        let profile_home = context.profile_home();

        let keypair = if let Some(keypair) = app_state::load_keypair(profile_home)? {
            keypair
        } else {
            let mut stdout = std::io::stdout();
            write!(
                stdout,
                "You have no key. Would you like to create one? Y/n: "
            )?;
            stdout.flush()?;
            let mut answer = String::new();
            std::io::stdin().read_line(&mut answer)?;
            if answer.trim().eq_ignore_ascii_case("n") {
                return Ok(());
            }
            KeyGenArgs.invoke(context).await?;
            app_state::load_keypair(profile_home)?
                .ok_or_else(|| eyre::eyre!("Key generation failed"))?
        };

        println!("Public key: {}", keypair.key());
        if self.reveal {
            println!("Private key: {}", keypair.secret());
        } else {
            println!("Private key: this value is hidden");
        }
        Ok(())
    }
}

impl ToArgs for KeyShowArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        if self.reveal {
            vec!["--reveal".into()]
        } else {
            Vec::new()
        }
    }
}
