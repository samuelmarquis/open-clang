//! clg — offline renderer for the open-clang engine.
//! `clg render out.wav [--flag value ...]` — flags map 1:1 onto
//! EngineParams; `--brace X` applies the Bracing macro over them.

use clg_engine::{apply_brace_macro, apply_size_macro, Arch, Engine, EngineParams, Exciter};

fn die(msg: &str) -> ! {
    eprintln!("clg: {msg}");
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() && args[0] == "bench" {
        bench(&args[1..]);
        return;
    }
    if !args.is_empty() && args[0] == "hunt" {
        let n: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(2000);
        let seed: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
        hunt(n, seed);
        return;
    }
    if args.len() < 2 || args[0] != "render" {
        die("usage: clg render OUT.wav [--arch membrane|plate|bar] [--f0 HZ] [--vel 0..1] [--pos 0..1] [--listen-pos 0..1] [--stiff 0..1] [--t60 S] [--tilt X] [--n-axial N] [--glide ST] [--out-tilt DB_PER_OCT] [--casc 0..1] [--casc-tau S] [--casc-split MULT] [--casc-attack 0..1] [--casc-conserve] [--brace 0..1] [--sats wires|loose|trash] [--dust-level 0..1] [--exciter mallet|burst|buckling|raw|stick] [--ex-color 0..1] [--ex-time 0..1] [--decohere 0..1] [--stereo-floor 0..1] [--rattle-level 0..1] [--mode-spread 0..1] [--damp-asym 0..1] [--sub-rotate 0..1] [--rattle-casc 0..1] [--bounce 0..1] [--rattle-gap 0..1] [--gap-vel 0..1] [--rattle-tune ST] [--rattle-track 0..1] [--walk 0..1] [--bed-release 0..1] [--bed-source 0..1] [--bed-comb 0..1] [--bed-bright 0..1] [--cavity 0..1] [--cavity-tune HZ] [--head2-tune ST] [--head2-damp 0..1] [--wires 0..1] [--wire-tune HZ] [--wire-decay S] [--wire-throw 0..1] [--root-weight 0..1] [--size 0.4..2.5] [--vel-curve 0.25..4] [--dur S] [--sr HZ]");
    }
    let out_path = &args[1];
    let mut p = EngineParams::default();
    let mut brace: Option<f32> = None;
    let mut size: Option<f32> = None;
    let mut vel_curve = 1.0f32;
    let mut dur = 0.0f32; // 0 = auto (render until silent, cap 6 s)
    let mut sr = 44100.0f32;

    let mut i = 2;
    while i < args.len() {
        let flag = args[i].as_str();
        let mut val = || -> f32 {
            i += 1;
            args.get(i)
                .unwrap_or_else(|| die(&format!("{flag} needs a value")))
                .parse()
                .unwrap_or_else(|_| die(&format!("bad value for {flag}")))
        };
        match flag {
            "--arch" => {
                i += 1;
                p.arch = match args.get(i).map(|s| s.as_str()) {
                    Some("membrane") => Arch::Membrane,
                    Some("plate") => Arch::Plate,
                    Some("bar") => Arch::Bar,
                    _ => die("--arch membrane|plate|bar"),
                };
            }
            "--f0" => p.f0 = val(),
            "--vel" => p.velocity = val(),
            "--pos" => p.position = val(),
            "--listen-pos" => p.listen_pos = val(),
            "--stiff" => p.stiffness = val(),
            "--t60" => p.t60_base = val(),
            "--tilt" => p.tilt = val(),
            "--n-axial" => p.n_axial = val() as u32,
            "--glide" => p.glide_st = val(),
            "--out-tilt" => p.out_tilt_db_oct = val(),
            "--casc" => p.cascade_amt = val(),
            "--casc-tau" => p.cascade_tau = val(),
            "--casc-split" => p.cascade_split = val(),
            "--casc-attack" => p.cascade_attack = val(),
            "--casc-conserve" => p.cascade_conserve = true,
            "--casc-coherent" => p.cascade_coherent = true,
            "--brace" => brace = Some(val()),
            "--sats" => {
                i += 1;
                // M8 re-voicing: each satellite is a small modal OBJECT
                // (partial ratio/amp sets), not a sine — the one deliberate
                // baseline sound change of the round.
                let (n, fs, t60s, seats, rests, levels, pr, pa): (
                    u32, [f32; 4], [f32; 4], [f32; 4], [f32; 4], [f32; 4],
                    [[f32; 4]; 4], [[f32; 4]; 4],
                ) = match args.get(i).map(|s| s.as_str()) {
                    Some("wires") => (
                        2, [1900.0, 2700.0, 0.0, 0.0], [0.10, 0.08, 0.1, 0.1],
                        [0.22, 0.61, 0.0, 0.0], [0.15, 0.22, 0.0, 0.0],
                        [1.0, 0.8, 0.0, 0.0],
                        // bright inharmonic wire sets, 3 partials each
                        [[1.0, 1.53, 2.31, 0.0], [1.0, 1.71, 2.63, 0.0],
                         [1.0, 0.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0]],
                        [[1.0, 0.6, 0.35, 0.0], [1.0, 0.55, 0.3, 0.0],
                         [1.0, 0.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0]],
                    ),
                    Some("loose") => (
                        1, [900.0, 0.0, 0.0, 0.0], [0.15, 0.1, 0.1, 0.1],
                        [0.45, 0.0, 0.0, 0.0], [0.55, 0.0, 0.0, 0.0],
                        [1.0, 0.0, 0.0, 0.0],
                        // dull knocker + one overtone
                        [[1.0, 2.7, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0],
                         [1.0, 0.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0]],
                        [[1.0, 0.4, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0],
                         [1.0, 0.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0]],
                    ),
                    Some("trash") => (
                        3, [1300.0, 2100.0, 3400.0, 0.0], [0.12, 0.10, 0.07, 0.1],
                        [0.18, 0.52, 0.80, 0.0], [0.30, 0.45, 0.20, 0.0],
                        [1.0, 0.9, 0.7, 0.0],
                        // clattery junk: spread 3-4 partial sets
                        [[1.0, 1.34, 1.83, 2.51], [1.0, 1.47, 2.06, 2.9],
                         [1.0, 1.62, 2.24, 0.0], [1.0, 0.0, 0.0, 0.0]],
                        [[1.0, 0.7, 0.5, 0.35], [1.0, 0.65, 0.45, 0.3],
                         [1.0, 0.6, 0.4, 0.0], [1.0, 0.0, 0.0, 0.0]],
                    ),
                    _ => die("--sats wires|loose|trash"),
                };
                p.sat_count = n;
                p.sat_fs = fs;
                p.sat_t60 = t60s;
                p.sat_seat = seats;
                p.sat_rest = rests;
                p.sat_level = levels;
                p.sat_pr = pr;
                p.sat_pa = pa;
            }
            "--dust-level" => p.dust_level = val(),
            "--dust-thr" => p.dust_thr_db = val(),
            "--dust-follow" => p.dust_follow = val(),
            // M8 — the rattle control surface
            "--rattle-casc" => p.rattle_casc = val(),
            "--bounce" => p.bounce = val(),
            "--rattle-gap" => p.rattle_gap = val(),
            "--gap-vel" => p.gap_vel = val(),
            "--rattle-tune" => p.rattle_tune = val() / 12.0, // flag in SEMITONES
            "--rattle-track" => p.rattle_track = val(),
            "--walk" => p.walk = val(),
            "--bed-release" => p.bed_release = val(),
            "--bed-source" => p.bed_source = val(),
            "--bed-comb" => p.bed_comb = val(),
            "--bed-bright" => p.bed_bright = val(),
            // M10 — cavity + resonant head
            "--cavity" => p.cavity = val(),
            "--cavity-tune" => p.cavity_tune = val(),
            "--head2-tune" => p.head2_tune = val(), // semitones vs f0
            "--head2-damp" => p.head2_damp = val(),
            // M11 — the wire bank (Net1) + Root Weight
            "--wires" => p.wires = val(),
            "--wire-tune" => p.wire_tune = val(), // Hz, band center
            "--wire-decay" => p.wire_decay = val(), // seconds, T60
            "--wire-throw" => p.wire_throw = val(),
            "--root-weight" => p.root_weight = val(),
            "--exciter" => {
                i += 1;
                p.exciter = match args.get(i).map(|s| s.as_str()) {
                    Some("mallet") => Exciter::Mallet,
                    Some("burst") => Exciter::Burst,
                    Some("buckling") => Exciter::Buckling,
                    Some("raw") => Exciter::Raw,
                    Some("stick") => Exciter::Stick,
                    _ => die("--exciter mallet|burst|buckling|raw|stick"),
                };
            }
            "--ex-color" => p.ex_color = val(),
            "--ex-time" => p.ex_time = val(),
            "--decohere" => p.decohere = val(),
            "--stereo-floor" => p.stereo_floor = val(),
            "--rattle-level" => p.rattle_level = val(),
            "--mode-spread" => p.mode_spread = val(),
            "--damp-asym" => p.damp_asym = val(),
            "--sub-rotate" => p.sub_rotate = val(),
            "--size" => size = Some(val()),
            "--vel-curve" => vel_curve = val(),
            "--dur" => dur = val(),
            "--sr" => sr = val(),
            other => die(&format!("unknown flag {other}")),
        }
        i += 1;
    }
    if vel_curve != 1.0 {
        p.velocity = p.velocity.max(0.0).powf(vel_curve);
    }
    if let Some(b) = brace {
        apply_brace_macro(&mut p, b);
    }
    if let Some(s) = size {
        apply_size_macro(&mut p, s);
    }

    let mut engine = Engine::new(sr);
    engine.trigger(&p);

    let cap = if dur > 0.0 {
        (sr * dur) as usize
    } else {
        (sr * 6.0) as usize
    };
    let mut buf_l = vec![0.0f32; 0];
    let mut buf_r = vec![0.0f32; 0];
    let mut bl = [0.0f32; 256];
    let mut br = [0.0f32; 256];
    let mut run_peak = 0.0f32;
    while buf_l.len() < cap {
        let live = engine.process(&mut bl, &mut br);
        let bp = bl
            .iter()
            .chain(br.iter())
            .fold(0.0f32, |a, &x| a.max(x.abs()));
        run_peak = run_peak.max(bp);
        buf_l.extend_from_slice(&bl);
        buf_r.extend_from_slice(&br);
        if dur == 0.0 && (!live || (run_peak > 0.0 && bp < run_peak * 3.16e-5)) {
            break; // -90 dB below running peak
        }
    }
    // trim trailing silence (auto mode): last sample above -72 dB + 250 ms
    let peak = buf_l
        .iter()
        .chain(buf_r.iter())
        .fold(0.0f32, |a, &x| a.max(x.abs()));
    if dur == 0.0 && peak > 0.0 {
        let thr = peak * 2.51e-4;
        let last_l = buf_l.iter().rposition(|&x| x.abs() > thr).unwrap_or(0);
        let last_r = buf_r.iter().rposition(|&x| x.abs() > thr).unwrap_or(0);
        let end = (last_l.max(last_r) + (sr * 0.25) as usize).min(buf_l.len());
        buf_l.truncate(end);
        buf_r.truncate(end);
    }
    if peak > 0.0 {
        let g = 0.708 / peak;
        for x in buf_l.iter_mut().chain(buf_r.iter_mut()) {
            *x *= g;
        }
    }
    // M10: the batch-001-era 1.5 ms fade-in is GONE — it was erasing
    // sub-ms attack transients (the Stick tick lives entirely inside
    // it), so every CLI render under-reported the crack. The engine's
    // exciters all start from zero (raised-cosine onsets); Raw's t=0
    // impulse is the sound, not an artifact. The plugin path never had
    // this ramp — renders and Live now agree about attacks.

    // true stereo (STEREO v1); with width/decohere at 0 the channels are
    // bit-identical, i.e. the canonical mono voice
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: sr as u32,
        bits_per_sample: 24,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(out_path, spec)
        .unwrap_or_else(|e| die(&format!("cannot write {out_path}: {e}")));
    for (&xl, &xr) in buf_l.iter().zip(buf_r.iter()) {
        w.write_sample((xl.clamp(-1.0, 1.0) * 8_388_607.0) as i32).unwrap();
        w.write_sample((xr.clamp(-1.0, 1.0) * 8_388_607.0) as i32).unwrap();
    }
    w.finalize().unwrap();
    let (cl, cr) = engine.contacts_lr();
    eprintln!(
        "clg: {} samples ({:.2} s), {} modes, {} contact-samples (L {} / R {}), {} entries, peak {:.3} -> {}",
        buf_l.len(),
        buf_l.len() as f32 / sr,
        engine.n_modes(),
        engine.contacts(),
        cl,
        cr,
        engine.entries(),
        peak,
        out_path
    );
}

