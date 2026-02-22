use crate::cli::app_state;
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
pub(super) fn print_detailed_profile(profile: &str, is_active: bool) -> Result<()> {
    let title = if is_active {
        format!("{} {}", profile, paint(GREEN, "(active)"))
    } else {
        profile.to_owned()
    };
    println!("{BOLD_CYAN}{title}{RESET}");

    let keypair = app_state::load_keypair(profile)?;
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

    let friends = app_state::list_friends(profile)?;
    println!("  {} {}", paint(YELLOW, "friends:"), friends.len());
    if friends.is_empty() {
        println!("    {}", paint(DIM, "<none>"));
    } else {
        for friend in &friends {
            println!("    {} ({})", friend.name, friend.pubkey);
        }
    }

    let friend_routes = app_state::list_friend_route_keys(profile, None)?;
    println!(
        "  {} {}",
        paint(YELLOW, "friend routes:"),
        friend_routes.len()
    );
    if friend_routes.is_empty() {
        println!("    {}", paint(DIM, "<none>"));
    } else {
        for route in &friend_routes {
            println!("    {} -> {}", route.friend, route.record_key);
        }
    }

    let listen_routes = app_state::list_local_route_identities(profile)?;
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
