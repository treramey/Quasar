#!/bin/sh
# quazar installer — installs or updates `quazar` and its compatibility aliases.
#
#   curl -fsSL https://raw.githubusercontent.com/treramey/Quasar/main/install.sh | sh
#
# Grabs the prebuilt binary for this platform from the latest GitHub release,
# falling back to `cargo install --git` when no release (or no matching asset)
# exists. Running it again updates in place.
#
# Environment overrides:
#   QUAZAR_INSTALL_DIR   install destination (default: ~/.local/bin)
set -eu

REPO="treramey/Quasar"
INSTALL_DIR="${QUAZAR_INSTALL_DIR:-$HOME/.local/bin}"

say() { printf '%s\n' "$*"; }
err() {
    printf 'error: %s\n' "$*" >&2
    exit 1
}

detect_target() {
    case "$(uname -s)" in
    Darwin)
        case "$(uname -m)" in
        arm64) echo aarch64-apple-darwin ;;
        x86_64) echo x86_64-apple-darwin ;;
        *) return 1 ;;
        esac
        ;;
    Linux)
        case "$(uname -m)" in
        x86_64) echo x86_64-unknown-linux-musl ;;
        aarch64 | arm64) echo aarch64-unknown-linux-musl ;;
        *) return 1 ;;
        esac
        ;;
    *) err "quazar runs on macOS and Linux only (the daemon needs unix sockets)" ;;
    esac
}

install_from_release() {
    url="https://github.com/$REPO/releases/latest/download/quazar-$1.tar.gz"
    tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' EXIT
    say "downloading $url"
    curl -fsSL "$url" -o "$tmp/quazar.tar.gz" || return 1
    tar -xzf "$tmp/quazar.tar.gz" -C "$tmp"
    mkdir -p "$INSTALL_DIR"
    install -m 755 "$tmp/quazar" "$INSTALL_DIR/quazar"
}

install_from_source() {
    command -v cargo >/dev/null 2>&1 ||
        err "no prebuilt binary available and cargo is not installed — get Rust from https://rustup.rs and re-run"
    say "building from source (this takes a few minutes)…"
    cargo install --git "https://github.com/$REPO" quazar --locked --force
}

install_compatibility_aliases() {
    bin_dir=$(dirname "$1")
    ln -sf quazar "$bin_dir/quasar"
    ln -sf quazar "$bin_dir/nebula"
}

main() {
    command -v curl >/dev/null 2>&1 || err "curl is required"

    installed=""
    if target=$(detect_target); then
        if install_from_release "$target"; then
            installed="$INSTALL_DIR/quazar"
        else
            reason="couldn't download quazar-$target from the latest release"
        fi
    else
        reason="no prebuilt binary for $(uname -s) $(uname -m)"
    fi

    if [ -z "$installed" ]; then
        say "$reason — building from source instead"
        install_from_source
        installed="$HOME/.cargo/bin/quazar"
    fi

    install_compatibility_aliases "$installed"
    version=$("$installed" --version 2>/dev/null || echo quazar)
    say "installed $version → $installed"
    say "installed compatibility aliases → $(dirname "$installed")/quasar, $(dirname "$installed")/nebula"

    bin_dir=$(dirname "$installed")
    case ":$PATH:" in
    *":$bin_dir:"*) ;;
    *) say "note: $bin_dir is not on your PATH — add: export PATH=\"$bin_dir:\$PATH\"" ;;
    esac

    # Replacing the file doesn't touch an already-running daemon: sessions keep
    # running on the old binary until it's restarted. `quazar reload` restarts
    # it in place, sessions and all; a daemon from before reload existed can
    # only be restarted by `quazar kill`, which stops every session. `quazar
    # upgrade` handles this itself and suppresses the note via
    # QUAZAR_UPGRADE_HANDOFF.
    if [ -z "${QUAZAR_UPGRADE_HANDOFF:-}" ] && pgrep -f '(quazar|quasar|nebula) daemon' >/dev/null 2>&1; then
        say "note: a quazar daemon from the previous version is still running."
        say "      run 'quazar reload' to move it onto the new one (sessions keep running);"
        say "      a daemon too old for that needs 'quazar kill' (stops all sessions)."
    fi
}

main