// ---------------------------------------------------------------------------
// M12 — `clg bench`: the CPU truth-teller. Fixed methodology so numbers are
// comparable across rounds (this table is a QC gate: see CLAUDE.md).
//
// Scenario: SNARE RECIPE v3 (the expensive patch — wires + cavity + bed +
// satellites + decohere + Stick), 64-frame blocks at 44.1k, 4 s per case,
// every voice re-triggered each second (staggered 25 ms) so the whole run
// stays in the loud/contact-heavy phase — the worst block is the one that
// causes underruns, so worst-block % of budget is the headline number.
// ---------------------------------------------------------------------------

fn bench_recipe_v3(vel: f32) -> EngineParams {
    let mut p = EngineParams::default();
    p.arch = Arch::Membrane;
    p.f0 = 190.0;
    p.velocity = vel;
    p.stiffness = 0.8;
    p.t60_base = 0.42;
    p.tilt = 0.6;
    p.out_tilt_db_oct = -6.0;
    // --sats wires preset (M8 voicing) + rattle level
    p.sat_count = 2;
    p.sat_fs = [1900.0, 2700.0, 0.0, 0.0];
    p.sat_t60 = [0.10, 0.08, 0.1, 0.1];
    p.sat_seat = [0.22, 0.61, 0.0, 0.0];
    p.sat_rest = [0.15, 0.22, 0.0, 0.0];
    p.sat_level = [1.0, 0.8, 0.0, 0.0];
    p.sat_pr = [
        [1.0, 1.53, 2.31, 0.0],
        [1.0, 1.71, 2.63, 0.0],
        [1.0, 0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0, 0.0],
    ];
    p.sat_pa = [
        [1.0, 0.6, 0.35, 0.0],
        [1.0, 0.55, 0.3, 0.0],
        [1.0, 0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0, 0.0],
    ];
    p.rattle_level = 0.35;
    p.decohere = 0.2;
    p.dust_level = 0.45;
    p.dust_thr_db = -55.0;
    p.dust_follow = 0.7;
    p.bed_source = 0.7;
    p.bed_comb = 0.3;
    p.bed_release = 0.55;
    p.bed_bright = 0.7;
    p.exciter = Exciter::Stick;
    p.ex_color = 0.35;
    p.ex_time = 0.3;
    p.cavity = 0.45;
    p.cavity_tune = 130.0;
    p.head2_tune = 7.0;
    p.head2_damp = 0.75;
    p.wires = 1.0;
    p.wire_tune = 1800.0;
    p.wire_decay = 0.45;
    p.wire_throw = 1.0;
    p.root_weight = 0.8;
    apply_brace_macro(&mut p, 0.15);
    p
}

