use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::media::player::catalog::canonical_media_player_key;
use arbitrary::Arbitrary;
use eyre::Result;
use eyre::WrapErr;
use facet::Facet;
use figue as args;
use std::fmt;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct MediaPlayerAddArgs {
    #[facet(args::positional)]
    pub key: String,

    #[facet(args::positional)]
    pub path: std::path::PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct MediaPlayerAddResponse {
    key: String,
    configured_path: String,
}

impl fmt::Display for MediaPlayerAddResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Configured media player '{}' at {}",
            self.key, self.configured_path
        )
    }
}

impl MediaPlayerAddArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<MediaPlayerAddResponse> {
        let key = canonical_media_player_key(&self.key);
        let canonical_path = std::fs::canonicalize(&self.path)
            .wrap_err_with(|| format!("failed to canonicalize '{}'", self.path.display()))?;

        app_state::upsert_media_player(context.profile_home(), &key, &canonical_path)?;
        Ok(MediaPlayerAddResponse {
            key,
            configured_path: canonical_path.display().to_string(),
        })
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

