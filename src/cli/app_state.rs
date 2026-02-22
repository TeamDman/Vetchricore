use crate::cli::global_args::GlobalArgs;
use crate::paths::APP_HOME;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;
use veilid_core::KeyPair;
use veilid_core::PublicKey;
use veilid_core::RecordKey;

const PROFILES_DIR: &str = "profiles";
const ACTIVE_PROFILE_FILE: &str = "active_profile.txt";
const KEYPAIR_FILE: &str = "keypair.txt";
const FRIENDS_FILE: &str = "friends.tsv";
const ROUTES_FILE: &str = "routes.tsv";
const ROUTE_IDENTITIES_FILE: &str = "route_identities.tsv";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FriendEntry {
    pub name: String,
    pub pubkey: PublicKey,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalRouteIdentity {
    pub name: String,
    pub keypair: KeyPair,
    pub record_key: RecordKey,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FriendRouteEntry {
    pub friend: String,
    pub record_key: RecordKey,
}

/// Ensure app home and profile metadata exist.
///
/// # Errors
///
/// Returns an error if directories or initial files cannot be created.
pub fn ensure_initialized() -> Result<()> {
    APP_HOME.ensure_dir()?;
    std::fs::create_dir_all(profiles_root())?;

    if list_profiles()?.is_empty() {
        create_profile("main")?;
    }

    if !active_profile_file().exists() {
        set_active_profile("main")?;
    }

    Ok(())
}

/// Resolve the effective profile from global CLI arguments.
///
/// # Errors
///
/// Returns an error if initialization fails, the profile name is invalid,
/// or the selected profile does not exist.
pub fn resolve_profile(global: &GlobalArgs) -> Result<String> {
    ensure_initialized()?;

    if let Some(profile) = &global.profile {
        validate_profile_name(profile)?;
        if !profile_dir(profile).exists() {
            bail!("Profile '{}' does not exist.", profile);
        }
        return Ok(profile.clone());
    }

    current_active_profile()
}

/// List all local profiles.
///
/// # Errors
///
/// Returns an error if the profile directory cannot be read.
pub fn list_profiles() -> Result<Vec<String>> {
    let mut names = Vec::new();
    if !profiles_root().exists() {
        return Ok(names);
    }

    for entry in std::fs::read_dir(profiles_root())? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            names.push(entry.file_name().to_string_lossy().to_string());
        }
    }

    names.sort();
    Ok(names)
}

/// Create a new profile directory and default profile data files.
///
/// # Errors
///
/// Returns an error if the profile name is invalid, already exists,
/// or files/directories cannot be created.
pub fn create_profile(name: &str) -> Result<()> {
    validate_profile_name(name)?;
    let dir = profile_dir(name);
    if dir.exists() {
        bail!("Profile '{}' already exists.", name);
    }

    std::fs::create_dir_all(&dir)?;
    std::fs::write(friends_file(name), "")?;
    std::fs::write(routes_file(name), "")?;
    Ok(())
}

/// Remove a profile and adjust active profile if needed.
///
/// # Errors
///
/// Returns an error if the profile name is invalid, profile does not exist,
/// or filesystem operations fail.
pub fn remove_profile(name: &str) -> Result<()> {
    validate_profile_name(name)?;
    let dir = profile_dir(name);
    if !dir.exists() {
        bail!("Profile '{}' does not exist.", name);
    }

    std::fs::remove_dir_all(&dir)?;

    let active = current_active_profile()?;
    if active == name {
        if list_profiles()?.is_empty() {
            create_profile("main")?;
        }
        set_active_profile("main")?;
    }

    Ok(())
}

/// Set the active profile by name.
///
/// # Errors
///
/// Returns an error if the profile name is invalid, profile does not exist,
/// or the active profile file cannot be written.
pub fn set_active_profile(name: &str) -> Result<()> {
    validate_profile_name(name)?;
    if !profile_dir(name).exists() {
        bail!("Profile '{}' does not exist.", name);
    }
    std::fs::write(active_profile_file(), format!("{name}\n"))?;
    Ok(())
}

/// Get the currently active profile.
///
/// # Errors
///
/// Returns an error if initialization fails, the active profile file cannot be read,
/// or the file contains an empty profile name.
pub fn current_active_profile() -> Result<String> {
    ensure_initialized()?;
    let text =
        std::fs::read_to_string(active_profile_file()).wrap_err("failed to read active profile")?;
    let name = text.trim();
    if name.is_empty() {
        bail!("Active profile is empty.");
    }
    Ok(name.to_owned())
}