struct BenchResult {
    ns_per_frame: f64,
    worst_block_us: f64,
    checksum: f32,
}

fn bench_case(p: &EngineParams, voices: usize, sr: f32, seconds: f32) -> BenchResult {
    const BLOCK: usize = 64;
    let mut engines: Vec<Engine> = (0..voices).map(|_| Engine::new(sr)).collect();
    let total_blocks = (seconds * sr) as usize / BLOCK;
    let retrig_blocks = (sr as usize) / BLOCK; // every ~1 s
    let stagger = (0.025 * sr) as usize / BLOCK; // 25 ms, in blocks
    let mut bl = [0.0f32; BLOCK];
    let mut br = [0.0f32; BLOCK];
    let mut mix = [0.0f32; BLOCK];
    let mut checksum = 0.0f32;
    let mut total_ns = 0u128;
    let mut worst_ns = 0u128;
    for b in 0..total_blocks {
        // deterministic trigger schedule (independent of liveness)
        for (vi, e) in engines.iter_mut().enumerate() {
            if b % retrig_blocks == (vi * stagger) % retrig_blocks {
                e.trigger(p);
            }
        }
        let t0 = std::time::Instant::now();
        for e in engines.iter_mut() {
            e.process(&mut bl, &mut br);
            for i in 0..BLOCK {
                mix[i] += bl[i] + br[i];
            }
        }
        let dt = t0.elapsed().as_nanos();
        total_ns += dt;
        worst_ns = worst_ns.max(dt);
        checksum += std::hint::black_box(mix.iter().sum::<f32>());
        mix = [0.0; BLOCK];
    }
    BenchResult {
        ns_per_frame: total_ns as f64 / (total_blocks * BLOCK) as f64,
        worst_block_us: worst_ns as f64 / 1000.0,
        checksum,
    }
}

