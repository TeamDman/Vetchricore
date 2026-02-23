use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::key::key_gen::KeyGenArgs;
use crate::cli::response::CliResponse;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;
use std::fmt;
use std::io::Write;

#[derive(Facet, Arbitrary, Debug, PartialEq, Default)]
pub struct KeyShowArgs {
    #[facet(args::named, default)]
    pub reveal: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct KeyShowResponse {
    public_key: String,
    private_key: String,
}

impl fmt::Display for KeyShowResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Public key: {}", self.public_key)?;
        write!(f, "Private key: {}", self.private_key)
    }
}

impl KeyShowArgs {
    pub async fn invoke(self, context: &InvokeContext) -> Result<CliResponse> {
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
                return Ok(CliResponse::empty());
            }
            let _ = KeyGenArgs.invoke(context).await?;
            app_state::load_keypair(profile_home)?
                .ok_or_else(|| eyre::eyre!("Key generation failed"))?
        };

        CliResponse::from_facet(KeyShowResponse {
            public_key: keypair.key().to_string(),
            private_key: if self.reveal {
                keypair.secret().to_string()
            } else {
                "this value is hidden".to_owned()
            },
        })
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
