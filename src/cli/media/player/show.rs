use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::app_state;
use crate::cli::media::player::catalog::canonical_media_player_key;
use crate::cli::media::player::catalog::display_name_for_key;
use crate::cli::media::player::catalog::support_for_key;
use arbitrary::Arbitrary;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use figue as args;
use std::fmt;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct MediaPlayerShowArgs {
    #[facet(args::positional)]
    pub key: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct MediaPlayerShowResponse {
    name: String,
    key: String,
    support: String,
    configured_path: String,
    is_default: bool,
}

impl fmt::Display for MediaPlayerShowResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Name: {}", self.name)?;
        writeln!(f, "Key: {}", self.key)?;
        writeln!(f, "Support: {}", self.support)?;
        writeln!(f, "Configured path: {}", self.configured_path)?;
        write!(f, "Default: {}", if self.is_default { "yes" } else { "no" })
    }
}

impl MediaPlayerShowArgs {
    #[expect(
        clippy::unused_async,
        reason = "command handlers use async invoke signature consistently"
    )]
    pub async fn invoke(self, context: &InvokeContext) -> Result<MediaPlayerShowResponse> {
        let key = canonical_media_player_key(&self.key);
        let Some(configured) = app_state::media_player(context.profile_home(), &key)? else {
            bail!("Media player '{}' is not configured.", key);
        };
        let default_key = app_state::default_media_player(context.profile_home())?;

        Ok(MediaPlayerShowResponse {
            name: display_name_for_key(&key),
            key: key.clone(),
            support: if support_for_key(&key) {
                "supported".to_owned()
            } else {
                "not supported".to_owned()
            },
            configured_path: configured.path.display().to_string(),
            is_default: default_key.as_deref() == Some(&key),
        })
    }
}

impl ToArgs for MediaPlayerShowArgs {
    fn to_args(&self) -> Vec<std::ffi::OsString> {
        vec![self.key.clone().into()]
    }
}