fn bench(args: &[String]) {
    let mut sr = 44100.0f32;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--sr" => {
                i += 1;
                sr = args
                    .get(i)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(|| die("--sr needs a value"));
            }
            other => die(&format!("bench: unknown flag {other}")),
        }
        i += 1;
    }
    let budget_us = 64.0 / sr as f64 * 1e6; // one 64-frame block, in µs
    let p = bench_recipe_v3(0.95);

    // warmup (page-in, branch predictors, clocks)
    let _ = bench_case(&p, 1, sr, 1.0);

    eprintln!("clg bench — recipe v3, 64-frame blocks @ {sr} Hz");
    eprintln!("budget per block: {budget_us:.1} µs");
    eprintln!();
    eprintln!("{:<22} {:>10} {:>12} {:>14} {:>10}", "case", "ns/frame", "x-realtime", "worst-block µs", "% budget");
    let report = |name: &str, r: &BenchResult| {
        let rt = 1e9 / sr as f64 / r.ns_per_frame;
        eprintln!(
            "{:<22} {:>10.1} {:>12.1} {:>14.1} {:>9.1}%  (chk {:.2e})",
            name,
            r.ns_per_frame,
            rt,
            r.worst_block_us,
            r.worst_block_us / budget_us * 100.0,
            r.checksum
        );
    };
    for &v in &[1usize, 4, 8] {
        let r = bench_case(&p, v, sr, 4.0);
        report(&format!("full x{v}"), &r);
    }
    eprintln!();
    // component ablations (1 voice): what each mechanism costs
    let mut cases: Vec<(&str, EngineParams)> = Vec::new();
    let mut q = p;
    q.wires = 0.0;
    cases.push(("- wires", q));
    let mut q = p;
    q.cavity = 0.0;
    cases.push(("- cavity", q));
    let mut q = p;
    q.dust_level = 0.0;
    cases.push(("- bed/dust", q));
    let mut q = p;
    q.sat_count = 0;
    q.rattle_level = 0.0;
    cases.push(("- satellites", q));
    let mut q = p;
    q.decohere = 0.0;
    cases.push(("- decohere", q));
    let mut q = p;
    q.exciter = Exciter::Mallet;
    cases.push(("- stick (mallet)", q));
    let mut q = p;
    q.root_weight = 0.0;
    cases.push(("- root weight", q));
    let mut q = p;
    q.cascade_amt = 0.7;
    q.cascade_coherent = true;
    q.cascade_conserve = true;
    cases.push(("+ cascade 0.7", q));
    let mut q = p;
    q.n_axial = 12; // 144-mode bank: the dense-transect stress case
    cases.push(("+ modes 144", q));
    for (name, cp) in &cases {
        let r = bench_case(cp, 1, sr, 4.0);
        report(name, &r);
    }
}

