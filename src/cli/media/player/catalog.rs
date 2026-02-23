use std::collections::BTreeSet;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinSet;
use tracing::debug;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KnownMediaPlayer {
    pub key: &'static str,
    pub display_name: &'static str,
    pub supported: bool,
    pub exe_names: &'static [&'static str],
    pub aliases: &'static [&'static str],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DetectedMediaPlayer {
    pub key: String,
    pub path: PathBuf,
}

const KNOWN_MEDIA_PLAYERS: &[KnownMediaPlayer] = &[
    KnownMediaPlayer {
        key: "vlc",
        display_name: "VLC",
        supported: true,
        exe_names: &["vlc.exe"],
        aliases: &["videolan", "videolan-vlc"],
    },
    KnownMediaPlayer {
        key: "mpv",
        display_name: "MPV",
        supported: true,
        exe_names: &["mpv.exe"],
        aliases: &[],
    },
    KnownMediaPlayer {
        key: "mpvnet",
        display_name: "mpv.net",
        supported: true,
        exe_names: &["mpvnet.exe"],
        aliases: &["mpv-net"],
    },
    KnownMediaPlayer {
        key: "mpc-hc",
        display_name: "MPC-HC",
        supported: true,
        exe_names: &["mpc-hc.exe"],
        aliases: &["mpc", "mpchc"],
    },
    KnownMediaPlayer {
        key: "mpc-be",
        display_name: "MPC-BE",
        supported: true,
        exe_names: &["mpc-be.exe"],
        aliases: &["mpcbe"],
    },
    KnownMediaPlayer {
        key: "mplayer",
        display_name: "MPlayer",
        supported: true,
        exe_names: &["mplayer.exe"],
        aliases: &[],
    },
    KnownMediaPlayer {
        key: "memento",
        display_name: "Memento",
        supported: true,
        exe_names: &["memento.exe"],
        aliases: &[],
    },
    KnownMediaPlayer {
        key: "iina",
        display_name: "IINA",
        supported: true,
        exe_names: &["iina.exe"],
        aliases: &[],
    },
    KnownMediaPlayer {
        key: "wmplayer",
        display_name: "Windows Media Player",
        supported: false,
        exe_names: &["wmplayer.exe"],
        aliases: &["windows-media-player"],
    },
];

#[must_use]
pub fn known_media_player(key: &str) -> Option<&'static KnownMediaPlayer> {
    KNOWN_MEDIA_PLAYERS
        .iter()
        .find(|player| player.key.eq_ignore_ascii_case(key))
}

#[must_use]
pub fn canonical_media_player_key(input: &str) -> String {
    let trimmed = input.trim().to_ascii_lowercase();
    if let Some(found) = KNOWN_MEDIA_PLAYERS.iter().find(|player| {
        player.key.eq_ignore_ascii_case(&trimmed)
            || player
                .aliases
                .iter()
                .any(|alias| alias.eq_ignore_ascii_case(&trimmed))
    }) {
        return found.key.to_owned();
    }
    trimmed
}

#[must_use]
pub fn display_name_for_key(key: &str) -> String {
    known_media_player(key)
        .map(|player| player.display_name.to_owned())
        .unwrap_or_else(|| key.to_owned())
}

#[must_use]
pub fn support_for_key(key: &str) -> bool {
    known_media_player(key).is_some_and(|player| player.supported)
}

/// # Errors
///
/// Returns an error if reading environment variables or canonicalizing paths fails.
pub fn detect_media_players_on_path() -> eyre::Result<Vec<DetectedMediaPlayer>> {
    let mut detected = BTreeSet::<(String, PathBuf)>::new();
    let mut path_dirs = Vec::new();
    if let Some(path_var) = std::env::var_os("PATH") {
        path_dirs.extend(std::env::split_paths(&path_var));
    }

    debug!(
        path_dir_count = path_dirs.len(),
        "detecting media players on PATH"
    );

    for dir in path_dirs {
        for player in KNOWN_MEDIA_PLAYERS {
            for exe_name in player.exe_names {
                let candidate = dir.join(exe_name);
                if !candidate.is_file() {
                    continue;
                }
                let canonical = std::fs::canonicalize(&candidate).unwrap_or(candidate);
                detected.insert((player.key.to_owned(), canonical));
            }
        }
    }

    let detected = detected
        .into_iter()
        .map(|(key, path)| DetectedMediaPlayer { key, path })
        .collect::<Vec<_>>();
    debug!(
        detected_count = detected.len(),
        "completed PATH media player detection"
    );
    Ok(detected)
}

