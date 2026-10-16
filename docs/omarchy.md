# Omarchy colors

Leave Settings → Appearance → Theme on `default` to follow Omarchy.
Quazar reads `~/.local/state/omarchy/current/theme/colors.toml`, the path
published by Omarchy. If that file is absent, older installations are supported
through `$XDG_CONFIG_HOME/omarchy/current/theme/colors.toml`, or
`~/.config/omarchy/current/theme/colors.toml` when XDG_CONFIG_HOME is unset.

The palette supplies UI text, accents, status colors, selection fills,
backgrounds, and the quazar animation. Focus fills are blended against the
theme background, so light themes do not get black focus panels. The Omarchy
background takes precedence over the existing Black Background setting.
Explicit colors emitted by child terminals are left unchanged.

While following Omarchy, Quazar checks for palette changes every two seconds.
An incomplete palette during a theme switch leaves the last valid colors in
place. At startup, a missing or incomplete palette uses the built-in default.
Selecting another named theme opts out. No settings or data paths are changed.