/// Load a profile's keypair from disk.
///
/// # Errors
///
/// Returns an error if the keypair file exists but cannot be read or parsed.
pub fn load_keypair(profile: &str) -> Result<Option<KeyPair>> {
    let path = keypair_file(profile);
    if !path.exists() {
        return Ok(None);
    }

    let text = std::fs::read_to_string(path)?;
    let keypair = text.trim().parse::<KeyPair>()?;
    Ok(Some(keypair))
}

/// Persist a profile keypair to disk.
///
/// # Errors
///
/// Returns an error if the profile does not exist or the keypair file cannot be written.
pub fn store_keypair(profile: &str, keypair: &KeyPair) -> Result<()> {
    ensure_profile_exists(profile)?;
    std::fs::write(keypair_file(profile), format!("{keypair}\n"))?;
    Ok(())
}

/// Remove the stored keypair for a profile if it exists.
///
/// # Errors
///
/// Returns an error if removing the keypair file fails.
pub fn remove_keypair(profile: &str) -> Result<()> {
    let path = keypair_file(profile);
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

/// List friends configured for a profile.
///
/// # Errors
///
/// Returns an error if the profile is invalid or friend data cannot be read/parsed.
pub fn list_friends(profile: &str) -> Result<Vec<FriendEntry>> {
    ensure_profile_exists(profile)?;
    parse_friends_file(&friends_file(profile))
}

/// Add a friend entry to a profile.
///
/// # Errors
///
/// Returns an error if the friend already exists or friend data cannot be persisted.
pub fn add_friend(profile: &str, name: &str, pubkey: PublicKey) -> Result<()> {
    let mut friends = list_friends(profile)?;
    if friends.iter().any(|f| f.name == name) {
        bail!("Friend '{}' already exists.", name);
    }
    friends.push(FriendEntry {
        name: name.to_owned(),
        pubkey,
    });
    friends.sort_by(|a, b| a.name.cmp(&b.name));
    write_friends_file(&friends_file(profile), &friends)
}

/// Rename a friend entry for a profile.
///
/// # Errors
///
/// Returns an error if the source friend does not exist, target name already exists,
/// or friend data cannot be persisted.
pub fn rename_friend(profile: &str, old_name: &str, new_name: &str) -> Result<()> {
    let mut friends = list_friends(profile)?;
    if friends.iter().any(|f| f.name == new_name) {
        bail!("Friend '{}' already exists.", new_name);
    }

    let Some(friend) = friends.iter_mut().find(|f| f.name == old_name) else {
        bail!("Friend '{}' does not exist.", old_name);
    };
    new_name.clone_into(&mut friend.name);

    friends.sort_by(|a, b| a.name.cmp(&b.name));
    write_friends_file(&friends_file(profile), &friends)
}

/// Remove a friend entry from a profile.
///
/// # Errors
///
/// Returns an error if the friend does not exist or friend data cannot be persisted.
pub fn remove_friend(profile: &str, name: &str) -> Result<()> {
    let mut friends = list_friends(profile)?;
    let prior_len = friends.len();
    friends.retain(|f| f.name != name);
    if friends.len() == prior_len {
        bail!("Friend '{}' does not exist.", name);
    }
    write_friends_file(&friends_file(profile), &friends)
}

/// Get a friend's public key by friend name.
///
/// # Errors
///
/// Returns an error if the profile data cannot be loaded.
pub fn friend_public_key(profile: &str, name: &str) -> Result<Option<PublicKey>> {
    let friends = list_friends(profile)?;
    Ok(friends
        .into_iter()
        .find(|f| f.name == name)
        .map(|f| f.pubkey))
}

/// Get a friend name by public key.
///
/// # Errors
///
/// Returns an error if the profile data cannot be loaded.
pub fn friend_name_by_public_key(profile: &str, pubkey: &PublicKey) -> Result<Option<String>> {
    let friends = list_friends(profile)?;
    Ok(friends
        .into_iter()
        .find(|f| &f.pubkey == pubkey)
        .map(|f| f.name))
}

/// Add a route record key for a friend in a profile.
///
/// # Errors
///
/// Returns an error if route data cannot be loaded or persisted.
pub fn add_route_key(profile: &str, friend: &str, record_key: &RecordKey) -> Result<()> {
    let mut routes = list_route_keys_by_friend(profile)?;
    let keys = routes.entry(friend.to_owned()).or_default();
    let record_key_text = record_key.to_string();
    if !keys.iter().any(|rk| rk == &record_key_text) {
        keys.push(record_key_text);
    }
    write_routes(profile, &routes)
}

/// Get known route record keys for a friend.
///
/// # Errors
///
/// Returns an error if route data cannot be loaded or record keys cannot be parsed.
pub fn route_keys_for_friend(profile: &str, friend: &str) -> Result<Vec<RecordKey>> {
    let routes = list_route_keys_by_friend(profile)?;
    let Some(keys) = routes.get(friend) else {
        return Ok(Vec::new());
    };

    keys.iter()
        .map(|k| k.parse::<RecordKey>().map_err(Into::into))
        .collect()
}

/// List friend route record keys, optionally filtered by friend name.
///
/// # Errors
///
/// Returns an error if route data cannot be loaded or record keys cannot be parsed.
pub fn list_friend_route_keys(
    profile: &str,
    friend: Option<&str>,
) -> Result<Vec<FriendRouteEntry>> {
    let routes = list_route_keys_by_friend(profile)?;
    let mut out = Vec::new();

    for (friend_name, keys) in routes {
        if let Some(target_friend) = friend
            && friend_name != target_friend
        {
            continue;
        }

        for key in keys {
            out.push(FriendRouteEntry {
                friend: friend_name.clone(),
                record_key: key.parse::<RecordKey>()?,
            });
        }
    }

    out.sort_by(|a, b| {
        a.friend
            .cmp(&b.friend)
            .then_with(|| a.record_key.to_string().cmp(&b.record_key.to_string()))
    });
    Ok(out)
}

/// Remove friend route record keys by optional friend and/or record key filters.
///
/// # Errors
///
/// Returns an error if route data cannot be loaded or persisted.
pub fn remove_friend_route_keys(
    profile: &str,
    friend: Option<&str>,
    record_key: Option<&RecordKey>,
) -> Result<usize> {
    let mut routes = list_route_keys_by_friend(profile)?;
    let before_count = routes.values().map(Vec::len).sum::<usize>();

    for (friend_name, keys) in &mut routes {
        if let Some(target_friend) = friend
            && friend_name != target_friend
        {
            continue;
        }

        if let Some(target_key) = record_key {
            let target_key_text = target_key.to_string();
            keys.retain(|key| key != &target_key_text);
        } else {
            keys.clear();
        }
    }

    routes.retain(|_, keys| !keys.is_empty());
    write_routes(profile, &routes)?;

    let after_count = routes.values().map(Vec::len).sum::<usize>();
    Ok(before_count.saturating_sub(after_count))
}

/// Persist a named local route identity for a profile.
///
/// # Errors
///
/// Returns an error if the route already exists or route identity data cannot be persisted.
pub fn add_local_route_identity(
    profile: &str,
    name: &str,
    keypair: &KeyPair,
    record_key: &RecordKey,
) -> Result<()> {
    validate_route_name(name)?;
    let mut identities = list_local_route_identities(profile)?;
    if identities.iter().any(|route| route.name == name) {
        bail!("Route '{}' already exists.", name);
    }

    identities.push(LocalRouteIdentity {
        name: name.to_owned(),
        keypair: keypair.clone(),
        record_key: record_key.clone(),
    });
    identities.sort_by(|a, b| a.name.cmp(&b.name));

    write_local_route_identities(profile, &identities)
}

/// Load a named local route identity for a profile.
///
/// # Errors
///
/// Returns an error if route identity data cannot be loaded or parsed.
pub fn local_route_identity(profile: &str, name: &str) -> Result<Option<LocalRouteIdentity>> {
    let identities = list_local_route_identities(profile)?;
    Ok(identities.into_iter().find(|route| route.name == name))
}

/// Remove a named local route identity for a profile.
///
/// # Errors
///
/// Returns an error if route identity data cannot be loaded/persisted,
/// or the route does not exist.
pub fn remove_local_route_identity(profile: &str, name: &str) -> Result<()> {
    let mut identities = list_local_route_identities(profile)?;
    let prior_len = identities.len();
    identities.retain(|route| route.name != name);
    if identities.len() == prior_len {
        bail!("Route '{}' does not exist.", name);
    }

    write_local_route_identities(profile, &identities)
}

/// List all named local route identities for a profile.
///
/// # Errors
///
/// Returns an error if route identity data cannot be loaded or parsed.
pub fn list_local_route_identities(profile: &str) -> Result<Vec<LocalRouteIdentity>> {
    ensure_profile_exists(profile)?;
    let path = route_identities_file(profile);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let mut routes = Vec::new();
    for line in std::fs::read_to_string(path)?.lines() {
        let mut parts = line.splitn(3, '\t');
        let Some(name) = parts.next() else {
            continue;
        };
        let Some(keypair_text) = parts.next() else {
            continue;
        };
        let Some(record_key_text) = parts.next() else {
            continue;
        };

        routes.push(LocalRouteIdentity {
            name: name.to_owned(),
            keypair: keypair_text.parse::<KeyPair>()?,
            record_key: record_key_text.parse::<RecordKey>()?,
        });
    }
    routes.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(routes)
}

fn write_local_route_identities(profile: &str, routes: &[LocalRouteIdentity]) -> Result<()> {
    let lines = routes
        .iter()
        .map(|route| format!("{}\t{}\t{}", route.name, route.keypair, route.record_key))
        .collect::<Vec<_>>();
    std::fs::write(route_identities_file(profile), lines.join("\n"))?;
    Ok(())
}

fn list_route_keys_by_friend(profile: &str) -> Result<BTreeMap<String, Vec<String>>> {
    ensure_profile_exists(profile)?;
    let mut out = BTreeMap::<String, Vec<String>>::new();
    let path = routes_file(profile);
    if !path.exists() {
        return Ok(out);
    }

    for line in std::fs::read_to_string(path)?.lines() {
        let Some((friend, record_key)) = line.split_once('\t') else {
            continue;
        };
        if friend.trim().is_empty() || record_key.trim().is_empty() {
            continue;
        }
        out.entry(friend.to_owned())
            .or_default()
            .push(record_key.to_owned());
    }

    Ok(out)
}

fn write_routes(profile: &str, routes: &BTreeMap<String, Vec<String>>) -> Result<()> {
    let mut lines = Vec::new();
    for (friend, keys) in routes {
        for key in keys {
            lines.push(format!("{friend}\t{key}"));
        }
    }
    std::fs::write(routes_file(profile), lines.join("\n"))?;
    Ok(())
}

fn parse_friends_file(path: &Path) -> Result<Vec<FriendEntry>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let mut friends = Vec::new();
    for line in std::fs::read_to_string(path)?.lines() {
        let Some((name, pubkey)) = line.split_once('\t') else {
            continue;
        };

        friends.push(FriendEntry {
            name: name.to_owned(),
            pubkey: pubkey.parse::<PublicKey>()?,
        });
    }

    friends.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(friends)
}

