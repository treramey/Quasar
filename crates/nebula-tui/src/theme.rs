//! Color theme: every color the UI draws, keyed by role rather than
//! hardcoded at the call site. The active theme is picked by name in the
//! settings overlay (`theme` in config.json) and lives on `App`, refreshed
//! by the event loop when the setting changes.
//! The default theme follows Omarchy's active colors.toml when present.
//! Named presets remain independent of the desktop theme.
//!
//! Presets stick to ANSI-16 and 256-color indexed values so they render
//! everywhere. One exception: `focus_tint` needs a near-black shade of the
//! accent that the 256 palette simply doesn't have (its darkest chromatic
//! steps start around 40%), so it's truecolor RGB — supported by modern
//! terminals including Terminal.app since macOS Tahoe.

use ratatui::style::Color;

/// What the BLACK BACKGROUND setting paints under every cell nothing else
/// colored. Truecolor rather than ANSI `Black`, which a terminal palette is
/// free to map to a dark gray (a stock Ghostty's is #1d1f21) — the very
/// gray the setting exists to get away from.
pub const BLACK_BACKGROUND: Color = Color::Rgb(0, 0, 0);

/// Names the settings overlay cycles through; `by_name` accepts them
/// case-insensitively and falls back to the first entry.
pub const THEMES: &[&str] = &[
    "default", "ocean", "forest", "rose", "amber", "lavender", "coral", "slate", "sand", "mono",
];

/// Semantic color roles for the whole TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    /// Omarchy supplies the canvas as well as foreground roles.
    pub background: Option<Color>,
    /// Focus borders, titles, cursors, match highlights.
    pub accent: Color,
    /// Text drawn on top of an `accent` background (focused title chip).
    pub on_accent: Color,
    /// Primary input text.
    pub text: Color,
    /// Secondary text: unfocused titles, inactive breadcrumb segments.
    /// Also what `dim` spans brighten to on an unfocused selection bar.
    pub muted: Color,
    /// Hints, dividers, badges, archived rows; also the unfocused
    /// selection-bar background.
    pub dim: Color,
    /// Added / connected / a review file ticked off — the plain "this went
    /// well" green. Also a finished turn you have already read: still a
    /// good outcome, just no longer a job.
    pub ok: Color,
    /// A turn that finished and nobody has read yet: the dot, and the
    /// `n done` counts pointing at it. Deliberately NOT `ok` — the whole
    /// point is that this one wants a human, and green is the color a
    /// terminal teaches you to skip over. Reading the session turns the
    /// dot green. Sky blue unless the preset already owns blue, and never
    /// a violet: `merged` is purple in every preset, a merged checkout's
    /// row sits right above its unread session's, and the two dots say
    /// opposite things (come read this / this landed, delete it).
    pub done: Color,
    /// Running — the STATUS DOT and, on the grid, the running card's
    /// frame, one yellow for both — modified / flash messages / remote
    /// host.
    pub warn: Color,
    /// Needs feedback / deleted / destructive actions.
    pub err: Color,
    /// Terminated sessions and the session kind badge.
    pub special: Color,
    /// A merged pull request: its row's arrow, rail and badge, and the
    /// state word in the PR PREVIEW. Purple in every preset — the color
    /// GitHub paints a merge in, so the row says "landed" before the badge
    /// is read — and its own role rather than `special`, so a merged PR
    /// never wears the same color as a terminated session, whatever the
    /// preset makes of that one.
    pub merged: Color,
    /// The checkout a LAUNCHER CARD's session runs in, when that checkout
    /// is the project's ROOT WORKTREE: the branch its edits land on with
    /// nothing fenced around them. Amber in every preset, the way `merged`
    /// is purple in every preset — which of the two scopes a card is in is
    /// not a matter of taste, and two presets (`amber`, `sand`) carry a
    /// warm `accent` that would leave them unable to tell one from the
    /// other.
    pub root: Color,
    /// The counterpart, for a session in a worktree of its own — work
    /// fenced off from the root branch. Far enough from `root` across the
    /// 256-color cube to read as a different hue rather than a shade of
    /// it: telling the pair apart at a glance is the whole job, and
    /// `the_two_scope_colors_are_a_hue_apart_in_every_preset` holds them
    /// to it.
    pub worktree: Color,
    /// Selected-row fill in the focused panel (a subtle raised surface,
    /// not a reverse-video slab).
    pub sel_bg: Color,
    /// Selected-row fill in unfocused panels (barely raised).
    pub sel_bg_dim: Color,
    /// Structural chrome: column rules, header underlines, dividers.
    /// Darker than `dim` so the frame recedes behind the content.
    pub edge: Color,
    /// Shades for the running-row text sweep, `[tail, mid, head]`: the
    /// whole name sits on the tail shade while a brighter two-cell band
    /// sweeps across it. Yellow family, paired with `warn`.
    pub warn_sweep: [Color; 3],
    /// Needs-feedback counterpart of `warn_sweep`. Red family, paired
    /// with `err`.
    pub err_sweep: [Color; 3],
    /// Merged counterpart of `warn_sweep`, for the checkout row whose pull
    /// request has landed. Purple family, paired with `merged`. A ONE-SHOT
    /// SWEEP: the row rides it for a few seconds after the merge is seen,
    /// then rests on `merged`.
    pub merged_sweep: [Color; 3],
    /// Unread-done counterpart, the other ONE-SHOT SWEEP: a row rides it
    /// for a few seconds after a turn finishes unread, then holds still.
    /// Rests on `done`, so a preset that moves `done` moves this with it.
    pub done_sweep: [Color; 3],
    /// Focused-surface background, behind the session pane or the card
    /// keys land in: the accent's own hue taken down to a near-black —
    /// OKLCH lightness 0.20 and chroma 0.04 in every preset (mono's is
    /// the one gray). On a black window it reads as the accent glowing
    /// faintly rather than as a gray slab, and it is darker than the gray
    /// it replaced, so dim text on it keeps more of its contrast.
    /// Truecolor by necessity (see module docs).
    pub focus_tint: Color,
}

