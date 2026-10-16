//! Names of the environment variables quazar reads and sets, so the daemon,
//! the TUI, the CLI, the hook installers and the e2e tests all spell them
//! from one place — a typo here fails to build instead of silently falling
//! back to a default.

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

/// Id of the agent a hook or CLI invocation is running inside. Set on every
/// agent PTY, scrubbed from plain terminals.
pub const AGENT_ID: &str = "QUAZAR_AGENT_ID";
/// Base URL of the daemon's hook receiver, set on agent PTYs.
pub const API_URL: &str = "QUAZAR_API_URL";
/// Bearer token the hook receiver expects, set on agent PTYs.
pub const API_TOKEN: &str = "QUAZAR_API_TOKEN";
/// Overrides the runtime dir holding the socket and pidfile.
pub const RUNTIME_DIR: &str = "QUAZAR_RUNTIME_DIR";
/// Overrides the data dir holding the database, config and logs.
pub const DATA_DIR: &str = "QUAZAR_DATA_DIR";
/// Moves `config.json` alone — into a dotfiles checkout, say — leaving the
/// database, the logs and `config.local.json` in the data dir.
pub const CONFIG_FILE: &str = "QUAZAR_CONFIG_FILE";
/// A SETTINGS BUNDLE (base64 JSON) that `quazar ssh` / `quazar tunnel` hand
/// the remote quazar. Read once at startup, merged into that machine's
/// settings, and removed from the environment before anything is spawned.
pub const IMPORT_BUNDLE: &str = "QUAZAR_IMPORT_BUNDLE";
/// Replaces every agent CLI with one command line, taken verbatim (tests
/// stand in `/bin/sh` or a stub script for `claude`).
pub const AGENT_CMD: &str = "QUAZAR_AGENT_CMD";
/// Idle-session reaper sweep period in ms; tests shorten it.
pub const IDLE_REAP_MS: &str = "QUAZAR_IDLE_REAP_MS";
/// External-worktree sync probe period in ms; tests shorten it.
pub const WORKTREE_SYNC_MS: &str = "QUAZAR_WORKTREE_SYNC_MS";
/// How long a WORKTREE HOOK may run before the daemon kills it, in ms
/// (default 30s); tests shorten it.
pub const HOOK_TIMEOUT_MS: &str = "QUAZAR_HOOK_TIMEOUT_MS";
/// `RUST_LOG`-style tracing filter for both the daemon and the TUI.
pub const LOG: &str = "QUAZAR_LOG";
/// Overrides the install script URL `quazar upgrade` / `quazar ssh` fetch.
pub const INSTALL_URL: &str = "QUAZAR_INSTALL_URL";
/// Editor command the file modals open, ahead of the config's `editor`.
pub const EDITOR: &str = "QUAZAR_EDITOR";
/// Cadence in seconds of the TUI's check for a newer published release
/// (the footer's `⇡ vX.Y.Z` update indicator); `0` turns it off, as the
/// e2e tests do so their footers never depend on what GitHub has published.
pub const UPDATE_CHECK_SECS: &str = "QUAZAR_UPDATE_CHECK_SECS";
/// A file the TUI writes its INPUT LATENCY PROBE's timeline to
/// (`make perf`); unset, there is no probe.
pub const PERF_LOG: &str = "QUAZAR_PERF_LOG";
/// Set by `quazar upgrade` on the install script it runs, so the script
/// leaves the "daemon still running" note to the upgrade. `install.sh`
/// reads it by this name.
pub const UPGRADE_HANDOFF: &str = "QUAZAR_UPGRADE_HANDOFF";
/// Set on a WORKTREE HOOK script: which hook it is running as
/// (`worktree-create` / `worktree-delete`), so one script can serve both.
pub const HOOK: &str = "QUAZAR_HOOK";
/// Set on a WORKTREE HOOK script: the worktree's branch.
pub const WORKTREE_BRANCH: &str = "QUAZAR_WORKTREE_BRANCH";
/// Set on a WORKTREE HOOK script: the worktree's id.
pub const WORKTREE_ID: &str = "QUAZAR_WORKTREE_ID";
/// Claude Code's own override of its config dir (`~/.claude`), honoured
/// wherever quazar reads Claude's settings or transcripts.
pub const CLAUDE_CONFIG_DIR: &str = "CLAUDE_CONFIG_DIR";
/// Codex's own override of its home (`~/.codex`), where quazar installs
/// its hooks.
pub const CODEX_HOME: &str = "CODEX_HOME";

