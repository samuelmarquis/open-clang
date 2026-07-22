//! clg — offline renderer for the open-clang engine.
//! `clg render out.wav [--flag value ...]` — flags map 1:1 onto
//! EngineParams; `--brace X` applies the Bracing macro over them.

use clg_engine::{apply_brace_macro, apply_size_macro, Arch, Engine, EngineParams};

fn die(msg: &str) -> ! {
    eprintln!("clg: {msg}");
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 || args[0] != "render" {
        die("usage: clg render OUT.wav [--arch membrane|plate|bar] [--f0 HZ] [--vel 0..1] [--pos 0..1] [--listen-pos 0..1] [--stiff 0..1] [--t60 S] [--tilt X] [--n-axial N] [--glide ST] [--out-tilt DB_PER_OCT] [--casc 0..1] [--casc-tau S] [--casc-split MULT] [--casc-attack 0..1] [--casc-conserve] [--brace 0..1] [--sats wires|loose|trash] [--dust-level 0..1] [--width 0..1] [--decohere 0..1] [--stereo-floor 0..1] [--rattle-level 0..1] [--mode-spread 0..1] [--damp-asym 0..1] [--sub-rotate 0..1] [--size 0.4..2.5] [--vel-curve 0.25..4] [--dur S] [--sr HZ]");
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
                let (n, fs, t60s, seats, rests, levels): (u32, [f32; 4], [f32; 4], [f32; 4], [f32; 4], [f32; 4]) =
                    match args.get(i).map(|s| s.as_str()) {
                        Some("wires") => (2, [1900.0, 2700.0, 0.0, 0.0], [0.10, 0.08, 0.1, 0.1],
                                          [0.22, 0.61, 0.0, 0.0], [0.15, 0.22, 0.0, 0.0],
                                          [1.0, 0.8, 0.0, 0.0]),
                        Some("loose") => (1, [900.0, 0.0, 0.0, 0.0], [0.15, 0.1, 0.1, 0.1],
                                          [0.45, 0.0, 0.0, 0.0], [0.55, 0.0, 0.0, 0.0],
                                          [1.0, 0.0, 0.0, 0.0]),
                        Some("trash") => (3, [1300.0, 2100.0, 3400.0, 0.0], [0.12, 0.10, 0.07, 0.1],
                                          [0.18, 0.52, 0.80, 0.0], [0.30, 0.45, 0.20, 0.0],
                                          [1.0, 0.9, 0.7, 0.0]),
                        _ => die("--sats wires|loose|trash"),
                    };
                p.sat_count = n;
                p.sat_fs = fs;
                p.sat_t60 = t60s;
                p.sat_seat = seats;
                p.sat_rest = rests;
                p.sat_level = levels;
            }
            "--dust-level" => p.dust_level = val(),
            "--dust-thr" => p.dust_thr_db = val(),
            "--dust-follow" => p.dust_follow = val(),
            "--width" => p.width = val(),
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
    let n_atk = (sr * 0.0015) as usize;
    for k in 0..n_atk.min(buf_l.len()) {
        let ramp = k as f32 / n_atk as f32;
        buf_l[k] *= ramp;
        buf_r[k] *= ramp;
    }

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
        "clg: {} samples ({:.2} s), {} modes, {} contact-samples (L {} / R {}), peak {:.3} -> {}",
        buf_l.len(),
        buf_l.len() as f32 / sr,
        engine.n_modes(),
        engine.contacts(),
        cl,
        cr,
        peak,
        out_path
    );
}