/// `done` and its sweep for a preset that already owns blue.
const DONE_PINK: [Color; 3] = [
    Color::Indexed(212),
    Color::Indexed(218),
    Color::Indexed(225),
];
/// `done` and its sweep for a preset whose pinks and violets crowd the blue.
const DONE_TURQUOISE: [Color; 3] = [Color::Indexed(45), Color::Indexed(81), Color::Indexed(159)];

impl Default for Theme {
    fn default() -> Self {
        // The classic nebula look: cyan accent on ANSI colors.
        Self {
            background: None,
            accent: Color::Cyan,
            on_accent: Color::Black,
            text: Color::White,
            muted: Color::Gray,
            dim: Color::DarkGray,
            ok: Color::Green,
            done: Color::Indexed(75), // sky blue — a hue away from merged's purple, not a shade
            warn: Color::Yellow,
            err: Color::Red,
            special: Color::Magenta,
            merged: Color::Indexed(135),  // purple — GitHub's merged
            root: Color::Indexed(214),    // amber — the branch itself
            worktree: Color::Indexed(79), // aquamarine — fenced off from it
            sel_bg: Color::Indexed(237),
            sel_bg_dim: Color::Indexed(235),
            edge: Color::Indexed(238),
            warn_sweep: [Color::Yellow, Color::Indexed(220), Color::Indexed(230)],
            err_sweep: [Color::Red, Color::Indexed(203), Color::Indexed(217)],
            merged_sweep: [
                Color::Indexed(135),
                Color::Indexed(141),
                Color::Indexed(183),
            ],
            done_sweep: [Color::Indexed(75), Color::Indexed(111), Color::Indexed(153)],
            focus_tint: Color::Rgb(0, 27, 28),
        }
    }
}

impl Theme {
    pub fn by_name(name: &str) -> Self {
        if name.trim().eq_ignore_ascii_case("default") {
            if let Some(theme) = omarchy_theme() {
                return theme;
            }
        }
        Self::preset(name)
    }