// ---------------------------------------------------------------------------
// M12.1 — the scream hunter. Random configs biased toward Sam's report
// (plate + stick + wires + satellites, all else free), 4 s per config,
// detectors for BOUNDED eternal oscillation (the finiteness gate's blind
// spot): trailing output that never decays, near-Nyquist concentration,
// hard L/R asymmetry. Deterministic per (seed, index): every flagged
// config prints a `clg render` repro line.
// ---------------------------------------------------------------------------

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        // xorshift64*
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }
    fn f(&mut self) -> f32 {
        (self.next() >> 40) as f32 / (1u64 << 24) as f32
    }
    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.f()
    }
    fn log_range(&mut self, lo: f32, hi: f32) -> f32 {
        (lo.ln() + (hi.ln() - lo.ln()) * self.f()).exp()
    }
    fn chance(&mut self, p: f32) -> bool {
        self.f() < p
    }
}

fn hunt_sats(kind: u32, p: &mut EngineParams) {
    // mirrors the render presets (duplicated on purpose: the hunter must
    // not disturb the render path)
    match kind {
        0 => {
            p.sat_count = 2;
            p.sat_fs = [1900.0, 2700.0, 0.0, 0.0];
            p.sat_t60 = [0.10, 0.08, 0.1, 0.1];
            p.sat_seat = [0.22, 0.61, 0.0, 0.0];
            p.sat_rest = [0.15, 0.22, 0.0, 0.0];
            p.sat_level = [1.0, 0.8, 0.0, 0.0];
            p.sat_pr = [[1.0, 1.53, 2.31, 0.0], [1.0, 1.71, 2.63, 0.0],
                        [1.0, 0.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0]];
            p.sat_pa = [[1.0, 0.6, 0.35, 0.0], [1.0, 0.55, 0.3, 0.0],
                        [1.0, 0.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0]];
        }
        1 => {
            p.sat_count = 1;
            p.sat_fs = [900.0, 0.0, 0.0, 0.0];
            p.sat_t60 = [0.15, 0.1, 0.1, 0.1];
            p.sat_seat = [0.45, 0.0, 0.0, 0.0];
            p.sat_rest = [0.55, 0.0, 0.0, 0.0];
            p.sat_level = [1.0, 0.0, 0.0, 0.0];
            p.sat_pr = [[1.0, 2.7, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0],
                        [1.0, 0.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0]];
            p.sat_pa = [[1.0, 0.4, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0],
                        [1.0, 0.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0]];
        }
        _ => {
            p.sat_count = 3;
            p.sat_fs = [1300.0, 2100.0, 3400.0, 0.0];
            p.sat_t60 = [0.12, 0.10, 0.07, 0.1];
            p.sat_seat = [0.18, 0.52, 0.80, 0.0];
            p.sat_rest = [0.30, 0.45, 0.20, 0.0];
            p.sat_level = [1.0, 0.9, 0.7, 0.0];
            p.sat_pr = [[1.0, 1.34, 1.83, 2.51], [1.0, 1.47, 2.06, 2.9],
                        [1.0, 1.62, 2.24, 0.0], [1.0, 0.0, 0.0, 0.0]];
            p.sat_pa = [[1.0, 0.7, 0.5, 0.35], [1.0, 0.65, 0.45, 0.3],
                        [1.0, 0.6, 0.4, 0.0], [1.0, 0.0, 0.0, 0.0]];
        }
    }
}