/// Canonical environment variables that had a `NEBULA_` name before the
/// rename. This table is the compatibility authority for both startup
/// normalization and direct [`non_empty`] reads.
const LEGACY_ENV_ALIASES: &[(&str, &str)] = &[
    (AGENT_ID, "NEBULA_AGENT_ID"),
    (API_URL, "NEBULA_API_URL"),
    (API_TOKEN, "NEBULA_API_TOKEN"),
    (RUNTIME_DIR, "NEBULA_RUNTIME_DIR"),
    (DATA_DIR, "NEBULA_DATA_DIR"),
    (CONFIG_FILE, "NEBULA_CONFIG_FILE"),
    (IMPORT_BUNDLE, "NEBULA_IMPORT_BUNDLE"),
    (AGENT_CMD, "NEBULA_AGENT_CMD"),
    (IDLE_REAP_MS, "NEBULA_IDLE_REAP_MS"),
    (WORKTREE_SYNC_MS, "NEBULA_WORKTREE_SYNC_MS"),
    (HOOK_TIMEOUT_MS, "NEBULA_HOOK_TIMEOUT_MS"),
    (LOG, "NEBULA_LOG"),
    (INSTALL_URL, "NEBULA_INSTALL_URL"),
    (EDITOR, "NEBULA_EDITOR"),
    (UPDATE_CHECK_SECS, "NEBULA_UPDATE_CHECK_SECS"),
    (PERF_LOG, "NEBULA_PERF_LOG"),
    (UPGRADE_HANDOFF, "NEBULA_UPGRADE_HANDOFF"),
    (HOOK, "NEBULA_HOOK"),
    (WORKTREE_BRANCH, "NEBULA_WORKTREE_BRANCH"),
    (WORKTREE_ID, "NEBULA_WORKTREE_ID"),
];

const LEGACY_AGENT_ID: &str = "NEBULA_AGENT_ID";
const LEGACY_API_URL: &str = "NEBULA_API_URL";
const LEGACY_API_TOKEN: &str = "NEBULA_API_TOKEN";

/// Env vars that identify an agent session to the daemon. They are set on
/// every agent PTY and must never leak into plain terminals.
pub const AGENT_SESSION_VARS: &[&str] = &[
    AGENT_ID,
    API_URL,
    API_TOKEN,
    LEGACY_AGENT_ID,
    LEGACY_API_URL,
    LEGACY_API_TOKEN,
];

/// `TERM` every PTY child is given. The pane is quazar's own grid — a vt100
/// parser the TUI repaints through ratatui — and that grid keeps 24-bit
/// colour whatever terminal quazar itself runs in, so the child never
/// hears the host's `TERM` (`foot`, `xterm-ghostty`, `tmux-256color`).
pub const PANE_TERM: &str = "xterm-256color";
/// `COLORTERM` for the same grid: every `38;2;r;g;b` the child sends is
/// kept, so chalk-style detection may pick truecolor over the 256-colour
/// downsample `TERM` alone allows.
pub const PANE_COLORTERM: &str = "truecolor";
/// Colour overrides scrubbed from every PTY child. A `NO_COLOR` or a
/// `FORCE_COLOR=0` describes the shell the daemon happened to be started
/// from — an agent's tool shell, a CI job — or a login-only profile, not
/// the pane; Claude Code reads either as "no colour at all" and paints its
/// whole UI in the default foreground while the TUI around it stays
/// coloured (#37). The TUI still honours its own `NO_COLOR` on the way
/// out, so a user who wants none keeps none.
pub const PANE_COLOR_OVERRIDES: &[&str] = &["NO_COLOR", "FORCE_COLOR"];

/// Variables a Claude Code session sets on the shells it runs tools in,
/// naming that session: its id, its process, its messaging socket and
/// token, and the marker that says "this process is my sub-process". A
/// daemon started from such a shell — `quazar` typed by an agent, or run
/// in its terminal — would pass them to every agent, terminal and `claude`
/// probe it starts, and each would believe it belonged to that other
/// session: Claude Code turns its transcript off for an inherited
/// `CLAUDE_CODE_CHILD_SESSION` (so `--resume`, and with it a `quazar
/// worktree` relocation, finds no conversation), reports the wrong
/// session id, and talks to the other session's socket. Only identity is
/// listed: user settings that share the `CLAUDE_CODE_` prefix
/// (`CLAUDE_CODE_USE_BEDROCK`, an OAuth token) are kept.
pub const HOST_CLAUDE_SESSION_VARS: &[&str] = &[
    "CLAUDECODE",
    "CLAUDE_CODE_CHILD_SESSION",
    "CLAUDE_CODE_SESSION_ID",
    "CLAUDE_CODE_SESSION_ATTENDED",
    "CLAUDE_CODE_BRIDGE_SESSION_ID",
    "CLAUDE_CODE_ENTRYPOINT",
    "CLAUDE_CODE_EXECPATH",
    "CLAUDE_CODE_MESSAGING_SOCKET",
    "CLAUDE_CODE_MESSAGING_TOKEN",
    "CLAUDE_CODE_INVOKED_SKILLS",
    "CLAUDE_PID",
    "CLAUDE_EFFORT",
    "AI_AGENT",
];