    fn preset(name: &str) -> Self {
        let base = Self::default();
        match name.trim().to_ascii_lowercase().as_str() {
            "ocean" => Self {
                accent: Color::Indexed(39),  // deep sky blue
                special: Color::Indexed(75), // steel blue
                // Blue is spoken for twice over here (the done sky blue IS
                // this preset's terminated), so done goes pink — the one
                // hue a blue preset leaves free.
                done: DONE_PINK[0],
                done_sweep: DONE_PINK,
                focus_tint: Color::Rgb(3, 24, 38),
                ..base
            },
            "forest" => Self {
                accent: Color::Indexed(114),  // pale green
                special: Color::Indexed(108), // sage
                focus_tint: Color::Rgb(10, 27, 10),
                ..base
            },
            "rose" => Self {
                accent: Color::Indexed(211),  // pink
                special: Color::Indexed(141), // violet
                // Pink and violet are both spoken for here, and sky blue
                // sits too near the violet, so done goes turquoise — the one
                // hue this preset leaves free.
                done: DONE_TURQUOISE[0],
                done_sweep: DONE_TURQUOISE,
                focus_tint: Color::Rgb(36, 14, 21),
                ..base
            },
            "amber" => Self {
                accent: Color::Indexed(214),  // orange
                special: Color::Indexed(173), // copper
                focus_tint: Color::Rgb(32, 19, 1),
                ..base
            },
            "lavender" => Self {
                accent: Color::Indexed(147),  // periwinkle
                special: Color::Indexed(176), // orchid
                // The done sky blue would vanish into a periwinkle accent,
                // so done goes turquoise here, as in rose.
                done: DONE_TURQUOISE[0],
                done_sweep: DONE_TURQUOISE,
                focus_tint: Color::Rgb(20, 19, 39),
                ..base
            },
            "coral" => Self {
                accent: Color::Indexed(209), // coral
                // The cooled-off complement, the way default pairs cyan with
                // magenta: a terminated session in a warm preset reads as
                // gone cold rather than as a faded needs-feedback red.
                special: Color::Indexed(73), // cadet teal
                focus_tint: Color::Rgb(37, 15, 8),
                ..base
            },
            "slate" => Self {
                accent: Color::Indexed(110), // dusty sky blue
                special: Color::Indexed(67), // steel blue
                // The done sky blue would land between this preset's two
                // blues, so done goes pink here, as in ocean.
                done: DONE_PINK[0],
                done_sweep: DONE_PINK,
                focus_tint: Color::Rgb(6, 23, 39),
                ..base
            },
            "sand" => Self {
                accent: Color::Indexed(180),  // tan
                special: Color::Indexed(137), // bronze
                focus_tint: Color::Rgb(34, 18, 2),
                ..base
            },
            "mono" => Self {
                // Grayscale chrome for a terminal whose own palette is
                // colorful enough: the frame, titles and cursors go white
                // and gray, and only the statuses keep their color.
                accent: Color::White,
                // A step under the accent, so a white-on-white match
                // highlight still stands out from the text around it.
                text: Color::Indexed(252),
                // And a step under that, so unfocused titles still read as
                // secondary next to the text.
                muted: Color::Indexed(247),
                special: Color::Indexed(245),
                focus_tint: Color::Rgb(22, 22, 22),
                ..base
            },
            _ => base,
        }
    }
}

/// Read the active Omarchy palette without changing application settings.
/// Missing or incomplete palettes retain the built-in theme.
pub fn omarchy_theme() -> Option<Theme> {
    use std::sync::{Mutex, OnceLock};
    use std::time::{Duration, Instant};
    // Bound filesystem reads even when several widgets ask during a frame.
    static CACHE: OnceLock<Mutex<(Instant, Option<Theme>)>> = OnceLock::new();
    let mut cache = CACHE
        .get_or_init(|| Mutex::new((Instant::now(), load_omarchy_theme())))
        .lock()
        .ok()?;
    if cache.0.elapsed() >= Duration::from_secs(1) {
        *cache = (Instant::now(), load_omarchy_theme());
    }
    cache.1
}

fn load_omarchy_theme() -> Option<Theme> {
    let home = nebula_core::env::home_dir()?;
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|p| !p.is_empty())
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| home.join(".config"));
    load_omarchy_theme_at(&home, &config)
}

fn load_omarchy_theme_at(home: &std::path::Path, config: &std::path::Path) -> Option<Theme> {
    // omarchy-theme-set publishes here, independently of XDG_CONFIG_HOME.
    // Only older installations publish under the config directory.
    let current = home.join(".local/state/omarchy/current/theme/colors.toml");
    let raw = match std::fs::read_to_string(current) {
        Ok(raw) => raw,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            std::fs::read_to_string(config.join("omarchy/current/theme/colors.toml")).ok()?
        }
        Err(_) => return None,
    };
    parse_omarchy_palette(&raw)
}

