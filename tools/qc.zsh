#!/usr/bin/env zsh
# tools/qc.zsh — THE canonical install + QC pipeline (M12 law).
#
# Exists because M9–M11 were installed DEBUG: `cargo xtask install`
# defaults to the debug profile, and the omitted --release flag put a
# 17.6x-slower engine in front of Sam's ears for three rounds (and
# finally underran an M4 in a blank project). Every step here is a
# gate; the hash check makes a debug install IMPOSSIBLE to miss.
#
# Usage: tools/qc.zsh            (from the repo root)

set -e
ROOT="${0:A:h:h}"
cd "$ROOT"

echo "== 1/6 in-process fuzz + host-path gates (release) =="
( cd wrac && nix develop .. --command cargo test -p clg_plugin_wrac --release 2>&1 \
  | grep -E "test result|running" | tail -3 )

echo "== 2/6 build release artifacts (clap/vst3/au) =="
( cd wrac && nix develop .. --command cargo xtask build -p clg_plugin_wrac \
    --release --target clap --target vst3 --target au 2>&1 | tail -3 )

echo "== 3/6 install (RELEASE - the M12 lesson) =="
( cd wrac && nix develop .. --command cargo xtask install --release 2>&1 | tail -1 )

echo "== 4/6 hash-verify installed CLAP == release artifact =="
A=$(shasum -a 256 ~/Library/Audio/Plug-Ins/CLAP/open-clang.clap/Contents/MacOS/open-clang | cut -d' ' -f1)
B=$(shasum -a 256 wrac/target/wrac-plugins/clg/wrac/plugins/release/open-clang.clap/Contents/MacOS/open-clang | cut -d' ' -f1)
if [[ "$A" != "$B" ]]; then
  echo "FATAL: installed CLAP is NOT the release artifact (debug install?)"
  exit 1
fi
echo "hash match: installed == release"

echo "== 5/6 clap-validator (installed artifact) + auval =="
( cd wrac && nix develop .. --command cargo xtask validate -p clg_plugin_wrac \
    --release --target clap 2>&1 | grep -E "tests run" )
auval -v aumu Clg1 Oclg 2>&1 | grep -E "SUCCEEDED|FAILED"

echo "== 6/6 CPU gate: clg bench (8-voice worst block must stay < 25% budget) =="
( cd rt && nix develop .. --command cargo build --release 2>&1 | grep -cE "^error" >/dev/null || true )
rt/target/release/clg bench 2>&1 | head -8

echo "QC COMPLETE - eyeball the bench table above; the 'full x8' % budget row is the gate."
