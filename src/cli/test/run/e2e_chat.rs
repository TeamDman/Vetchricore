use crate::cli::Cli;
use crate::cli::Command as CliCommand;
use crate::cli::InvokeContext;
use crate::cli::ToArgs;
use crate::cli::global_args::GlobalArgs;
use crate::cli::key::KeyArgs;
use crate::cli::key::KeyCommand;
use crate::cli::key::key_gen::KeyGenArgs;
use crate::cli::known_user::KnownUserArgs;
use crate::cli::known_user::KnownUserCommand;
use crate::cli::known_user::add::KnownUserAddArgs;
use crate::cli::known_user::route::KnownUserRouteArgs;
use crate::cli::known_user::route::KnownUserRouteCommand;
use crate::cli::known_user::route::add::KnownUserRouteAddArgs;
use crate::cli::profile::ProfileArgs;
use crate::cli::profile::ProfileCommand;
use crate::cli::profile::add::ProfileAddArgs;
use crate::cli::route::RouteArgs;
use crate::cli::route::RouteCommand;
use crate::cli::route::create::RouteCreateArgs;
use crate::cli::route::listen::RouteListenArgs;
use crate::cli::route::show::RouteShowArgs;
use crate::cli::send::SendArgs;
use crate::cli::send::SendCommand;
use crate::cli::send::chat::SendChatArgs;
use arbitrary::Arbitrary;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::ffi::OsString;
use std::io::BufRead;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command as ProcessCommand;
use std::process::Stdio;
use std::thread;
use std::time::Duration;
use tracing::info;

#[derive(Facet, Arbitrary, Debug, PartialEq, Default)]
pub struct E2eChatArgs;

impl E2eChatArgs {
    /// # Errors
    ///
    /// Returns an error if any end-to-end setup step or message assertion fails.
    pub async fn invoke(self, _context: &InvokeContext) -> Result<()> {
        run_e2e_chat().await
    }
}

impl ToArgs for E2eChatArgs {}

#[expect(
    clippy::too_many_lines,
    reason = "e2e setup intentionally documents full flow end-to-end"
)]
async fn run_e2e_chat() -> Result<()> {
    let temp = tempfile::tempdir().wrap_err("failed creating temp dir")?;
    let home_dir = temp.path().join("home");
    let cache_dir = temp.path().join("cache");
    std::fs::create_dir_all(&home_dir)?;
    std::fs::create_dir_all(&cache_dir)?;

    let exe = std::env::current_exe().wrap_err("failed to resolve current executable path")?;

    run_typed(
        &exe,
        &home_dir,
        &cache_dir,
        None,
        &CliCommand::Profile(ProfileArgs {
            command: ProfileCommand::Add(ProfileAddArgs {
                name: "Bob".to_owned(),
            }),
        }),
    )?;
    run_typed(
        &exe,
        &home_dir,
        &cache_dir,
        None,
        &CliCommand::Profile(ProfileArgs {
            command: ProfileCommand::Add(ProfileAddArgs {
                name: "Janet".to_owned(),
            }),
        }),
    )?;

    let bob_keygen = run_typed(
        &exe,
        &home_dir,
        &cache_dir,
        Some("Bob"),
        &CliCommand::Key(KeyArgs {
            command: KeyCommand::Gen(KeyGenArgs),
        }),
    )?;
    let bob_pubkey = parse_prefixed_line(&bob_keygen.stdout, "Public key: ")
        .ok_or_else(|| eyre::eyre!("failed parsing Bob public key"))?;

    let janet_keygen = run_typed(
        &exe,
        &home_dir,
        &cache_dir,
        Some("Janet"),
        &CliCommand::Key(KeyArgs {
            command: KeyCommand::Gen(KeyGenArgs),
        }),
    )?;
    let janet_pubkey = parse_prefixed_line(&janet_keygen.stdout, "Public key: ")
        .ok_or_else(|| eyre::eyre!("failed parsing Janet public key"))?;

    run_typed(
        &exe,
        &home_dir,
        &cache_dir,
        Some("Bob"),
        &CliCommand::KnownUser(KnownUserArgs {
            command: KnownUserCommand::Add(KnownUserAddArgs {
                name: "Janet".to_owned(),
                pubkey: janet_pubkey,
            }),
        }),
    )?;
    run_typed(
        &exe,
        &home_dir,
        &cache_dir,
        Some("Janet"),
        &CliCommand::KnownUser(KnownUserArgs {
            command: KnownUserCommand::Add(KnownUserAddArgs {
                name: "Bob".to_owned(),
                pubkey: bob_pubkey,
            }),
        }),
    )?;

    run_typed(
        &exe,
        &home_dir,
        &cache_dir,
        Some("Janet"),
        &CliCommand::Route(RouteArgs {
            command: RouteCommand::Create(RouteCreateArgs {
                name: "janet-inbox".to_owned(),
                listen: false,
            }),
        }),
    )?;

    let janet_route_show = run_typed(
        &exe,
        &home_dir,
        &cache_dir,
        Some("Janet"),
        &CliCommand::Route(RouteArgs {
            command: RouteCommand::Show(RouteShowArgs {
                name: "janet-inbox".to_owned(),
            }),
        }),
    )?;
    let route_record_key = parse_prefixed_line(&janet_route_show.stdout, "Record key: ")
        .ok_or_else(|| eyre::eyre!("failed parsing Janet route record key"))?;

    run_typed(
        &exe,
        &home_dir,
        &cache_dir,
        Some("Bob"),
        &CliCommand::KnownUser(KnownUserArgs {
            command: KnownUserCommand::Route(KnownUserRouteArgs {
                command: KnownUserRouteCommand::Add(KnownUserRouteAddArgs {
                    known_user: "Janet".to_owned(),
                    record_id: route_record_key,
                }),
            }),
        }),
    )?;

    let listener_command = CliCommand::Route(RouteArgs {
        command: RouteCommand::Listen(RouteListenArgs {
            name: "janet-inbox".to_owned(),
            count: Some(1),
        }),
    });
    log_typed_command(Some("Janet"), &listener_command);
    let listener_args = make_args(&home_dir, &cache_dir, Some("Janet"), &listener_command);
    let listener = spawn_streaming(&exe, &listener_args, "Janet")
        .wrap_err("failed to start Janet listener")?;

    std::thread::sleep(Duration::from_millis(1200));

    let sender_command = CliCommand::Send(SendArgs {
        command: SendCommand::Chat(SendChatArgs {
            to: "to".to_owned(),
            known_user: "Janet".to_owned(),
            message: Some("schoolbus".to_owned()),
            retry: Some(20),
        }),
    });
    log_typed_command(Some("Bob"), &sender_command);
    let sender_args = make_args(&home_dir, &cache_dir, Some("Bob"), &sender_command);
    let sender =
        spawn_streaming(&exe, &sender_args, "Bob").wrap_err("failed to start Bob sender")?;

    let (listener_output, _sender_output) = tokio::time::timeout(
        Duration::from_secs(45),
        tokio::task::spawn_blocking(move || {
            wait_for_parallel_commands(listener, "Janet listener", sender, "Bob sender")
        }),
    )
    .await
    .wrap_err("timed out waiting for parallel listener/sender completion")?
    .wrap_err("parallel command join failure")?
    .wrap_err("parallel command failed")?;

    if !listener_output.stdout.contains("Bob> schoolbus") {
        bail!(
            "expected Janet listener output to contain 'Bob> schoolbus', got:\n{}",
            listener_output.stdout
        );
    }

    println!(
        "e2e_chat passed using temporary home at {}",
        home_dir.display()
    );
    Ok(())
}