fn parse_omarchy_palette(raw: &str) -> Option<Theme> {
    // Omarchy's color contract is a flat table of quoted #RRGGBB values.
    let mut colors = std::collections::HashMap::new();
    for line in raw.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim();
        let Some(quote) = value.chars().next() else {
            continue;
        };
        if !matches!(quote, '\'' | '"') {
            continue;
        }
        let Some(hex) = value[1..]
            .split_once(quote)
            .map(|(hex, _)| hex)
            .and_then(|v| v.strip_prefix('#'))
        else {
            continue;
        };
        if hex.len() != 6 || !hex.is_ascii() {
            continue;
        }
        let Ok(rgb) = u32::from_str_radix(hex, 16) else {
            continue;
        };
        colors.insert(
            key.trim(),
            Color::Rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8),
        );
    }
    let get = |key| colors.get(key).copied();
    let bg = get("background")?;
    let fg = get("foreground")?;
    let accent = get("accent")?;
    let warn = get("color3")?;
    let err = get("color1")?;
    let merged = get("color5")?;
    let done = get("color4")?;
    let sweep = |c| [c, blend(c, fg, 0.3), blend(c, fg, 0.6)];
    Some(Theme {
        background: Some(bg),
        accent,
        on_accent: bg,
        text: fg,
        muted: blend(bg, fg, 0.7),
        dim: blend(bg, fg, 0.5),
        ok: get("color2")?,
        done,
        warn,
        err,
        special: get("color13").unwrap_or(merged),
        merged,
        root: warn,
        worktree: get("color6")?,
        sel_bg: get("selection_background").unwrap_or(blend(bg, accent, 0.25)),
        sel_bg_dim: blend(bg, fg, 0.1),
        edge: blend(bg, fg, 0.25),
        warn_sweep: sweep(warn),
        err_sweep: sweep(err),
        merged_sweep: sweep(merged),
        done_sweep: sweep(done),
        focus_tint: blend(bg, accent, 0.12),
    })
}

/// Interpolate RGB palette colors, clamping the blend to 0..=1.
/// Non-RGB colors use the destination without guessing terminal RGB values.
pub fn blend(from: Color, to: Color, amount: f32) -> Color {
    match (from, to) {
        (Color::Rgb(r, g, b), Color::Rgb(x, y, z)) => {
            let mix = |a: u8, b: u8| {
                (f32::from(a) + (f32::from(b) - f32::from(a)) * amount.clamp(0.0, 1.0)).round()
                    as u8
            };
            Color::Rgb(mix(r, x), mix(g, y), mix(b, z))
        }
        _ => to,
    }
}

/// The NESTED layout's grays (Settings → Appearance → **Worktree layout**
/// → `nested`): a live root's title against an idle one's, the ages and
/// counts, and the cursor row's fill. Truecolor, like `focus_tint`: an
/// ANSI gray is whatever the terminal's palette makes of it, and the two
/// title steps could land on one shade there. Every hue the layout draws
/// — the status dots, the cursor's accent — is still the preset's own.
pub mod nested {
    use ratatui::style::Color;

    pub fn dim(th: super::Theme) -> Color {
        if th.background.is_some() {
            th.muted
        } else {
            DIM
        }
    }

    pub fn bright(th: super::Theme) -> Color {
        if th.background.is_some() {
            th.text
        } else {
            BRIGHT
        }
    }

    pub fn link(th: super::Theme) -> Color {
        if th.background.is_some() {
            th.done
        } else {
            LINK
        }
    }

