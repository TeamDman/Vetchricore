use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::media::player::catalog::canonical_media_player_key;
use arbitrary::Arbitrary;
use eyre::Result;
use eyre::WrapErr;
use facet::Facet;
use figue as args;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct MediaPlayerAddArgs {
    #[facet(args::positional)]
    pub key: String,

    #[facet(args::positional)]
    pub path: std::path::PathBuf,
}

impl MediaPlayerAddArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<()> {
        let key = canonical_media_player_key(&self.key);
        let canonical_path = std::fs::canonicalize(&self.path)
            .wrap_err_with(|| format!("failed to canonicalize '{}'", self.path.display()))?;

        app_state::upsert_media_player(context.profile_home(), &key, &canonical_path)?;
        println!("Configured media player '{key}' at {}", canonical_path.display());
        Ok(())
    }
}

impl ToArgs for MediaPlayerAddArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        vec![
            self.key.clone().into(),
            self.path.as_os_str().to_os_string().into(),
        ]
    }
}