struct CommandOutput {
    success: bool,
    stdout: String,
    _stderr: String,
}

fn run_typed(
    exe: &Path,
    home_dir: &Path,
    cache_dir: &Path,
    profile: Option<&str>,
    command: &CliCommand,
) -> Result<CommandOutput> {
    let command_display = Cli::display_invocation(&command);
    let profile_label = profile.unwrap_or("default");
    let args = make_args(home_dir, cache_dir, profile, command);
    run_cli(exe, &args, profile_label, &command_display)
}

fn run_cli(
    exe: &Path,
    args: &[OsString],
    profile: &str,
    command_display: &str,
) -> Result<CommandOutput> {
    info!("Running command ({}): {}", profile, command_display);
    let output = run_cli_once(exe, args, profile)?;
    if !output.success {
        bail!("child command failed: {}", command_display);
    }
    Ok(output)
}

fn run_cli_once(exe: &Path, args: &[OsString], profile: &str) -> Result<CommandOutput> {
    let streaming_child = spawn_streaming(exe, args, profile)?;
    collect_streaming_output(streaming_child)
}

struct StreamingChild {
    child: Child,
    stdout_thread: thread::JoinHandle<Vec<u8>>,
    stderr_thread: thread::JoinHandle<Vec<u8>>,
}

fn spawn_streaming(exe: &Path, args: &[OsString], stream_prefix: &str) -> Result<StreamingChild> {
    let mut child = ProcessCommand::new(exe)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .wrap_err("failed running child command")?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| eyre::eyre!("child stdout unavailable"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| eyre::eyre!("child stderr unavailable"))?;

    let stdout_prefix = styled_stream_prefix(stream_prefix);
    let stderr_prefix = stdout_prefix.clone();

    let stdout_thread = thread::spawn(move || {
        let mut reader = std::io::BufReader::new(stdout);
        let mut all = Vec::new();
        let mut line = Vec::new();
        let mut out = std::io::stdout();
        loop {
            line.clear();
            let read = reader.read_until(b'\n', &mut line).unwrap_or(0);
            if read == 0 {
                break;
            }
            all.extend_from_slice(&line);
            let _ = out.write_all(stdout_prefix.as_bytes());
            let _ = out.write_all(&line);
            if !line.ends_with(b"\n") {
                let _ = out.write_all(b"\n");
            }
            let _ = out.flush();
        }
        all
    });

    let stderr_thread = thread::spawn(move || {
        let mut reader = std::io::BufReader::new(stderr);
        let mut all = Vec::new();
        let mut line = Vec::new();
        let mut err = std::io::stderr();
        loop {
            line.clear();
            let read = reader.read_until(b'\n', &mut line).unwrap_or(0);
            if read == 0 {
                break;
            }
            all.extend_from_slice(&line);
            let _ = err.write_all(stderr_prefix.as_bytes());
            let _ = err.write_all(&line);
            if !line.ends_with(b"\n") {
                let _ = err.write_all(b"\n");
            }
            let _ = err.flush();
        }
        all
    });

    Ok(StreamingChild {
        child,
        stdout_thread,
        stderr_thread,
    })
}