    /// A live root's title: working, or waiting on you.
    pub const BRIGHT: Color = Color::Rgb(0xf2, 0xf2, 0xf2);
    /// An idle root's title, and every row's age, count and connector.
    pub const DIM: Color = Color::Rgb(0x8c, 0x8c, 0x8c);
    /// A pull request's `#42`, underlined: a link, in the blue links wear.
    pub const LINK: Color = Color::Rgb(0x58, 0xa6, 0xff);
    /// How much of the accent the cursor's row keeps as its fill: the
    /// accent taken nearly to black, so the row is washed in it.
    pub const SELECTED_FILL: f32 = 0.18;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn omarchy_state_palette_takes_precedence_over_legacy_config() {
        let home = tempfile::tempdir().unwrap();
        let config = home.path().join("custom-config");
        let state = home
            .path()
            .join(".local/state/omarchy/current/theme/colors.toml");
        let legacy = config.join("omarchy/current/theme/colors.toml");
        // Required colors from the installed Omarchy palette.
        let raw = "background = \"#171b1a\"\nforeground = \"#b8c2ae\"\naccent = \"#7fa38d\"\ncolor1 = \"#b0573f\"\ncolor2 = \"#7f966b\"\ncolor3 = \"#c9a63a\"\ncolor4 = \"#6f8f96\"\ncolor5 = \"#8b6f5c\"\ncolor6 = \"#5c9680\"";
        assert!(load_omarchy_theme_at(home.path(), &config).is_none());
        std::fs::create_dir_all(legacy.parent().unwrap()).unwrap();
        std::fs::write(&legacy, raw.replace("#171b1a", "#ffffff")).unwrap();
        assert_eq!(
            load_omarchy_theme_at(home.path(), &config)
                .unwrap()
                .background,
            Some(Color::Rgb(255, 255, 255))
        );
        std::fs::create_dir_all(state.parent().unwrap()).unwrap();
        std::fs::write(&state, raw).unwrap();
        let theme = load_omarchy_theme_at(home.path(), &config).unwrap();
        assert_eq!(theme.background, Some(Color::Rgb(23, 27, 26)));
        assert_eq!(theme.accent, Color::Rgb(127, 163, 141));
        // An incomplete current palette must not activate a stale legacy one.
        std::fs::write(&state, "accent = \"#broken\"").unwrap();
        assert!(load_omarchy_theme_at(home.path(), &config).is_none());
    }

    #[test]
    fn omarchy_palette_maps_roles_and_rejects_incomplete_input() {
        let raw = "background = \"#101020\"\nforeground = \"#eeeeee\"\naccent = \"#80aaff\"\ncolor1 = \"#ee5555\"\ncolor2 = \"#55ee55\"\ncolor3 = \"#eeee55\"\ncolor4 = \"#5555ee\"\ncolor5 = \"#ee55ee\"\ncolor6 = \"#55eeee\"";
        let th = parse_omarchy_palette(raw).unwrap();
        assert_eq!(th.background, Some(Color::Rgb(16, 16, 32)));
        assert_eq!(th.accent, Color::Rgb(128, 170, 255));
        assert_eq!(th.err, Color::Rgb(238, 85, 85));
        assert_eq!(th.warn_sweep[0], th.warn);
        assert_ne!(th.focus_tint, th.background.unwrap());
        assert!(parse_omarchy_palette("accent = \"#bad\"").is_none());
        let light = parse_omarchy_palette(&raw.replace("#101020", "#ffffff")).unwrap();
        assert_eq!(light.on_accent, Color::Rgb(255, 255, 255));
        assert_ne!(light.focus_tint, th.focus_tint);
    }

    /// Red, green, blue coordinates (0..=5) of a 6x6x6 cube color.
    fn cube(c: Color) -> Option<[u8; 3]> {
        match c {
            Color::Indexed(n @ 16..=231) => {
                let n = n - 16;
                Some([n / 36, (n / 6) % 6, n % 6])
            }
            _ => None,
        }
    }

    /// How far two cube colors sit apart, in steps; None when either is
    /// not a cube color and the distance is not ours to measure.
    fn cube_steps(a: Color, b: Color) -> Option<u8> {
        let (a, b) = (cube(a)?, cube(b)?);
        Some((0..3).map(|i| a[i].abs_diff(b[i])).sum())
    }