fn write_friends_file(path: &Path, friends: &[FriendEntry]) -> Result<()> {
    let lines = friends
        .iter()
        .map(|f| format!("{}\t{}", f.name, f.pubkey))
        .collect::<Vec<_>>();
    std::fs::write(path, lines.join("\n"))?;
    Ok(())
}

#[must_use]
pub fn profile_dir(profile: &str) -> PathBuf {
    profiles_root().join(profile)
}

#[must_use]
pub fn profile_veilid_dir(profile: &str) -> PathBuf {
    profile_dir(profile).join("veilid")
}

fn profiles_root() -> PathBuf {
    APP_HOME.file_path(PROFILES_DIR)
}

fn active_profile_file() -> PathBuf {
    APP_HOME.file_path(ACTIVE_PROFILE_FILE)
}

fn keypair_file(profile: &str) -> PathBuf {
    profile_dir(profile).join(KEYPAIR_FILE)
}

fn friends_file(profile: &str) -> PathBuf {
    profile_dir(profile).join(FRIENDS_FILE)
}

fn routes_file(profile: &str) -> PathBuf {
    profile_dir(profile).join(ROUTES_FILE)
}

fn route_identities_file(profile: &str) -> PathBuf {
    profile_dir(profile).join(ROUTE_IDENTITIES_FILE)
}

fn ensure_profile_exists(profile: &str) -> Result<()> {
    validate_profile_name(profile)?;
    let dir = profile_dir(profile);
    if !dir.exists() {
        bail!("Profile '{}' does not exist.", profile);
    }
    Ok(())
}

fn validate_profile_name(profile: &str) -> Result<()> {
    let trimmed = profile.trim();
    if trimmed.is_empty() {
        bail!("Profile name cannot be empty.");
    }
    if trimmed.contains(['/', '\\']) {
        bail!("Profile name cannot contain path separators.");
    }
    Ok(())
}

fn validate_route_name(name: &str) -> Result<()> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        bail!("Route name cannot be empty.");
    }
    if trimmed.contains(['/', '\\']) {
        bail!("Route name cannot contain path separators.");
    }
    Ok(())
}