fn collect_streaming_output(mut streaming_child: StreamingChild) -> Result<CommandOutput> {
    let status = streaming_child
        .child
        .wait()
        .wrap_err("failed waiting for child command")?;

    collect_streaming_output_with_status(streaming_child, status.success())
}

fn collect_streaming_output_with_status(
    streaming_child: StreamingChild,
    success: bool,
) -> Result<CommandOutput> {
    let streaming_child = streaming_child;

    let stdout_bytes = streaming_child
        .stdout_thread
        .join()
        .map_err(|join_err| eyre::eyre!("failed joining stdout stream thread: {join_err:?}"))?;
    let stderr_bytes = streaming_child
        .stderr_thread
        .join()
        .map_err(|join_err| eyre::eyre!("failed joining stderr stream thread: {join_err:?}"))?;

    Ok(CommandOutput {
        success,
        stdout: String::from_utf8(stdout_bytes).wrap_err("child stdout was not utf-8")?,
        _stderr: String::from_utf8(stderr_bytes).wrap_err("child stderr was not utf-8")?,
    })
}

fn wait_for_parallel_commands(
    mut first: StreamingChild,
    first_name: &str,
    mut second: StreamingChild,
    second_name: &str,
) -> Result<(CommandOutput, CommandOutput)> {
    loop {
        if let Some(status) = first
            .child
            .try_wait()
            .wrap_err_with(|| format!("failed polling {first_name}"))?
        {
            let first_output = collect_streaming_output_with_status(first, status.success())?;
            if !first_output.success {
                let _ = second.child.kill();
                let _second_output = collect_streaming_output(second)?;
                bail!("{} failed while running in parallel.", first_name,);
            }

            let second_output = collect_streaming_output(second)?;
            if !second_output.success {
                bail!("{} failed while running in parallel.", second_name,);
            }

            return Ok((first_output, second_output));
        }

        if let Some(status) = second
            .child
            .try_wait()
            .wrap_err_with(|| format!("failed polling {second_name}"))?
        {
            let second_output = collect_streaming_output_with_status(second, status.success())?;
            if !second_output.success {
                let _ = first.child.kill();
                let _first_output = collect_streaming_output(first)?;
                bail!("{} failed while running in parallel.", second_name,);
            }

            let first_output = collect_streaming_output(first)?;
            if !first_output.success {
                bail!("{} failed while running in parallel.", first_name,);
            }

            return Ok((first_output, second_output));
        }

        std::thread::sleep(Duration::from_millis(100));
    }
}

fn styled_stream_prefix(stream_prefix: &str) -> String {
    let color = match stream_prefix {
        "Bob" => "\x1b[38;5;45m",
        "Janet" => "\x1b[38;5;213m",
        "default" => "\x1b[38;5;244m",
        _ => "\x1b[38;5;250m",
    };
    format!("{color}{stream_prefix}|\x1b[0m")
}

fn make_args_with_global(command: &impl ToArgs, global: &GlobalArgs) -> Vec<OsString> {
    let mut args = global.to_args();
    args.extend(command.to_args());
    args
}

fn make_global(home_dir: &Path, cache_dir: &Path, profile: Option<&str>) -> GlobalArgs {
    GlobalArgs {
        profile: profile.map(ToOwned::to_owned),
        home_dir: Some(PathBuf::from(home_dir)),
        cache_dir: Some(PathBuf::from(cache_dir)),
        debug: false,
        no_veilid_logs: false,
        log_filter: None,
        log_file: None,
    }
}

fn make_args(
    home_dir: &Path,
    cache_dir: &Path,
    profile: Option<&str>,
    command: &CliCommand,
) -> Vec<OsString> {
    let global = make_global(home_dir, cache_dir, profile);
    make_args_with_global(command, &global)
}

fn log_typed_command(profile: Option<&str>, command: &CliCommand) {
    info!(
        "Running command ({}): {}",
        profile.unwrap_or("default"),
        Cli::display_invocation(command)
    );
}

fn parse_prefixed_line(output: &str, prefix: &str) -> Option<String> {
    output
        .lines()
        .find_map(|line| line.strip_prefix(prefix).map(ToOwned::to_owned))
}
