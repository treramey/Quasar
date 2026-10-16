mod browser;
mod cli;
mod ssh;
mod tunnel;
mod upgrade;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command, ConfigCommand, OrchestratorCommand};
use std::path::Path;

fn main() -> Result<()> {
    // Old live sessions and shell configs still export NEBULA_* variables.
    // Normalize before parsing, reading settings, or starting any threads.
    quazar_core::env::normalize_legacy_environment();
    let cli = Cli::parse();
    // A `quazar ssh` / `quazar tunnel` from another machine may have sent its
    // settings along. Merge them before anything reads a setting, and before
    // a thread or a child exists to inherit the variable.
    quazar_tui::bundle::apply_forwarded();
    // Likewise before any thread or child: nothing `quazar` starts — the
    // daemon and everything it runs, or the TUI's own helpers — may
    // inherit the Claude Code session it was typed in.
    let dropped = quazar_core::env::scrub_host_claude_session();
    match cli.command {
        Some(Command::Daemon { foreground, adopt }) => {
            init_daemon_logging(foreground)?;
            if !dropped.is_empty() {
                tracing::info!(vars = ?dropped, "dropped the inherited Claude Code session env");
            }
            log_fatal(
                quazar_daemon::run_daemon(quazar_daemon::DaemonOpts { foreground, adopt }),
                &quazar_core::paths::daemon_log_path(),
            )
        }
        Some(Command::Reload) => upgrade::run_reload(),
        Some(Command::Add { path }) => quazar_tui::run_add_project(path),
        Some(Command::Config { command }) => quazar_tui::run_config(match command {
            ConfigCommand::Path => quazar_tui::ConfigOp::Path,
            ConfigCommand::Export { path } => quazar_tui::ConfigOp::Export { path },
            ConfigCommand::Import { source } => quazar_tui::ConfigOp::Import { source },
            ConfigCommand::Harnesses => quazar_tui::ConfigOp::Harnesses,
        }),
        Some(Command::Kill) => quazar_tui::run_kill(),
        Some(Command::Rename { title, force }) => {
            let mode = if force {
                quazar_tui::RenameMode::Force
            } else {
                quazar_tui::RenameMode::Auto
            };
            quazar_tui::run_rename(title.join(" "), mode)
        }
        Some(Command::Worktree { name, base }) => quazar_tui::run_worktree(name.join(" "), base),
        Some(Command::Spawn {
            task,
            kind,
            worktree,
            base,
        }) => quazar_tui::run_spawn(task.join(" "), kind, worktree, base),
        Some(Command::Open { files }) => quazar_tui::run_open(files),
        Some(Command::Sessions { json }) => quazar_tui::run_sessions(json),
        Some(Command::Read { session, lines }) => quazar_tui::run_read_session(&session, lines),
        Some(Command::Ask {
            session,
            question,
            timeout,
            no_wait,
        }) => quazar_tui::run_ask_session(&session, question.join(" "), timeout, !no_wait),
        Some(Command::Orchestrator { command }) => match command {
            OrchestratorCommand::List { json } => quazar_tui::run_orchestrator_list(json),
            OrchestratorCommand::Read { session, bytes } => {
                quazar_tui::run_orchestrator_read(&session, bytes)
            }
            OrchestratorCommand::Send { session, prompt } => {
                quazar_tui::run_orchestrator_send(&session, prompt.join(" "))
            }
            OrchestratorCommand::Spawn {
                project,
                worktree,
                base,
                kind,
                task,
            } => quazar_tui::run_orchestrator_spawn(
                &project,
                worktree.as_deref(),
                base,
                kind,
                task.join(" "),
            ),
            OrchestratorCommand::Review { session, tabs } => {
                quazar_tui::run_orchestrator_review(&session, tabs)
            }
        },
        Some(Command::Browser {
            port,
            bind,
            public,
            credential,
            no_open,
        }) => browser::run_browser(browser::BrowserOpts {
            port,
            // --public is --bind 0.0.0.0 with a name; clap keeps the two
            // from being given at once.
            bind: bind.unwrap_or(if public {
                browser::PUBLIC_BIND
            } else {
                browser::DEFAULT_BIND
            }),
            credential,
            open: !no_open,
        }),
        Some(Command::Ssh {
            host,
            path,
            no_sync_config,
        }) => ssh::run_ssh(&host, path.as_deref(), !no_sync_config),
        Some(Command::Tunnel {
            host,
            path,
            port,
            remote_port,
            no_sync_config,
        }) => tunnel::run_tunnel(tunnel::TunnelOpts {
            host,
            path,
            port,
            remote_port,
            sync_config: !no_sync_config,
        }),
        Some(Command::Upgrade { force }) => upgrade::run_upgrade(force),
        Some(Command::StaleDaemonNote) => {
            if quazar_daemon::lifecycle::daemon_is_stale() {
                println!("note: the running daemon was built from older code.");
                if quazar_daemon::handoff::daemon_can_restart() {
                    println!("{}", upgrade::RELOAD_HINT);
                } else {
                    println!("{}", upgrade::KILL_HINT);
                }
            }
            Ok(())
        }
        Some(Command::ProtocolVersion) => {
            println!("{}", quazar_core::PROTOCOL_VERSION);
            Ok(())
        }
        Some(Command::ProtocolMinCompatibleVersion) => {
            println!("{}", quazar_core::MIN_COMPATIBLE_PROTOCOL);
            Ok(())
        }
        Some(Command::RestartVersion) => {
            println!("{}", quazar_daemon::handoff::VERSION);
            Ok(())
        }
        None => match cli.dir {
            Some(dir) => quazar_tui::run_add_project(dir),
            None => {
                init_tui_logging()?;
                if !dropped.is_empty() {
                    tracing::info!(vars = ?dropped, "dropped the inherited Claude Code session env");
                }
                let handoff =
                    log_fatal(quazar_tui::run_tui(), &quazar_core::paths::tui_log_path())?;
                match handoff {
                    // Hosts-picker handoff: the TUI quit and restored the
                    // terminal so a fresh `quazar ssh` can exec over us (the
                    // local daemon and its sessions stay up).
                    Some(entry) => {
                        eprintln!("quazar: connecting to {}…", entry.host);
                        ssh::run_ssh(&entry.host, entry.path.as_deref(), true)
                    }
                    None => Ok(()),
                }
            }
        },
    }
}

