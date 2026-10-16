//! First-run splash: a procedurally animated quazar filling the body while
//! the project tree is empty. Every cell samples a tilted accretion disk,
//! a luminous core and twin jets with outward-moving pulses. A glyph and
//! color ramp renders their intensity, with stars twinkling behind them and the
//! wordmark materializing in a carved-out band the dust never paints.
//! Uses the active Omarchy palette when available, otherwise indexed ramps.
//!
//! The event loop ticks a repaint every [`FRAME`] while [`App::splash_active`]
//! holds; the scene itself is a pure function of elapsed time, so a missed
//! frame skips ahead instead of stuttering. The sky on its own
//! ([`draw_sky`]) is also what the empty GRID's welcome is drawn over,
//! ticked while [`App::welcome_active`] holds.

use crate::app::{App, Focus, HitTarget};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use std::time::Duration;

/// Repaint cadence while the splash is up.
pub const FRAME: Duration = Duration::from_millis(100);

/// Seconds to fade the sky in from black. Also hides the splash flashing
/// briefly on every launch before the daemon's first tree snapshot lands.
const FADE_IN: f32 = 1.2;

/// Glyph ramp, thin dust -> bright core. Single-width chars only.
const RAMP: &[char] = &['.', ':', '·', '+', '*', '*', 'o', '@'];
/// 256-color ramps: warm accretion disk, cool jets, both ending in white.
const DISK: &[u8] = &[54, 55, 92, 129, 169, 215, 229, 231];
const JET: &[u8] = &[17, 18, 24, 31, 38, 81, 159, 231];
/// Wordmark gradient, swept left to right.
const MARK: &[u8] = &[99, 105, 141, 177, 213, 219];

/// 5-row block bitmaps for Q U A Z A R.
const LETTERS: &[&[&str; 5]] = &[
    &[".###.", "#...#", "#...#", "#..##", ".####"],
    &["#..#", "#..#", "#..#", "#..#", ".##."],
    &[".##.", "#..#", "####", "#..#", "#..#"],
    &["####", "...#", "..#.", ".#..", "####"],
    &[".##.", "#..#", "####", "#..#", "#..#"],
    &["###.", "#..#", "###.", "#.#.", "#..#"],
];

fn hash(x: i32, y: i32, salt: u32) -> u32 {
    let mut h = (x as u32).wrapping_mul(374_761_393)
        ^ (y as u32).wrapping_mul(668_265_263)
        ^ salt.wrapping_mul(2_246_822_519);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^ (h >> 16)
}

fn hash01(x: i32, y: i32, salt: u32) -> f32 {
    (hash(x, y, salt) & 0xffff) as f32 / 65535.0
}

/// One octave of smooth 2D value noise.
fn vnoise(x: f32, y: f32, salt: u32) -> f32 {
    let (xi, yi) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (x - x.floor(), y - y.floor());
    let sx = fx * fx * (3.0 - 2.0 * fx);
    let sy = fy * fy * (3.0 - 2.0 * fy);
    let a = hash01(xi, yi, salt);
    let b = hash01(xi + 1, yi, salt);
    let c = hash01(xi, yi + 1, salt);
    let d = hash01(xi + 1, yi + 1, salt);
    a + (b - a) * sx + (c - a) * sy + (a - b - c + d) * sx * sy
}

/// Stylized quazar intensity and whether the cool jet palette dominates.
/// The fixed tilt keeps the axis stable while disk texture rotates and
/// pulses travel away from the core along both jets. No particle state.
fn quazar_sample(dx: f32, dy: f32, t: f32) -> (f32, bool) {
    let (s, c) = 0.25_f32.sin_cos();
    let x = dx * c + dy * s;
    let y = -dx * s + dy * c;
    // Projected minor/major axis ratio: larger exposes more disk surface
    // to the viewer without changing its on-screen rotation or jet axis.
    let disk_y = y / 0.45;
    let r = x.hypot(disk_y);
    let angle = disk_y.atan2(x) - t * 0.35;
    let grain = vnoise(r * angle.cos() * 4.0, r * angle.sin() * 4.0, 991);
    let bands = 0.75 + 0.25 * (r * 14.0 - angle * 2.0).sin();
    let disk = (-(r - 0.85).powi(2) * 2.8).exp() * bands * (0.65 + 0.35 * grain);
    let core = (-(x * x + y * y) * 32.0).exp();
    let distance = y.abs();
    let width = 0.07 + distance * 0.065;
    let pulse = 0.7 + 0.3 * (distance * 9.0 - t * 3.0).cos();
    let jet = (-(x / width).powi(2)).exp() * (-distance * 0.48).exp() * pulse;
    ((disk + core + jet).clamp(0.0, 1.0), jet > disk)
}

