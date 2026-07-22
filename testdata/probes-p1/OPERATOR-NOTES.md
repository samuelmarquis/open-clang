# OPERATOR-NOTES — clang-probes-p1

*Hello, colleague. You're the Claude on sam-pc, automating render
passes through REAPER (and FL Studio if needed). I'm the Claude on
Sam's Mac building **open-clang**, an open drum-synthesis plugin. This
pack characterizes two commercial references by measured behavior:
**Waves Torque** (drum formant/pitch shifter) and **Image-Line
Vocodex** (Sam's preset: **`autovocoding`**). Your renders come back to
me for analysis. Clean-room rules: we measure input/output behavior and
read public docs; no disassembly, no resource extraction.*

## Ground rules (apply to every render)

1. **44.1 kHz project rate, 24-bit WAV renders, no dither, no
   resampling.** Render from t=0 with the probe placed at t=0 —
   sample alignment is data.
2. **Unity path**: no master FX, track faders at 0 dB, pan center,
   mono probes stay mono (stereo render is fine if the plugin is
   inherently stereo — note it if so).
3. **Never normalize or trim the renders.** Tails matter; latency
   matters. Render length = probe length + 2 s.
4. One plugin per chain. Note the exact plugin & shell versions
   (Torque appears via a Waves WaveShell — record WaveShell version
   too) in `versions.txt`.
5. **Record reported PDC**: what latency does REAPER report for the
   Torque standard component vs the Live component? (Manual claims
   32 samples @44.1k std, 0 live — verify and write it down.)
6. If anything is impossible (a parameter missing, a grid cell
   nonsensical), skip it and log the skip in `NOTES.md` rather than
   improvising silently. Improvisations welcome *in addition*, clearly
   named (`x__*` prefix).

## Naming grammar

```
renders/torque/<probe>__T<sign><cents>_F<hz>_Th<db>_S<ms>_<std|live>.wav
renders/torque/<probe>__j00.wav                      (bypass control)
renders/vocodex/<probe>__vdx-autovocoding[_<variant>].wav
```

Examples: `r01_kick_catsum__T-700_F300_Th-48_S15_std.wav`,
`p06_sweep_slow__vdx-autovocoding_mp0.wav`.
The filename is the settings sheet — there must be nothing to
cross-reference.

## Torque render matrix

**j00 bypass control** — all 20 probes through the Torque track with
the plugin **bypassed** (measures rig latency/level; 20 renders).

**Grid A — core pitch grid** (fixed: Th −48 dBFS, Speed 15 ms, std):

| axis | values |
|---|---|
| subjects | r01_kick_catsum, r02_snare_catsum, r04_kick_wayelm, p11_modal_stack, p12_glide_kick |
| Torque (cents) | −1200, −700, −200, +200, +700, +1200 |
| Focus (Hz) | 98, 300, 900 |

= 90 renders.

**Grid B — threshold** (fixed: T −700, F 300, S 15, std):
subjects {p02_dirac_steps, r02, r03_hat_catsum} × Th {−70, −48, −24,
−6} = 12 renders. (p02 is impulses stepping −60→−1 dBFS: this maps the
gate.)

**Grid C — speed** (fixed: F 300, Th −48, std):
subjects {p12, r02} × T {−700, +700} × Speed {15, 50} = 8 renders.

**Grid D — component/phase** (fixed: T −700, F 300, Th −48, S 15):
subjects {r01, p07_sweep_fast} × {std, live} = 4 renders. These get
diffed against dry for phase coherence, so alignment discipline
matters most here.

**Grid E — analytical** (fixed: F 300, Th −70, S 15, std):
subjects {p01_dirac, p03_click_lp2k, p06_sweep_slow, p07_sweep_fast} ×
T {−1200, +1200} = 8 renders. Th −70 so processing is always engaged.

Total ≈ 142. If Torque's UI exposes anything not in this grid
(mix/trim controls), leave at default and record the defaults in
`NOTES.md`.

## Vocodex renders (preset: `autovocoding`)

Vocodex may need to be hosted in FL Studio rather than REAPER — fine;
same ground rules and naming. The probe is the **modulator**; the
preset defines the carrier and routing. Do not "fix" the preset.

1. **As-is pass**: subjects {r01, r02, r04, r05_metalbirds,
   r06_kick_btrec, r07_amen_loop, p11, p12, p13_snare_synth,
   p06_sweep_slow, p02_dirac_steps} → `<probe>__vdx-autovocoding.wav`
   (11 renders). p06 through the preset is the band-distribution map;
   p02 maps its dynamics.
2. **Isolation variants** (if the preset's windows are editable —
   duplicate the preset, change ONE thing, render, revert):
   - `_bdlinear`: Band Distribution window flattened to the default
     linear curve — subjects {r01, p06} (2 renders).
   - `_mp0`: Modulator pitch shift window zeroed — subjects {r01, p06}
     (2 renders). This isolates the octave-offset trick.
3. **Metadata**: export/copy the `autovocoding` preset file itself if
   FL allows; screenshot Vocodex's main UI, the Band Distribution
   window, and the Modulator Pitch window; note FL Studio + Vocodex
   versions in `versions.txt`.

## QC gates before returning

- File count matches the matrix (or every miss is logged in NOTES.md).
- All renders 44.1 kHz, nonzero audio, no unexpected full-scale
  clipping (plugin-inherent overs are data — log, don't fix).
- Spot-check three j00 files: they should be sample-aligned,
  level-identical copies of the probes (any offset = rig latency —
  measure and note it).

## Return packaging

```
clang-probes-p1-return.zip
├── renders/torque/…  renders/vocodex/…
├── versions.txt      (Torque, WaveShell, REAPER, FL, Vocodex, OS)
├── NOTES.md          (skips, oddities, defaults, PDC readings, improvisations)
└── screenshots/      (Vocodex windows)
```

Taildrop it back to **smq** (this pack's origin machine):
`tailscale file cp clang-probes-p1-return.zip smq:`

Thanks. The measured docs these feed are
`open-clang/docs/research/02-torque-measured.md` and
`03-vocodex-measured.md` — your NOTES.md gets quoted there, so write it
like evidence.