/// Record a fatal top-level error in the log file before it goes to stderr —
/// the TUI's stderr disappears with the terminal, the daemon's is /dev/null.
fn log_fatal<T>(result: Result<T>, log_path: &Path) -> Result<T> {
    if let Err(err) = &result {
        quazar_core::crashlog::append(log_path, &format!("FATAL {err:#}"));
    }
    result
}

fn log_filter() -> tracing_subscriber::EnvFilter {
    tracing_subscriber::EnvFilter::try_from_env(quazar_core::env::LOG)
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
}

/// Route tracing to `log_path` (created on demand, appended, no ANSI) —
/// neither binary can log to the terminal: the TUI owns it and the daemon
/// has no stderr.
fn init_file_logging(log_path: &Path) -> Result<()> {
    std::fs::create_dir_all(quazar_core::paths::log_dir())?;
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;
    tracing_subscriber::fmt()
        .with_env_filter(log_filter())
        .with_writer(file)
        .with_ansi(false)
        .init();
    Ok(())
}

fn init_daemon_logging(foreground: bool) -> Result<()> {
    // The daemon runs detached with stderr on /dev/null — without this hook a
    // panic (on any thread, tokio workers included) leaves no trace.
    let log_path = quazar_core::paths::daemon_log_path();
    quazar_core::crashlog::install_panic_hook(log_path.clone());
    if foreground {
        tracing_subscriber::fmt()
            .with_env_filter(log_filter())
            .init();
        return Ok(());
    }
    init_file_logging(&log_path)
}

fn init_tui_logging() -> Result<()> {
    // Panic output to stderr dies with the alternate screen — capture it to
    // the log file. The TUI later wraps this hook with its terminal-restore,
    // so the chain on panic is: restore terminal → log to file → stderr.
    let log_path = quazar_core::paths::tui_log_path();
    quazar_core::crashlog::install_panic_hook(log_path.clone());
    // stdout belongs to the UI — log to file only.
    init_file_logging(&log_path)
}