/// Seconds into the scene that started at `epoch` — or, with animations
/// off, a moment well past the fade-in: the event loop doesn't tick a
/// still scene, so it holds one finished frame instead of whatever
/// instant a stray redraw lands on.
pub fn scene_time(app: &App, epoch: std::time::Instant) -> f32 {
    if app.chrome.animations {
        epoch.elapsed().as_secs_f32()
    } else {
        60.0
    }
}

/// How far the scene has faded in from black `t` seconds in: 0 -> 1 over
/// [`FADE_IN`], eased at both ends.
fn fade_at(t: f32) -> f32 {
    let raw = (t / FADE_IN).clamp(0.0, 1.0);
    raw * raw * (3.0 - 2.0 * raw)
}

/// The wordmark's color at `u` (0 -> 1) across it: the gradient, with the
/// slow shine sweeping through once the scene has faded in.
fn mark_color(u: f32, t: f32, fade: f32, th: crate::theme::Theme) -> Color {
    if th.background.is_some() {
        let shine = ((u * 5.0 - t * 1.4).sin() * 0.5 + 0.5) * fade;
        return crate::theme::blend(th.accent, th.text, shine * 0.65);
    }
    let shine = (u * 5.0 - t * 1.4).sin() > 0.93;
    if shine && fade >= 1.0 {
        return Color::Indexed(231); // near-white glint
    }
    let gi = (u * (MARK.len() as f32 - 1.0)).round() as usize;
    Color::Indexed(MARK[gi])
}

/// `word` in the wordmark's gradient and shine, one cell per letter: the
/// name where there is no room for the block letters, or no call for
/// them.
pub fn wordmark_word(word: &str, t: f32, th: crate::theme::Theme) -> Vec<Span<'static>> {
    let fade = fade_at(t);
    let n = word.chars().count().max(1) as f32;
    word.chars()
        .enumerate()
        .map(|(i, ch)| {
            Span::styled(
                ch.to_string(),
                Style::default()
                    .fg(mark_color(i as f32 / n, t, fade, th))
                    .add_modifier(Modifier::BOLD),
            )
        })
        .collect()
}

/// One wordmark row as per-cell spans: gradient across the word, a slow
/// shine sweeping through, and the blocks materializing from static
/// (`░` -> `▒` -> `█`) while the scene fades in.
fn wordmark_line(row: usize, t: f32, fade: f32, th: crate::theme::Theme) -> Line<'static> {
    let width: usize = LETTERS.iter().map(|l| l[0].len()).sum::<usize>() + 2 * (LETTERS.len() - 1);
    let block = if fade < 0.5 {
        "░"
    } else if fade < 0.85 {
        "▒"
    } else {
        "█"
    };
    let mut spans = Vec::new();
    let mut col = 0usize;
    for (i, letter) in LETTERS.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
            col += 2;
        }
        for ch in letter[row].chars() {
            if ch == '#' {
                let u = col as f32 / width as f32;
                spans.push(Span::styled(
                    block,
                    Style::default()
                        .fg(mark_color(u, t, fade, th))
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                spans.push(Span::raw(" "));
            }
            col += 1;
        }
    }
    Line::from(spans)
}

