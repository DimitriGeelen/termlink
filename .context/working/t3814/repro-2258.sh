#!/usr/bin/env bash
# T-3814: T-2258 live repro against the .107 hub on the deployed binary — large scratch topic
# (1500 posts), post then immediate re-read, 4 concurrent full reads while 20 posts land,
# plus framework:pickup reads (read only). The scratch topic is deleted afterwards.
set -u
H=192.168.10.107:9100
T="t3814-repro-$(date +%s)"
ms() { echo $(( ($(date +%s%N) - $1) / 1000000 )); }
rd() { local s=$(date +%s%N); timeout -s KILL 12 termlink channel state "$1" --hub "$H" > /dev/null 2>&1; echo "rc=$? $(ms $s)ms"; }
termlink channel post "$T" --ensure-topic --hub "$H" --payload "seed 0" > /dev/null 2>&1 || { echo "cannot create $T"; exit 2; }
for i in $(seq 1 1499); do termlink channel post "$T" --hub "$H" --payload "seed $i" > /dev/null 2>&1 & (( i % 25 == 0 )) && wait; done; wait
echo "hub binary: $(termlink fleet doctor --json 2>/dev/null | grep -o '"version":"[^"]*"' | head -1)"
echo "cold read ($T):                            $(rd "$T")"
termlink channel post "$T" --hub "$H" --payload "write-then-read" > /dev/null 2>&1
echo "post once, then immediate re-read:          $(rd "$T")"
( for i in $(seq 1 20); do termlink channel post "$T" --hub "$H" --payload "landing $i" > /dev/null 2>&1; done ) &
P=$!
for k in 1 2 3 4; do ( echo "concurrent read $k while 20 posts land:   $(rd "$T")" ) & done; wait
for k in 1 2 3; do echo "framework:pickup read $k:                   $(rd framework:pickup)"; done
termlink channel delete "$T" --yes --hub "$H" 2>&1 | tail -1
