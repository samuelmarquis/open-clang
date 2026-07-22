# PATHS NOT TAKEN — open-clang

Deferrals, not rejections. Per entry: **what / why deferred / what
hides there / re-entry notes.**

## 001 — Free-bar archetype as a *clean/solo* voice (Batch 001) — REVERSED same day

**What:** free-free bar modal archetype (ratios 1, 2.756, 5.404, …)
as a clean solo resonator alongside membrane and plate.
**Why deferred:** Batch 001 verdict — "Not a fan of bar. Sounds like
stock ableton Corpus." The sparse clean inharmonic ratio stack is the
most instantly Corpus-flavored object in the linear family.
**Reversal (Sam, same day):** "You can leave bar […] I'm curious to
hear how it gets mangled with the nonlinearities — at this point it's
not doing anything good. We might kill it later but for now I think it
could evolve into something cool." Bar stays in the batch rotation
**on probation, as nonlinearity fodder** — what's dead is only the
ambition of bar-as-clean-voice. It rides Batches 003+ (rattle,
cascade) as a mangling subject; kill/keep decision deferred to those
verdicts.
**What hides there:** NESS Net1's rattling nonlinear string/bar
interconnections (research 01 §A.7); marimba-adjacent territory if the
project ever wants pitched percussion.
**Re-entry notes:** n/a — never fully left. This entry stands as the
record of a one-day deferral and why it bounced.

## 002 — Traditional pitch envelope alongside NL1 (Batch 002)

**What:** a classic explicit pitch envelope (909-style fixed
exponential settle) as a user control coexisting with NL1's
energy-tracked glide. Batch 002's FAKE render demonstrated the sound:
"distinctly … faster set down to the expected pitch."
**Why deferred:** Sam, on hearing the A/B: "I don't hate the sound of
it but I'm not attached to it either … no need to introduce this yet,
it'll just bloat complexity, keep it for late-stage."
**What hides there:** the fixed-settle character itself (useful for
genre-idiomatic kicks), velocity-decoupled pitch moves, and layering
both mechanisms (physical glide + stylized envelope) for exaggerated
hits.
**Re-entry notes:** trivially cheap in the modal engine (one more
frequency multiplier); re-enter at late-stage polish (post-M5) as a
"Pitch Env" section, default off, after the core macro story is
frozen.

## 003 — Listening-position stereo (x-stereo-pilot)

**What:** L/R as two listening positions on the same object (dual
mode-weight render). Physically honest; kick corr 0.88, clang 0.24.
**Why deferred:** Sam: "only mildly interesting. It's definitely
detuned width … I never use it [in Vocodex] and don't find it all
that interesting — I think we'll likely get more interesting stuff
out of decohering the modal bank left/right."
**What hides there:** it's nearly free and mono-safe; could return as
a subtle default under whatever the real stereo program becomes.
**Re-entry notes:** the stereo program's primary direction is now
**per-mode L/R decoherence** (micro-detune/phase/damping divergence —
opq's coherence control, drum-sized), scheduled with bracing/space
batches. Listening-position width re-enters, if ever, as seasoning
under that.

## 004 — Width (per-mode phase divergence), stereo prototype 1

**What:** param 22 "Width" — per-mode L/R phase-tap divergence,
frequency-ramped. Stereo round 1 prototype.
**Why deferred (killed):** survived round 1 on probation ("boring,
but keep for now"); executed after round 2 — Sam: "I think I can now
say 'kill width' with confidence, as all these other controls are far
more rewarding." Decohere/Mode Spread/Sub Rotate/floor-0 outcompeted
it on every material.
**What hides there:** nothing unique — the identical phase-tap math
survives inside **Sub Rotate** (inverse ramp, aimed at the sub),
which is the part of the mechanism that earned its keep.
**Re-entry notes:** param slot 22 retained (deprecated-inert, renamed
"(deprecated)") for ABI hygiene; if a future control wants the slot,
that's a release-boundary decision. DSP re-entry is trivial (unforce
the zero in state.rs) if taste reverses again.