fn hunt_config(rng: &mut Rng) -> (EngineParams, u32) {
    let mut p = EngineParams::default();
    p.arch = if rng.chance(0.6) {
        Arch::Plate
    } else if rng.chance(0.6) {
        Arch::Membrane
    } else {
        Arch::Bar
    };
    p.exciter = if rng.chance(0.6) {
        Exciter::Stick
    } else {
        match rng.next() % 4 {
            0 => Exciter::Mallet,
            1 => Exciter::Burst,
            2 => Exciter::Buckling,
            _ => Exciter::Raw,
        }
    };
    p.f0 = rng.log_range(30.0, 2000.0);
    p.velocity = rng.range(0.25, 1.0);
    p.position = rng.f();
    p.listen_pos = rng.f();
    p.stiffness = rng.f();
    p.t60_base = rng.log_range(0.05, 4.0);
    p.tilt = rng.f();
    p.out_tilt_db_oct = rng.range(-12.0, 6.0);
    p.glide_st = rng.range(0.0, 12.0);
    if rng.chance(0.5) {
        p.cascade_amt = rng.f();
        p.cascade_tau = rng.log_range(0.01, 0.3);
        p.cascade_split = rng.range(1.2, 3.0);
        p.cascade_attack = rng.f();
        p.cascade_conserve = rng.chance(0.7);
        p.cascade_coherent = rng.chance(0.5);
    }
    let sat_kind = (rng.next() % 3) as u32;
    hunt_sats(sat_kind, &mut p);
    p.rattle_level = rng.range(0.1, 1.0);
    p.rattle_casc = rng.f();
    p.bounce = rng.f();
    p.rattle_gap = rng.f();
    p.gap_vel = rng.f();
    p.rattle_tune = rng.range(-1.5, 1.5);
    p.rattle_track = rng.f();
    p.walk = rng.f();
    p.dust_level = rng.f();
    p.dust_thr_db = rng.range(-80.0, 0.0);
    p.dust_follow = rng.range(0.25, 2.0);
    p.bed_release = rng.f();
    p.bed_source = rng.f();
    p.bed_comb = rng.f();
    p.bed_bright = rng.f();
    if rng.chance(0.5) {
        p.cavity = rng.range(0.2, 1.0);
        p.cavity_tune = rng.log_range(40.0, 800.0);
        p.head2_tune = rng.range(-5.0, 12.0);
        p.head2_damp = rng.f();
    }
    p.wires = rng.range(0.3, 1.0);
    p.wire_tune = rng.log_range(800.0, 4000.0);
    p.wire_decay = rng.range(0.15, 1.2);
    p.wire_throw = rng.f();
    p.root_weight = rng.f();
    p.decohere = rng.f();
    p.stereo_floor = rng.f();
    p.mode_spread = rng.f();
    p.damp_asym = rng.f();
    p.sub_rotate = rng.f();
    p.ex_color = rng.f();
    p.ex_time = rng.f();
    if rng.chance(0.4) {
        apply_brace_macro(&mut p, rng.f());
    }
    (p, sat_kind)
}