pub fn draw_splash(f: &mut Frame, app: &mut App, area: Rect) {
    let th = app.chrome.theme;
    if area.width < 8 || area.height < 4 {
        return;
    }
    let t = scene_time(app, app.chrome.splash_epoch);
    let fade = fade_at(t);

    // ---- text block: wordmark, tagline, key hints, bottom-anchored ----
    let big = area.width >= 50 && area.height >= 18;
    let mut lines: Vec<Line> = Vec::new();
    if big {
        for row in 0..5 {
            lines.push(wordmark_line(row, t, fade, th));
        }
    } else {
        lines.push(Line::from(vec![
            Span::styled("◆ ", Style::default().fg(th.accent)),
            Span::styled(
                "quazar",
                Style::default().fg(th.text).add_modifier(Modifier::BOLD),
            ),
        ]));
    }
    lines.push(Line::from(""));
    if area.width >= 47 {
        lines.push(Line::from(Span::styled(
            "your agents keep running, even when you leave",
            Style::default().fg(th.dim),
        )));
        lines.push(Line::from(""));
    }
    let key = |k: &str, label: &str| {
        vec![
            Span::styled(
                k.to_string(),
                Style::default().fg(th.accent).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!(" {label}"), Style::default().fg(th.dim)),
        ]
    };
    let mut hint = Vec::new();
    if !app.tree.has_projects() {
        // Started inside a repo, that repo is one key away; anywhere else
        // `o` browses for one. Nothing here asks for anything but a folder.
        match app.launch_repo_name() {
            Some(name) => {
                hint.extend(key("Enter", &format!("open {name}")));
                hint.push(Span::styled("   ·   ", Style::default().fg(th.dim)));
                hint.extend(key("o", "another folder"));
            }
            None => hint.extend(key("n / o", "create your first project")),
        }
        hint.push(Span::styled("   ·   ", Style::default().fg(th.dim)));
        hint.extend(key("?", "help"));
    } else if app.launcher.projects_closed {
        // Every tab closed: the projects are all still there, so `+`
        // lists them; Enter still opens the repo quazar was started in.
        let here = app.launcher.launch_repo.as_deref().and_then(|path| {
            app.tree
                .project_at_path(path)
                .map(|p| p.name.clone())
                .or_else(|| path.file_name().map(|n| n.to_string_lossy().into_owned()))
        });
        if let Some(name) = here {
            hint.extend(key("Enter", &format!("open {name}")));
            hint.push(Span::styled("   ·   ", Style::default().fg(th.dim)));
        }
        hint.extend(key("+", "your projects"));
        hint.push(Span::styled("   ·   ", Style::default().fg(th.dim)));
        hint.extend(key("o", "another folder"));
        hint.push(Span::styled("   ·   ", Style::default().fg(th.dim)));
        hint.extend(key("?", "help"));
    }
    lines.push(Line::from(hint));

    let block_w = (lines.iter().map(Line::width).max().unwrap_or(0) as u16).min(area.width);
    let block_h = (lines.len() as u16).min(area.height);
    let text = Rect {
        x: area.x + (area.width - block_w) / 2,
        y: area.y + area.height - block_h - u16::from(area.height > block_h),
        width: block_w,
        height: block_h,
    };

    draw_sky(f.buffer_mut(), area, text, t, th);
    f.render_widget(Paragraph::new(lines).centered(), text);
    // A click anywhere lands focus back on the (invisible) projects panel,
    // where `n` creates the first project.
    app.chrome
        .hits
        .push((area, HitTarget::PanelBg(Focus::Projects)));
}

/// The quazar and its starfield across `area`, `t` seconds into the
/// scene: the disk centered in the sky above `text` and fitted without
/// distorting its axes. Dust and stars stay off a band around `text` so words sit
/// on clear black. The first-run splash and the empty GRID's welcome
/// (`ui::launcher_view`) are both drawn over it.
pub fn draw_sky(buf: &mut Buffer, area: Rect, text: Rect, t: f32, th: crate::theme::Theme) {
    let theme = th.background.map(|_| th);
    let fade = fade_at(t);
    // ---- quazar centered in the sky above the text ----
    let above = text.y.saturating_sub(area.y).max(4);
    let cx = f32::from(area.x) + f32::from(area.width) / 2.0;
    let cy = f32::from(area.y) + f32::from(above) / 2.0;
    let (sx, sy) = quazar_cell_scale(area.width, above);
    // Text carve: rows the dust and stars never touch.
    let carve = Rect {
        x: text.x.saturating_sub(3),
        y: text.y.saturating_sub(1),
        width: text.width + 6,
        height: text.height + 2,
    }
    .intersection(area);

    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if x >= carve.left() && x < carve.right() && y >= carve.top() && y < carve.bottom() {
                continue;
            }
            let dx = (f32::from(x) - cx) * sx;
            let dy = (f32::from(y) - cy) * sy;
            let (intensity, jet) = quazar_sample(dx, dy, t);
            let d = intensity * fade;
            if d >= 0.055 {
                let v = ((d - 0.055) / 0.945).clamp(0.0, 1.0);
                let i = (v * (RAMP.len() as f32 - 1.0)).round() as usize;
                let color = theme
                    .map(|th| {
                        let hue = if jet { th.accent } else { th.merged };
                        if v < 0.5 {
                            crate::theme::blend(
                                th.background.unwrap_or(th.dim),
                                hue,
                                0.25 + v * 1.5,
                            )
                        } else {
                            crate::theme::blend(hue, th.text, (v - 0.5) * 2.0)
                        }
                    })
                    .unwrap_or(Color::Indexed(if jet { JET[i] } else { DISK[i] }));
                buf[(x, y)].set_char(RAMP[i]).set_fg(color);
                continue;
            }
            // Empty sky: sparse stars on their own twinkle phases, plus
            // the rare accent-colored sparkle.
            let h = hash(i32::from(x), i32::from(y), 12_345);
            if !h.is_multiple_of(53) {
                continue;
            }
            let phase = ((h >> 8) % 8) as f32 * 0.8;
            let tw = ((t * 2.5 + phase).sin() * 0.5 + 0.5) * fade;
            if tw <= 0.45 {
                continue;
            }
            if (h >> 4).is_multiple_of(111) {
                buf[(x, y)].set_char('+').set_fg(th.accent);
            } else if tw > 0.8 {
                buf[(x, y)]
                    .set_char('·')
                    .set_fg(theme.map_or(Color::Indexed(189), |th| th.muted));
            } else {
                buf[(x, y)]
                    .set_char('.')
                    .set_fg(theme.map_or(Color::Indexed(60), |th| th.dim));
            }
        }
    }
}

