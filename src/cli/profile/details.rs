use crate::cli::app_state;
use crate::cli::app_state::ProfileHome;
use eyre::Result;

const RESET: &str = "\x1b[0m";
const BOLD_CYAN: &str = "\x1b[1;36m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const MAGENTA: &str = "\x1b[35m";
const DIM: &str = "\x1b[2m";

fn paint(style: &str, value: &str) -> String {
    format!("{style}{value}{RESET}")
}

fn push_line(buffer: &mut String, line: &str) {
    if !buffer.is_empty() {
        buffer.push('\n');
    }
    buffer.push_str(line);
}

/// Format a profile in detailed colorful text.
///
/// # Errors
///
/// Returns an error if profile metadata cannot be loaded.
pub(super) fn format_detailed_profile(
    profile_home: &ProfileHome,
    is_active: bool,
) -> Result<String> {
    let mut out = String::new();

    let profile = profile_home.profile();
    let title = if is_active {
        format!("{} {}", profile, paint(GREEN, "(active)"))
    } else {
        profile.to_owned()
    };
    push_line(&mut out, &format!("{BOLD_CYAN}{title}{RESET}"));

    let keypair = app_state::load_keypair(profile_home)?;
    match keypair {
        Some(keypair) => {
            push_line(
                &mut out,
                &format!("  {} {}", paint(MAGENTA, "profile pubkey:"), keypair.key()),
            );
        }
        None => {
            push_line(
                &mut out,
                &format!(
                    "  {} {}",
                    paint(MAGENTA, "profile pubkey:"),
                    paint(DIM, "<none>")
                ),
            );
        }
    }

    let known_users = app_state::list_known_users(profile_home)?;
    push_line(
        &mut out,
        &format!("  {} {}", paint(YELLOW, "known users:"), known_users.len()),
    );
    if known_users.is_empty() {
        push_line(&mut out, &format!("    {}", paint(DIM, "<none>")));
    } else {
        for known_user in &known_users {
            push_line(
                &mut out,
                &format!("    {} ({})", known_user.name, known_user.pubkey),
            );
        }
    }

    let known_user_routes = app_state::list_known_user_route_keys(profile_home, None)?;
    push_line(
        &mut out,
        &format!(
            "  {} {}",
            paint(YELLOW, "known-user routes:"),
            known_user_routes.len()
        ),
    );
    if known_user_routes.is_empty() {
        push_line(&mut out, &format!("    {}", paint(DIM, "<none>")));
    } else {
        for route in &known_user_routes {
            push_line(
                &mut out,
                &format!("    {} -> {}", route.known_user, route.record_key),
            );
        }
    }

    let listen_routes = app_state::list_local_route_identities(profile_home)?;
    push_line(
        &mut out,
        &format!(
            "  {} {}",
            paint(YELLOW, "listen routes:"),
            listen_routes.len()
        ),
    );
    if listen_routes.is_empty() {
        push_line(&mut out, &format!("    {}", paint(DIM, "<none>")));
    } else {
        for route in &listen_routes {
            push_line(&mut out, &format!("    {}", route.name));
            push_line(
                &mut out,
                &format!(
                    "      {} {}",
                    paint(MAGENTA, "record key:"),
                    route.record_key
                ),
            );
            push_line(
                &mut out,
                &format!(
                    "      {} {}",
                    paint(MAGENTA, "public key:"),
                    route.keypair.key()
                ),
            );
        }
    }

    Ok(out)
}