fn repro_line(p: &EngineParams, sat_kind: u32, tag: &str) -> String {
    let arch = match p.arch {
        Arch::Membrane => "membrane",
        Arch::Plate => "plate",
        Arch::Bar => "bar",
    };
    let exc = match p.exciter {
        Exciter::Mallet => "mallet",
        Exciter::Burst => "burst",
        Exciter::Buckling => "buckling",
        Exciter::Raw => "raw",
        Exciter::Stick => "stick",
    };
    let sats = ["wires", "loose", "trash"][sat_kind as usize % 3];
    format!(
        "clg render {tag}.wav --arch {arch} --exciter {exc} --sats {sats} \
--f0 {:.2} --vel {:.3} --pos {:.3} --listen-pos {:.3} --stiff {:.3} \
--t60 {:.3} --tilt {:.3} --out-tilt {:.2} --glide {:.2} \
--casc {:.3} --casc-tau {:.4} --casc-split {:.3} --casc-attack {:.3} \
--rattle-level {:.3} --rattle-casc {:.3} --bounce {:.3} --rattle-gap {:.3} \
--gap-vel {:.3} --rattle-tune {:.2} --rattle-track {:.3} --walk {:.3} \
--dust-level {:.3} --dust-thr {:.1} --dust-follow {:.3} \
--bed-release {:.3} --bed-source {:.3} --bed-comb {:.3} --bed-bright {:.3} \
--cavity {:.3} --cavity-tune {:.2} --head2-tune {:.2} --head2-damp {:.3} \
--wires {:.3} --wire-tune {:.1} --wire-decay {:.3} --wire-throw {:.3} \
--root-weight {:.3} --decohere {:.3} --stereo-floor {:.3} \
--mode-spread {:.3} --damp-asym {:.3} --sub-rotate {:.3} \
--ex-color {:.3} --ex-time {:.3} --dur 4",
        p.f0, p.velocity, p.position, p.listen_pos, p.stiffness,
        p.t60_base, p.tilt, p.out_tilt_db_oct, p.glide_st,
        p.cascade_amt, p.cascade_tau.max(0.01), p.cascade_split.max(1.2),
        p.cascade_attack,
        p.rattle_level, p.rattle_casc, p.bounce, p.rattle_gap,
        p.gap_vel, p.rattle_tune * 12.0, p.rattle_track, p.walk,
        p.dust_level, p.dust_thr_db, p.dust_follow,
        p.bed_release, p.bed_source, p.bed_comb, p.bed_bright,
        p.cavity, p.cavity_tune.max(40.0), p.head2_tune, p.head2_damp,
        p.wires, p.wire_tune, p.wire_decay, p.wire_throw,
        p.root_weight, p.decohere, p.stereo_floor,
        p.mode_spread, p.damp_asym, p.sub_rotate,
        p.ex_color, p.ex_time,
    )
}