/// # Errors
///
/// Returns an error if root discovery fails.
pub async fn detect_media_players_by_walk(
    timeout: Duration,
    roots: &[PathBuf],
) -> eyre::Result<Vec<DetectedMediaPlayer>> {
    let mut detected = BTreeSet::<(String, PathBuf)>::new();
    let mut frontier = if roots.is_empty() {
        filesystem_roots()
    } else {
        roots.to_vec()
    };
    let lookup = Arc::new(exe_lookup_map());
    let deadline = tokio::time::Instant::now() + timeout;
    debug!(
        walk_timeout_ms = timeout.as_millis(),
        root_count = frontier.len(),
        using_explicit_roots = !roots.is_empty(),
        "starting filesystem walk media player detection"
    );

    while !frontier.is_empty() && tokio::time::Instant::now() < deadline {
        debug!(
            frontier_count = frontier.len(),
            detected_count = detected.len(),
            "scanning BFS frontier"
        );
        let mut join_set = JoinSet::new();
        for dir in &frontier {
            join_set.spawn(scan_directory(dir.clone(), Arc::clone(&lookup)));
        }

        let mut next_frontier = Vec::new();
        while !join_set.is_empty() {
            let Ok(next_result) = tokio::time::timeout_at(deadline, join_set.join_next()).await else {
                let detected = detected
                    .into_iter()
                    .map(|(key, path)| DetectedMediaPlayer { key, path })
                    .collect::<Vec<_>>();
                debug!(
                    detected_count = detected.len(),
                    "filesystem walk detection timed out"
                );
                return Ok(detected);
            };

            let Some(join_result) = next_result else {
                break;
            };

            let Ok((subdirs, found)) = join_result else {
                continue;
            };

            next_frontier.extend(subdirs);
            for (key, path) in found {
                detected.insert((key, path));
            }
        }

        frontier = next_frontier;
    }

    let detected = detected
        .into_iter()
        .map(|(key, path)| DetectedMediaPlayer { key, path })
        .collect::<Vec<_>>();
    debug!(
        detected_count = detected.len(),
        "finished filesystem walk media player detection"
    );
    Ok(detected)
}

async fn scan_directory(
    dir: PathBuf,
    exe_lookup: Arc<HashMap<String, String>>,
) -> (Vec<PathBuf>, Vec<(String, PathBuf)>) {
    let mut subdirs = Vec::new();
    let mut detected = Vec::new();

    let Ok(mut entries) = tokio::fs::read_dir(&dir).await else {
        return (subdirs, detected);
    };

    loop {
        let Ok(next_entry) = entries.next_entry().await else {
            break;
        };
        let Some(entry) = next_entry else {
            break;
        };

        let path = entry.path();
        let Ok(file_type) = entry.file_type().await else {
            continue;
        };

        if file_type.is_dir() {
            subdirs.push(path);
            continue;
        }

        if !file_type.is_file() {
            continue;
        }

        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        let file_name = file_name.to_ascii_lowercase();
        let Some(key) = exe_lookup.get(&file_name) else {
            continue;
        };

        let canonical = tokio::fs::canonicalize(&path).await.unwrap_or(path);
        detected.push((key.clone(), canonical));
    }

    (subdirs, detected)
}

fn exe_lookup_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    for player in KNOWN_MEDIA_PLAYERS {
        for exe_name in player.exe_names {
            map.insert(exe_name.to_ascii_lowercase(), player.key.to_owned());
        }
    }
    map
}

fn filesystem_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    #[cfg(windows)]
    {
        for drive_letter in 'A'..='Z' {
            let root = PathBuf::from(format!("{drive_letter}:\\"));
            if root.exists() {
                roots.push(root);
            }
        }
    }

    #[cfg(not(windows))]
    {
        roots.push(PathBuf::from("/"));
    }

    roots
}