/// Overrides Claude Code puts on its tool shells for its own non-interactive
/// use: `GIT_EDITOR=true` (in a quazar pane a bare `git commit` would abort
/// on an empty message) and `COREPACK_ENABLE_AUTO_PIN=0`. Dropped only at
/// exactly the value it sets, and only beside a session marker, so a user
/// who sets either themselves keeps it.
pub const HOST_CLAUDE_TOOL_SHELL_OVERRIDES: &[(&str, &str)] =
    &[("GIT_EDITOR", "true"), ("COREPACK_ENABLE_AUTO_PIN", "0")];

/// Whether an inherited `name=value` belongs to the Claude Code session
/// `quazar` was started from: one of [`HOST_CLAUDE_SESSION_VARS`], or —
/// when `in_session` (a session marker sits in the same environment) —
/// one of [`HOST_CLAUDE_TOOL_SHELL_OVERRIDES`] at its exact value.
pub fn is_host_claude_session_var(name: &OsStr, value: &OsStr, in_session: bool) -> bool {
    let Some(name) = name.to_str() else {
        return false;
    };
    HOST_CLAUDE_SESSION_VARS.contains(&name)
        || (in_session
            && HOST_CLAUDE_TOOL_SHELL_OVERRIDES
                .iter()
                .any(|&(var, set)| var == name && value == set))
}

/// The names in `vars` that belong to the host Claude Code session.
pub fn host_claude_session_names(
    vars: impl IntoIterator<Item = (OsString, OsString)>,
) -> Vec<OsString> {
    let vars: Vec<(OsString, OsString)> = vars.into_iter().collect();
    let in_session = vars
        .iter()
        .any(|(name, _)| name == "CLAUDECODE" || name == "CLAUDE_CODE_CHILD_SESSION");
    vars.into_iter()
        .filter(|(name, value)| is_host_claude_session_var(name, value, in_session))
        .map(|(name, _)| name)
        .collect()
}

/// Drop the inherited Claude Code session variables from this process's
/// environment, returning the names dropped. Called first thing in
/// `main`, before a thread or a child exists, so nothing `quazar` starts —
/// the daemon and its agents, terminals and probes, or the TUI's own
/// helpers — inherits them.
pub fn scrub_host_claude_session() -> Vec<String> {
    let names = host_claude_session_names(std::env::vars_os());
    for name in &names {
        std::env::remove_var(name);
    }
    names
        .into_iter()
        .map(|name| name.to_string_lossy().into_owned())
        .collect()
}

fn legacy_env_alias(var: &str) -> Option<&'static str> {
    LEGACY_ENV_ALIASES
        .iter()
        .find_map(|&(canonical, legacy)| (canonical == var).then_some(legacy))
}

fn env_value_with_legacy(
    var: &str,
    mut value: impl FnMut(&str) -> Option<OsString>,
) -> Option<OsString> {
    match value(var) {
        // Presence, including an empty or non-UTF-8 value, makes the
        // canonical name authoritative.
        present @ Some(_) => present,
        None => legacy_env_alias(var).and_then(value),
    }
}

fn pending_legacy_normalizations(
    mut value: impl FnMut(&str) -> Option<OsString>,
) -> Vec<(&'static str, OsString)> {
    LEGACY_ENV_ALIASES
        .iter()
        .filter_map(|&(canonical, legacy)| {
            if value(canonical).is_some() {
                None
            } else {
                value(legacy).map(|value| (canonical, value))
            }
        })
        .collect()
}

/// Copy legacy `NEBULA_*` values to their canonical `QUAZAR_*` names.
///
/// Call this once at CLI startup before argument parsing, while the process
/// is still single-threaded. A present canonical variable always wins, even
/// when it is empty. Legacy agent identity names are then removed so child
/// terminals cannot inherit the obsolete identity alongside the canonical
/// one.
pub fn normalize_legacy_environment() {
    for (canonical, value) in pending_legacy_normalizations(|name| std::env::var_os(name)) {
        std::env::set_var(canonical, value);
    }
    for legacy in [LEGACY_AGENT_ID, LEGACY_API_URL, LEGACY_API_TOKEN] {
        std::env::remove_var(legacy);
    }
}

/// The value of `var`, treating unset and empty the same way. For canonical
/// `QUAZAR_*` names, an unset variable falls back to its legacy `NEBULA_*`
/// alias. An empty canonical value intentionally requests the default and
/// therefore blocks the legacy fallback.
pub fn non_empty(var: &str) -> Option<String> {
    env_value_with_legacy(var, |name| std::env::var_os(name))
        .and_then(|value| value.into_string().ok())
        .filter(|value| !value.is_empty())
}

