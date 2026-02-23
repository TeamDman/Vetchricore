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

/// Print a profile in detailed colorful format.
///
/// # Errors
///
/// Returns an error if profile metadata cannot be loaded.
pub(super) fn print_detailed_profile(profile_home: &ProfileHome, is_active: bool) -> Result<()> {
    let profile = profile_home.profile();
    let title = if is_active {
        format!("{} {}", profile, paint(GREEN, "(active)"))
    } else {
        profile.to_owned()
    };
    println!("{BOLD_CYAN}{title}{RESET}");

    let keypair = app_state::load_keypair(profile_home)?;
    match keypair {
        Some(keypair) => {
            println!("  {} {}", paint(MAGENTA, "profile pubkey:"), keypair.key());
        }
        None => {
            println!(
                "  {} {}",
                paint(MAGENTA, "profile pubkey:"),
                paint(DIM, "<none>")
            );
        }
    }

    let known_users = app_state::list_known_users(profile_home)?;
    println!("  {} {}", paint(YELLOW, "known users:"), known_users.len());
    if known_users.is_empty() {
        println!("    {}", paint(DIM, "<none>"));
    } else {
        for known_user in &known_users {
            println!("    {} ({})", known_user.name, known_user.pubkey);
        }
    }

    let known_user_routes = app_state::list_known_user_route_keys(profile_home, None)?;
    println!(
        "  {} {}",
        paint(YELLOW, "known-user routes:"),
        known_user_routes.len()
    );
    if known_user_routes.is_empty() {
        println!("    {}", paint(DIM, "<none>"));
    } else {
        for route in &known_user_routes {
            println!("    {} -> {}", route.known_user, route.record_key);
        }
    }

    let listen_routes = app_state::list_local_route_identities(profile_home)?;
    println!(
        "  {} {}",
        paint(YELLOW, "listen routes:"),
        listen_routes.len()
    );
    if listen_routes.is_empty() {
        println!("    {}", paint(DIM, "<none>"));
    } else {
        for route in &listen_routes {
            println!("    {}", route.name);
            println!(
                "      {} {}",
                paint(MAGENTA, "record key:"),
                route.record_key
            );
            println!(
                "      {} {}",
                paint(MAGENTA, "public key:"),
                route.keypair.key()
            );
        }
    }

    Ok(())
}