/// Fit a 5.6 by 4.8 scene using one physical scale. Terminal cells are
/// approximately twice as tall as wide. Independent stretch factors would
/// skew the tilted disk and make its perpendicular jets look misaligned.
fn quazar_cell_scale(cols: u16, rows: u16) -> (f32, f32) {
    let scale = (5.6 / f32::from(cols.max(1))).max(4.8 / (2.0 * f32::from(rows.max(1))));
    (scale, scale * 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn omarchy_animation_uses_palette_colors_without_changing_geometry() {
        let area = Rect::new(0, 0, 80, 30);
        let text = Rect::new(20, 24, 40, 5);
        let base = crate::theme::Theme::default();
        let theme = crate::theme::Theme {
            background: Some(Color::Rgb(10, 10, 10)),
            accent: Color::Rgb(80, 80, 80),
            merged: Color::Rgb(120, 120, 120),
            text: Color::Rgb(240, 240, 240),
            muted: Color::Rgb(160, 160, 160),
            dim: Color::Rgb(60, 60, 60),
            ..base
        };
        let mut original = Buffer::empty(area);
        let mut themed = Buffer::empty(area);
        draw_sky(&mut original, area, text, 5.0, base);
        draw_sky(&mut themed, area, text, 5.0, theme);
        for (a, b) in original.content.iter().zip(&themed.content) {
            assert_eq!(a.symbol(), b.symbol());
            if b.symbol() != " " {
                let Color::Rgb(r, g, b) = b.fg else {
                    panic!("fixed palette leaked into animation")
                };
                assert_eq!(r, g);
                assert_eq!(g, b);
            }
        }
        for span in wordmark_word("quazar", 5.0, theme) {
            assert!(matches!(span.style.fg, Some(Color::Rgb(..))));
        }
    }

    #[test]
    fn projected_disk_and_jets_stay_perpendicular_after_resize() {
        let (s, c) = 0.25_f32.sin_cos();
        for (cols, rows) in [(100, 46), (190, 30), (40, 60), (1, 1)] {
            let (sx, sy) = quazar_cell_scale(cols, rows);
            // Convert both scene axes to physical screen coordinates,
            // accounting for cells being twice as tall as wide.
            let disk = (c / sx, 2.0 * s / sy);
            let jet = (-s / sx, 2.0 * c / sy);
            let cosine =
                (disk.0 * jet.0 + disk.1 * jet.1) / (disk.0.hypot(disk.1) * jet.0.hypot(jet.1));
            assert!(cosine.abs() < 0.00001, "{cols}x{rows}: {cosine}");
            assert!(sx * f32::from(cols) >= 5.6 - 0.00001);
            assert!(sy * f32::from(rows) >= 4.8 - 0.00001);
        }
    }

    #[test]
    fn quazar_has_a_bright_core_and_two_narrow_jets() {
        assert_eq!(quazar_sample(0.0, 0.0, 0.0).0, 1.0);
        let (s, c) = 0.25_f32.sin_cos();
        for distance in [-1.8, 1.8] {
            let on_axis = quazar_sample(-distance * s, distance * c, 2.0);
            let off_axis = quazar_sample(c - distance * s, s + distance * c, 2.0);
            assert!(on_axis.1);
            assert!(on_axis.0 > off_axis.0 + 0.1);
        }
    }

    #[test]
    fn quazar_is_bounded_and_animates() {
        let mut changed = false;
        for x in -20..=20 {
            for y in -20..=20 {
                let a = quazar_sample(x as f32 / 10.0, y as f32 / 10.0, 0.0).0;
                let b = quazar_sample(x as f32 / 10.0, y as f32 / 10.0, 4.0).0;
                assert!((0.0..=1.0).contains(&a));
                assert!((0.0..=1.0).contains(&b));
                changed |= (a - b).abs() > 0.05;
            }
        }
        assert!(changed);
    }
}
