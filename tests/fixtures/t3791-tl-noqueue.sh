#!/bin/bash
# T-3791 negative control: real termlink, but a queued post's JSON loses its `queued` field.
real=$(command -v termlink)
if [ "$1 $2" = "channel post" ] && printf '%s\n' "$@" | grep -qx p2-during-blip; then
  "$real" "$@" | jq -c 'del(.queued)'; exit "${PIPESTATUS[0]}"
fi
exec "$real" "$@"
