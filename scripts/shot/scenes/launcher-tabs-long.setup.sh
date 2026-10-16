# launcher-tabs-narrow's projects plus two with long names, so a narrow header has more tabs than room
# and the MORE CHIP has something to stand in for.
. "$HERE/scenes/launcher.setup.sh"
for extra in agentsystemlabs-website quazar-marketing-docs; do
  mkdir -p "$WORK/$extra"
  git -C "$WORK/$extra" init -q -b main
  git -C "$WORK/$extra" -c user.name=shot -c user.email=shot@example.invalid commit -q --allow-empty -m "$extra"
done
INNER_BIN="$BIN"
cat > "$RUNTIME/quazar-long" <<WRAP
#!/bin/sh
if [ "\$1" = add ]; then
  "$REAL_BIN" add "$WORK/agentsystemlabs-website" >/dev/null
  "$REAL_BIN" add "$WORK/quazar-marketing-docs" >/dev/null
fi
exec "$INNER_BIN" "\$@"
WRAP
chmod +x "$RUNTIME/quazar-long"
BIN="$RUNTIME/quazar-long"