    #[test]
    fn by_name_covers_all_presets_and_falls_back() {
        for name in THEMES {
            // Every listed preset must parse (and not accidentally be a
            // misspelling that falls back to default without noticing).
            let theme = Theme::preset(name);
            if *name != "default" {
                assert_ne!(theme, Theme::default(), "{name} fell back to default");
            }
        }
        assert_eq!(Theme::preset("no-such-theme"), Theme::default());
        assert_eq!(Theme::preset(" Ocean "), Theme::preset("ocean"));

        // A done dot has to stay readable AS done: never green (that's
        // `ok`, which the eye files as "nothing to do here"), and never
        // the same color as another status in the same preset.
        for name in THEMES {
            let th = Theme::preset(name);
            assert_ne!(th.done, th.ok, "{name}: done reads as plain success");
            assert_ne!(th.done, th.warn, "{name}: done reads as running");
            assert_ne!(th.done, th.err, "{name}: done reads as needs-feedback");
            assert_ne!(th.done, th.special, "{name}: done reads as terminated");
            assert_ne!(th.done, th.dim, "{name}: done reads as fresh");
            assert_ne!(th.done, th.accent, "{name}: done reads as the cursor");
        }
    }

    /// A running card's frame is the running yellow itself — the color its
    /// STATUS DOT wears — so no preset may hand `warn` to the accent or to
    /// the quiet edge: a running card would then pass for the cursor's, or
    /// for a card with nothing going on.
    #[test]
    fn a_running_frame_never_reads_as_the_focus() {
        for name in THEMES {
            let th = Theme::preset(name);
            assert_ne!(th.warn, th.accent, "{name}: running reads as the cursor");
            assert_ne!(th.warn, th.edge, "{name}: running reads as a quiet card");
        }
    }

    /// An unread-done dot sits right under a merged checkout's purple one,
    /// and the two mean opposite things — so done keeps a whole hue away
    /// from the merge, not a shade: it is no color the merged sweep passes
    /// through (the branch name would flash "done" on every pass), and —
    /// when both sit in the 256-color cube — it is at least three steps
    /// from the merge there. The violet this replaced was one.
    #[test]
    fn done_is_never_a_shade_of_merged() {
        for name in THEMES {
            let th = Theme::preset(name);
            assert!(
                !th.merged_sweep.contains(&th.done),
                "{name}: the merged sweep flashes the done color"
            );
            if let (Some(done), Some(merged)) = (cube(th.done), cube(th.merged)) {
                let steps: u8 = (0..3).map(|i| done[i].abs_diff(merged[i])).sum();
                assert!(
                    steps >= 3,
                    "{name}: done is {steps} cube steps from merged: a shade, not a hue"
                );
            }
        }
    }

    /// The ONE-SHOT SWEEP an unread finish rides rests on that preset's
    /// `done` — a preset that moves the one moves the other — brightens
    /// toward its head, and is nobody else's sweep.
    #[test]
    fn done_sweep_rests_on_done_in_every_preset() {
        for name in THEMES {
            let th = Theme::preset(name);
            assert_eq!(th.done_sweep[0], th.done, "{name}: the sweep rests on done");
            let [tail, mid, head] = th.done_sweep;
            assert!(
                tail != mid && mid != head && tail != head,
                "{name}: a flat band"
            );
            for other in [th.warn_sweep, th.err_sweep, th.merged_sweep] {
                assert!(
                    th.done_sweep.iter().all(|c| !other.contains(c)),
                    "{name}: the done sweep borrows a shade from another"
                );
            }
        }
    }

    /// A LAUNCHER CARD names the checkout its session runs in in a SCOPE
    /// COLOR — `⌂` amber on the project's root branch, `↳` aquamarine in a
    /// worktree of its own — so a screenful of cards sorts into the two
    /// without a word being read. That only holds while the pair keeps a
    /// whole hue between them (the three cube steps `done` keeps from
    /// `merged`), and while neither lands on a color the same card already
    /// wears somewhere else: the status its dot and name take, the `done`
    /// its ago label takes, the `merged` its pull-request row takes, or
    /// the dim its harness takes.
    #[test]
    fn the_two_scope_colors_are_a_hue_apart_in_every_preset() {
        for name in THEMES {
            let th = Theme::preset(name);
            assert_ne!(th.root, th.worktree, "{name}: one scope, painted twice");
            let steps = cube_steps(th.root, th.worktree);
            assert!(
                steps.is_none_or(|s| s >= 3),
                "{name}: the scopes are {steps:?} cube steps apart: a shade, not a hue"
            );
            for (role, color) in [
                ("ok", th.ok),
                ("warn", th.warn),
                ("err", th.err),
                ("done", th.done),
                ("merged", th.merged),
                ("dim", th.dim),
                ("muted", th.muted),
            ] {
                assert_ne!(th.root, color, "{name}: a root checkout reads as {role}");
                assert_ne!(th.worktree, color, "{name}: a worktree reads as {role}");
            }
        }
    }