/// `$HOME`, when the environment has one. Read as an `OsString` so a
/// non-UTF-8 home still resolves — every `~/` expansion goes through here.
pub fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn host_claude_session_vars_are_identity_only() {
        let is = |name: &str, value: &str| {
            is_host_claude_session_var(OsStr::new(name), OsStr::new(value), true)
        };
        for name in [
            "CLAUDECODE",
            "CLAUDE_CODE_CHILD_SESSION",
            "CLAUDE_CODE_SESSION_ID",
            "CLAUDE_CODE_MESSAGING_SOCKET",
            "CLAUDE_CODE_MESSAGING_TOKEN",
            "CLAUDE_PID",
        ] {
            assert!(is(name, "1"), "{name}");
        }
        // User settings under the same prefix stay.
        for name in [
            "CLAUDE_CODE_USE_BEDROCK",
            "CLAUDE_CODE_OAUTH_TOKEN",
            CLAUDE_CONFIG_DIR,
            AGENT_ID,
            "PATH",
        ] {
            assert!(!is(name, "1"), "{name}");
        }
        assert!(is("GIT_EDITOR", "true"));
        assert!(!is("GIT_EDITOR", "vim"), "a user's editor stays");
        assert!(is("COREPACK_ENABLE_AUTO_PIN", "0"));
        assert!(
            !is("COREPACK_ENABLE_AUTO_PIN", "1"),
            "a user's choice stays"
        );
        assert!(
            !is_host_claude_session_var(OsStr::new("GIT_EDITOR"), OsStr::new("true"), false),
            "outside a Claude session, GIT_EDITOR=true is the user's"
        );
        assert!(
            !is("CLAUDE_CODE_SSE_PORT", "1"),
            "an IDE terminal's link stays"
        );
    }

    #[test]
    fn host_session_names_are_picked_out_of_an_environment() {
        let vars = [
            ("CLAUDE_CODE_CHILD_SESSION", "1"),
            ("CLAUDE_CODE_USE_BEDROCK", "1"),
            ("GIT_EDITOR", "true"),
            ("COREPACK_ENABLE_AUTO_PIN", "0"),
            ("COREPACK_ENABLE_AUTO_PIN_EXTRA", "0"),
            ("PATH", "/bin"),
        ]
        .map(|(k, v)| (OsString::from(k), OsString::from(v)));
        assert_eq!(
            host_claude_session_names(vars.clone()),
            [
                "CLAUDE_CODE_CHILD_SESSION",
                "GIT_EDITOR",
                "COREPACK_ENABLE_AUTO_PIN"
            ]
            .map(OsString::from)
        );
        // No session marker: the overrides are the user's own.
        let plain: Vec<_> = vars
            .into_iter()
            .filter(|(name, _)| name != "CLAUDE_CODE_CHILD_SESSION")
            .collect();
        assert!(host_claude_session_names(plain).is_empty());
    }

    #[test]
    fn non_empty_treats_unset_and_empty_alike() {
        let var = format!("QUAZAR_TEST_NON_EMPTY_{}", std::process::id());
        assert_eq!(non_empty(&var), None);
        std::env::set_var(&var, "");
        assert_eq!(non_empty(&var), None);
        std::env::set_var(&var, "x");
        assert_eq!(non_empty(&var).as_deref(), Some("x"));
        std::env::remove_var(&var);
    }

    #[test]
    fn legacy_env_values_obey_canonical_precedence() {
        let values = HashMap::from([
            (DATA_DIR, OsString::from("")),
            ("NEBULA_DATA_DIR", OsString::from("/legacy-data")),
            ("NEBULA_RUNTIME_DIR", OsString::from("/legacy-runtime")),
        ]);
        let lookup = |name: &str| values.get(name).cloned();

        assert_eq!(
            env_value_with_legacy(DATA_DIR, lookup),
            Some(OsString::new())
        );
        assert_eq!(
            env_value_with_legacy(RUNTIME_DIR, lookup),
            Some(OsString::from("/legacy-runtime"))
        );
        assert_eq!(env_value_with_legacy("HOME", lookup), None);

        assert_eq!(
            pending_legacy_normalizations(lookup),
            vec![(RUNTIME_DIR, OsString::from("/legacy-runtime"))]
        );
    }

    #[test]
    fn all_agent_identity_names_are_scrubbed_from_plain_terminals() {
        assert_eq!(
            AGENT_SESSION_VARS,
            [
                "QUAZAR_AGENT_ID",
                "QUAZAR_API_URL",
                "QUAZAR_API_TOKEN",
                "NEBULA_AGENT_ID",
                "NEBULA_API_URL",
                "NEBULA_API_TOKEN",
            ]
        );
    }
}