fn hunt(n: u64, seed: u64) {
    let sr = 44100.0f32;
    let total_samples = (sr * 4.0) as usize;
    let tail_samples = (sr * 0.5) as usize;
    let mut flagged = 0u64;
    for idx in 0..n {
        let mut rng = Rng(seed.wrapping_mul(0x9E3779B97F4A7C15) ^ (idx + 1));
        // warm the rng (xorshift starts correlated on sparse seeds)
        for _ in 0..8 {
            rng.next();
        }
        let (p, sat_kind) = hunt_config(&mut rng);
        let retrig = rng.chance(0.3);
        let mut retrigged = false;
        let (p2, sat_kind2) = hunt_config(&mut rng);
        let mut engine = Engine::new(sr);
        engine.trigger(&p);
        let mut bl = [0.0f32; 256];
        let mut br = [0.0f32; 256];
        let mut peak = 0.0f32;
        let mut done = 0usize;
        let mut tail_l = vec![0.0f32; 0];
        let mut tail_r = vec![0.0f32; 0];
        let mut live = true;
        let mut nonfinite = false;
        while done < total_samples {
            live = engine.process(&mut bl, &mut br);
            for (&l, &r) in bl.iter().zip(br.iter()) {
                if !(l.is_finite() && r.is_finite()) {
                    nonfinite = true;
                }
                peak = peak.max(l.abs().max(r.abs()));
            }
            done += bl.len();
            if retrig && !retrigged && done >= (sr * 0.5) as usize {
                engine.trigger(&p2); // the voice-steal path: mid-flight
                                     // retrigger with NEW params
                retrigged = true;
            }
            if done + 2 * tail_samples >= total_samples {
                tail_l.extend_from_slice(&bl);
                tail_r.extend_from_slice(&br);
            }
            if !live && !retrig {
                break; // healthy decay-to-sleep: config passes
            }
            if !live && retrig && done > (sr * 0.6) as usize {
                break;
            }
        }
        if !live {
            continue;
        }
        // detectors on the trailing windows: LAST 0.5 s vs the 0.5 s
        // before it — an eternal oscillation is FLAT (or rising); a
        // slow legit tail (t60 4 s ≈ −7.5 dB/s) falls between windows
        let nt = tail_l.len().min(tail_samples);
        if nt == 0 || tail_l.len() < 2 * nt {
            continue;
        }
        let rms = |s: &[f32]| (s.iter().map(|x| x * x).sum::<f32>() / s.len() as f32).sqrt();
        let (tl, tr) = (&tail_l[tail_l.len() - nt..], &tail_r[tail_r.len() - nt..]);
        let (pl, pr) = (
            &tail_l[tail_l.len() - 2 * nt..tail_l.len() - nt],
            &tail_r[tail_r.len() - 2 * nt..tail_r.len() - nt],
        );
        let (rms_l, rms_r) = (rms(tl), rms(tr));
        let trail = rms_l.max(rms_r);
        let prev = rms(pl).max(rms(pr));
        let sustain_db = 20.0 * (trail / peak.max(1e-12)).log10();
        let slope_db = 20.0 * (trail / prev.max(1e-12)).log10();
        if !nonfinite && (sustain_db <= -80.0 || slope_db < -2.5) {
            continue; // decayed, or still honestly decaying
        }
        // near-Nyquist concentration: sign-alternation ratio
        let mut alt = 0usize;
        let dom = if rms_l >= rms_r { tl } else { tr };
        for w in dom.windows(2) {
            if (w[0] >= 0.0) != (w[1] >= 0.0) {
                alt += 1;
            }
        }
        let alt_ratio = alt as f32 / (nt - 1) as f32;
        let asym_db = 20.0 * (rms_l.max(1e-12) / rms_r.max(1e-12)).log10();
        flagged += 1;
        println!(
            "HIT idx {idx} seed {seed}: sustain {sustain_db:.1} dB rel peak \
(peak {peak:.3e}, slope {slope_db:+.1} dB/win), alt-ratio {alt_ratio:.2}, L/R {asym_db:+.1} dB, \
nonfinite {nonfinite}, retrig {retrig}",
        );
        println!("  A: {}", repro_line(&p, sat_kind, &format!("scream_{idx}_a")));
        if retrig {
            println!("  B: {}", repro_line(&p2, sat_kind2, &format!("scream_{idx}_b")));
        }
    }
    println!("hunt: {n} configs, {flagged} flagged (seed {seed})");
}