    /// The accent has to stand out from the text it highlights, and the
    /// text from the muted tier under it, in every preset — including the
    /// grayscale one, which is the preset that moves those roles.
    #[test]
    fn accent_text_and_muted_stay_three_tiers_in_every_preset() {
        for name in THEMES {
            let th = Theme::preset(name);
            assert_ne!(th.accent, th.text, "{name}: a match highlight vanishes");
            assert_ne!(th.text, th.muted, "{name}: secondary text reads as primary");
            assert_ne!(th.muted, th.dim, "{name}: secondary text reads as a hint");
            assert_ne!(
                th.accent, th.muted,
                "{name}: an unfocused title reads as focused"
            );
        }
    }

    /// A merged pull request is purple in every preset, and that purple is
    /// nobody else's: not the terminated `special`, not the unread `done`
    /// a preset may move around, and not a status that would make a
    /// landed pull request look like it needs someone.
    #[test]
    fn merged_is_purple_and_its_own_color_in_every_preset() {
        for name in THEMES {
            let th = Theme::preset(name);
            assert_eq!(th.merged, Color::Indexed(135), "{name}: merged is purple");
            assert_ne!(th.merged, th.special, "{name}: merged reads as terminated");
            assert_ne!(th.merged, th.done, "{name}: merged reads as unread done");
            assert_ne!(th.merged, th.ok, "{name}: merged reads as plain success");
            assert_ne!(th.merged, th.warn, "{name}: merged reads as running");
            assert_ne!(th.merged, th.err, "{name}: merged reads as needs-feedback");
            assert_ne!(th.merged, th.accent, "{name}: merged reads as open");
            assert_ne!(th.merged, th.dim, "{name}: merged reads as a draft");
            // The sweep a merged checkout's branch name rides rests on that
            // same purple, and is nobody else's sweep.
            assert_eq!(
                th.merged_sweep[0], th.merged,
                "{name}: the sweep rests on merged"
            );
            assert_ne!(
                th.merged_sweep, th.warn_sweep,
                "{name}: the sweep reads as running"
            );
            assert_ne!(
                th.merged_sweep, th.err_sweep,
                "{name}: the sweep reads as needs-feedback"
            );
        }
    }

    /// `focus_tint` fills the surface keys land in, so it has to be seen —
    /// much darker and it reads as plain black (issue #6) — read as the
    /// accent rather than as gray, which every channel near one value is
    /// on a black window, and stay dark enough that dim text keeps its
    /// contrast: `#666` holds over 3.1:1 of the 3.7:1 it has on black.
    /// Mono is the one gray preset, and takes the one gray tint.
    #[test]
    fn focus_tint_is_visible_hued_and_dark() {
        // WCAG relative luminance.
        let luminance = |rgb: [u8; 3]| {
            let lin = |c: u8| {
                let c = f64::from(c) / 255.0;
                if c <= 0.04045 {
                    c / 12.92
                } else {
                    ((c + 0.055) / 1.055).powf(2.4)
                }
            };
            0.2126 * lin(rgb[0]) + 0.7152 * lin(rgb[1]) + 0.0722 * lin(rgb[2])
        };
        for name in THEMES {
            let th = Theme::preset(name);
            let Color::Rgb(r, g, b) = th.focus_tint else {
                panic!("{name}: focus_tint must be truecolor RGB");
            };
            let (hi, lo) = (r.max(g).max(b), r.min(g).min(b));
            assert!(
                hi >= 22,
                "{name}: focus_tint {:?} reads as black",
                (r, g, b)
            );
            assert!(
                luminance([r, g, b]) <= 0.009,
                "{name}: focus_tint {:?} costs dim text its contrast",
                (r, g, b)
            );
            if *name == "mono" {
                assert_eq!(hi, lo, "mono: a gray preset takes a gray tint");
            } else {
                assert!(
                    hi - lo >= 15,
                    "{name}: focus_tint {:?} reads as gray",
                    (r, g, b)
                );
            }
        }
    }
}
