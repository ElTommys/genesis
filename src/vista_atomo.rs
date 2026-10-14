//! Microscopio cuántico (`genesis atomo <Z|símbolo>`): zoom continuo con escalas reales,
//! de la nube electrónica (10⁻¹⁰ m) al núcleo (10⁻¹⁴ m), a protones y neutrones
//! (10⁻¹⁵ m) y a sus quarks (<10⁻¹⁸ m).
//!
//! Los electrones NO se dibujan como planetas: cada punto de la nube se muestrea de
//! |ψ_nlm|², la probabilidad de encontrar al electrón ahí, con la carga efectiva de Slater.

use crate::azar::Rng;
use crate::cuantica::{orden_aufbau, Atomo};
use crate::elementos::{NOMBRES, SIMBOLOS};
use crate::fisica as fis;
use macroquad::prelude::*;

/// Un electrón con sus cuatro números cuánticos (n, l, mₗ, mₛ).
#[derive(Clone, Copy, PartialEq)]
struct Electron {
    n: u32,
    l: u32,
    ml: i32,
    /// +1 = spin arriba (mₛ = +½), −1 = spin abajo (mₛ = −½).
    ms: i32,
}

/// Reparte `e` electrones por Aufbau; dentro de cada subcapa aplica Hund:
/// primero uno por orbital con spin paralelo (↑), luego se aparean (↓). Pauli: nunca
/// dos electrones con los mismos cuatro números cuánticos.
fn electrones(e: u32) -> Vec<Electron> {
    let mut v = Vec::new();
    let mut quedan = e;
    for (n, l) in orden_aufbau() {
        if quedan == 0 {
            break;
        }
        let orb = 2 * l + 1;
        let k = quedan.min(2 * orb);
        for i in 0..k {
            let ml = (i % orb) as i32 - l as i32;
            let ms = if i < orb { 1 } else { -1 };
            v.push(Electron { n, l, ml, ms });
        }
        quedan -= k;
    }
    v
}

fn config_de(elec: &[Electron]) -> Vec<(u32, u32, u32)> {
    let mut c: Vec<(u32, u32, u32)> = Vec::new();
    for e in elec {
        match c.iter_mut().find(|x| x.0 == e.n && x.1 == e.l) {
            Some(x) => x.2 += 1,
            None => c.push((e.n, e.l, 1)),
        }
    }
    c
}

/// Nucleones empacados en una esfera de radio R = 1.2·A^(1/3) fm (posiciones en metros).
fn nucleo(protones: u32, neutrones: u32, rng: &mut Rng) -> Vec<([f64; 3], bool)> {
    let a = protones + neutrones;
    let r_nuc = fis::radio_nuclear_fm(a.max(1)) * 1e-15;
    let mut tipos: Vec<bool> = (0..a).map(|i| i < protones).collect();
    for i in (1..tipos.len()).rev() {
        tipos.swap(i, rng.rango(i + 1));
    }
    let oro = std::f64::consts::PI * (3.0 - 5f64.sqrt());
    (0..a as usize)
        .map(|i| {
            let f = (i as f64 + 0.5) / a as f64;
            let r = if a == 1 { 0.0 } else { r_nuc * f.cbrt() * 0.85 };
            let y = 1.0 - 2.0 * f;
            let rad = (1.0 - y * y).max(0.0).sqrt();
            let t = oro * i as f64 * 7.0;
            ([r * rad * t.cos(), r * y, r * rad * t.sin()], tipos[i])
        })
        .collect()
}

struct Nube {
    /// (posición en metros, color, índice del orbital).
    puntos: Vec<([f64; 3], Color, usize)>,
    orbitales: Vec<(u32, u32, i32)>,
}

fn color_l(l: u32, valencia: bool) -> Color {
    let a = if valencia { 0.55 } else { 0.22 };
    match l {
        0 => Color::new(0.35, 0.7, 1.0, a),
        1 => Color::new(0.35, 1.0, 0.55, a),
        2 => Color::new(1.0, 0.75, 0.3, a),
        _ => Color::new(1.0, 0.4, 0.9, a),
    }
}

fn construir_nube(z: u32, elec: &[Electron], rng: &mut Rng) -> Nube {
    let config = config_de(elec);
    let n_max = elec.iter().map(|e| e.n).max().unwrap_or(1);
    let mut orbitales: Vec<(u32, u32, i32)> = Vec::new();
    for e in elec {
        if !orbitales.contains(&(e.n, e.l, e.ml)) {
            orbitales.push((e.n, e.l, e.ml));
        }
    }
    let presupuesto = 16000usize;
    let por_orbital = (presupuesto / orbitales.len().max(1)).clamp(120, 900);
    let mut puntos = Vec::new();
    for (k, &(n, l, ml)) in orbitales.iter().enumerate() {
        let z_ef = fis::z_efectiva(z, &config, n, l);
        let ocup = elec.iter().filter(|e| e.n == n && e.l == l && e.ml == ml).count();
        let c = color_l(l, n == n_max);
        for p in fis::muestrear_orbital(n, l, ml, z_ef, por_orbital * ocup / 2 + por_orbital / 2, rng) {
            puntos.push(([p[0] * fis::A0, p[1] * fis::A0, p[2] * fis::A0], c, k));
        }
    }
    Nube { puntos, orbitales }
}

/// Valor "bonito" (1, 2, 5 × 10ⁿ) para la barra de escala, con su unidad.
fn escala_bonita(m: f64) -> (f64, String) {
    let e = m.log10().floor();
    let base = 10f64.powf(e);
    let f = m / base;
    let v = if f < 2.0 { 1.0 } else if f < 5.0 { 2.0 } else { 5.0 } * base;
    let (div, u) = if v >= 1e-9 { (1e-9, "nm") } else if v >= 1e-12 { (1e-12, "pm") } else if v >= 1e-15 { (1e-15, "fm") } else { (1e-18, "am") };
    (v, format!("{} {u}", fmt_num(v / div)))
}

fn fmt_num(x: f64) -> String {
    if (x - x.round()).abs() < 1e-9 { format!("{}", x.round() as i64) } else { format!("{x:.2}") }
}

fn superindice(n: i32) -> String {
    const S: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];
    let mut s = String::new();
    if n < 0 {
        s.push('⁻');
    }
    for c in n.unsigned_abs().to_string().chars() {
        s.push(S[c.to_digit(10).unwrap() as usize]);
    }
    s
}

fn cientifico(x: f64) -> String {
    if x == 0.0 {
        return "0".into();
    }
    let e = x.abs().log10().floor() as i32;
    format!("{:.3}×10{}", x / 10f64.powi(e), superindice(e))
}

struct Fotón {
    nm: f64,
    de_ev: f64,
    t: f32,
}

pub fn ver(z: u32) {
    let conf = Conf {
        window_title: "Genesis - microscopio cuantico".into(),
        window_width: 1400,
        window_height: 880,
        high_dpi: true,
        sample_count: 4,
        window_resizable: true,
        ..Default::default()
    };
    macroquad::Window::from_config(conf, bucle(z));
}

async fn bucle(z_inicial: u32) {
    let fuente = std::fs::read("/System/Library/Fonts/Supplemental/Arial Unicode.ttf")
        .ok()
        .and_then(|b| load_ttf_font_from_bytes(&b).ok());
    let txt = |s: &str, x: f32, y: f32, tam: u16, c: Color| {
        draw_text_ex(s, x, y, TextParams { font: fuente.as_ref(), font_size: tam, color: c, ..Default::default() });
    };

    let mut rng = Rng::nuevo(11);
    let mut z = z_inicial.clamp(1, 118);
    let mut extra_e: i32 = 0;
    let mut extra_n: i32 = 0;
    let (mut yaw, mut pitch) = (0.7f32, 0.35f32);
    let mut dist: f64 = 4e-10; // distancia de la cámara en metros
    let mut dist_objetivo = dist;
    let mut ultimo = mouse_position();
    let mut t = 0.0f32;
    let mut pausa = false;
    let mut ver_nube = true;
    let mut clave_actual = (0u32, 0i32, 0i32);
    let mut nube = Nube { puntos: vec![], orbitales: vec![] };
    let mut nuc: Vec<([f64; 3], bool)> = vec![];
    let mut elec: Vec<Electron> = vec![];
    let mut excitado: Option<(usize, Electron, f32)> = None; // (índice, estado original, tiempo)
    let mut foton: Option<Fotón> = None;
    let mut medicion: Option<(Vec<[f64; 3]>, f32)> = None;

    loop {
        // ── Controles ──
        if is_key_pressed(KeyCode::Right) { z = (z % 118) + 1; extra_e = 0; extra_n = 0; }
        if is_key_pressed(KeyCode::Left) { z = if z == 1 { 118 } else { z - 1 }; extra_e = 0; extra_n = 0; }
        if is_key_pressed(KeyCode::Up) { extra_e += 1; }
        if is_key_pressed(KeyCode::Down) { extra_e -= 1; }
        if is_key_pressed(KeyCode::N) { extra_n += 1; }
        if is_key_pressed(KeyCode::B) { extra_n -= 1; }
        if is_key_pressed(KeyCode::Space) { pausa = !pausa; }
        if is_key_pressed(KeyCode::O) { ver_nube = !ver_nube; }
        let base = Atomo::nuevo(z);
        let ne = (z as i32 + extra_e).clamp(0, 118) as u32;
        extra_e = ne as i32 - z as i32;
        let nn = (base.neutrones as i32 + extra_n).clamp(0, 200) as u32;
        extra_n = nn as i32 - base.neutrones as i32;
        let a = z + nn;
        let r_nuc = fis::radio_nuclear_fm(a) * 1e-15;
        if is_key_pressed(KeyCode::Key1) { dist_objetivo = 4e-10; }
        if is_key_pressed(KeyCode::Key2) { dist_objetivo = r_nuc * 4.0; }
        if is_key_pressed(KeyCode::Key3) { dist_objetivo = 3.5e-15; }
        if is_key_pressed(KeyCode::Key4) { dist_objetivo = 1.4e-15; }

        if clave_actual != (z, extra_e, extra_n) {
            clave_actual = (z, extra_e, extra_n);
            elec = electrones(ne);
            nube = construir_nube(z, &elec, &mut rng);
            nuc = nucleo(z, nn, &mut rng);
            excitado = None;
        }
        let config = config_de(&elec);

        // Excitar: el electrón más externo absorbe un fotón y salta (Δl = ±1); luego vuelve y emite.
        if is_key_pressed(KeyCode::E) && excitado.is_none() && !elec.is_empty() {
            let i = elec.len() - 1;
            let orig = elec[i];
            let l2 = if orig.l + 1 <= orig.n { orig.l + 1 } else { orig.l - 1 };
            elec[i] = Electron { n: orig.n + 1, l: l2, ml: 0, ms: orig.ms };
            excitado = Some((i, orig, t));
            nube = construir_nube(z, &elec, &mut rng);
        }
        if let Some((i, orig, t0)) = excitado {
            if t - t0 > 2.5 {
                let z1 = fis::z_efectiva(z, &config_de(&elec), elec[i].n, elec[i].l);
                let e_alto = fis::energia_nivel_ev(z1, elec[i].n);
                elec[i] = orig;
                let z0 = fis::z_efectiva(z, &config_de(&elec), orig.n, orig.l);
                let e_bajo = fis::energia_nivel_ev(z0, orig.n);
                let de = e_alto - e_bajo;
                foton = Some(Fotón { nm: fis::longitud_onda_nm(de), de_ev: de, t });
                excitado = None;
                nube = construir_nube(z, &elec, &mut rng);
            }
        }
        // Medir: la función de onda colapsa y cada electrón aparece en UN punto.
        if is_key_pressed(KeyCode::M) {
            let mut pos = Vec::new();
            for e in &elec {
                let k = nube.orbitales.iter().position(|o| *o == (e.n, e.l, e.ml)).unwrap_or(0);
                let cand: Vec<&([f64; 3], Color, usize)> = nube.puntos.iter().filter(|p| p.2 == k).collect();
                if !cand.is_empty() {
                    pos.push(cand[rng.rango(cand.len())].0);
                }
            }
            medicion = Some((pos, t));
        }

        let raton = mouse_position();
        if is_mouse_button_down(MouseButton::Left) && raton.0 > 460.0 {
            yaw += (raton.0 - ultimo.0) * 0.008;
            pitch = (pitch + (raton.1 - ultimo.1) * 0.008).clamp(-1.5, 1.5);
        }
        ultimo = raton;
        let rueda = mouse_wheel().1;
        if rueda != 0.0 {
            dist_objetivo *= 1.12f64.powf(-rueda.signum() as f64 * rueda.abs().min(3.0) as f64);
        }
        if is_key_down(KeyCode::Equal) { dist_objetivo *= 0.97; }
        if is_key_down(KeyCode::Minus) { dist_objetivo *= 1.03; }
        dist_objetivo = dist_objetivo.clamp(8e-16, 3e-9);
        dist *= (dist_objetivo / dist).powf(0.12);
        if !pausa {
            t += get_frame_time();
        }

        // ── Escena 3D ── (coordenadas de dibujo = metros × k: la cámara siempre a 10 unidades)
        clear_background(Color::from_rgba(3, 4, 10, 255));
        let k = 10.0 / dist;
        let v = |p: &[f64; 3]| vec3((p[0] * k) as f32, (p[1] * k) as f32, (p[2] * k) as f32);
        let ojo = vec3(pitch.cos() * yaw.cos(), pitch.sin(), pitch.cos() * yaw.sin()) * 10.0;
        set_camera(&Camera3D { position: ojo, target: Vec3::ZERO, up: Vec3::Y, z_near: 0.005, z_far: 4000.0, ..Default::default() });

        // Nube electrónica |ψ|²
        let tam_punto = 0.012 * (dist / 4e-10).max(0.02).min(1.0) as f32 * 10.0 / 10.0;
        if ver_nube {
            let parpadeo = (t * 3.0).sin() * 0.5 + 0.5;
            for (p, c, idx) in &nube.puntos {
                let q = v(p);
                if q.length() > 900.0 { continue; }
                let mut c = *c;
                if let Some((i, _, _)) = excitado {
                    let e = elec[i];
                    if nube.orbitales[*idx] == (e.n, e.l, e.ml) { c = Color::new(1.0, 1.0, 1.0, 0.4 + 0.4 * parpadeo); }
                }
                draw_cube(q, Vec3::splat(tam_punto.max(0.004) * q.length().max(1.0).min(30.0)), None, c);
            }
        }
        // Medición: electrones como puntos (colapso)
        if let Some((pos, t0)) = &medicion {
            if t - t0 < 2.0 {
                for p in pos { draw_sphere(v(p), 0.09, None, Color::new(1.0, 1.0, 0.6, 1.0 - (t - t0) / 2.0)); }
            }
        }
        // Núcleo
        let r_nuc_dibujo = (r_nuc * k) as f32;
        if r_nuc_dibujo < 0.04 {
            // Demasiado pequeño para verse a esta escala: un punto marcador (no a escala).
            draw_sphere(Vec3::ZERO, 0.04, None, Color::new(1.0, 0.85, 0.3, 0.9));
        } else {
            let r_nucleon = (fis::RADIO_PROTON_FM * 1e-15 * k) as f32;
            let adentro = dist < 6e-15;
            for (i, (p, es_proton)) in nuc.iter().enumerate() {
                let c0 = if *es_proton { Color::new(0.92, 0.25, 0.2, 1.0) } else { Color::new(0.66, 0.67, 0.72, 1.0) };
                let c = if adentro { Color::new(c0.r, c0.g, c0.b, 0.13) } else { c0 };
                // Los nucleones vibran (agitación cuántica, principio de incertidumbre).
                let j = (t * 6.0 + i as f32 * 1.7).sin() * 0.04 * r_nucleon;
                let q = v(p) + vec3(j, (t * 5.0 + i as f32).cos() * 0.04 * r_nucleon, 0.0);
                draw_sphere(q, r_nucleon * 0.92, None, c);
                if adentro {
                    // Quarks: protón = u u d, neutrón = u d d. Carga de color rojo/verde/azul,
                    // que se intercambia por gluones (por eso la suma siempre es "blanca").
                    let sabores = if *es_proton { ["u", "u", "d"] } else { ["u", "d", "d"] };
                    let rot = ((t * 2.2 + i as f32) as usize) % 3;
                    let colores = [Color::new(1.0, 0.2, 0.2, 1.0), Color::new(0.2, 1.0, 0.3, 1.0), Color::new(0.3, 0.4, 1.0, 1.0)];
                    let mut qs = [Vec3::ZERO; 3];
                    for qk in 0..3 {
                        let ang = t * 1.3 + qk as f32 * std::f32::consts::TAU / 3.0 + i as f32;
                        let w = (t * 4.1 + qk as f32 * 2.0 + i as f32).sin() * 0.15;
                        qs[qk] = q + vec3(ang.cos(), w, ang.sin()) * (r_nucleon * 0.45);
                        draw_sphere(qs[qk], r_nucleon * 0.11, None, colores[(qk + rot) % 3]);
                        let _ = sabores[qk];
                    }
                    // Gluones: tubos de flujo de la fuerza fuerte entre quarks (ondulados).
                    for (x, y) in [(0, 1), (1, 2), (2, 0)] {
                        let mut prev = qs[x];
                        for s in 1..=12 {
                            let f = s as f32 / 12.0;
                            let onda = (f * 18.0 + t * 12.0).sin() * r_nucleon * 0.04;
                            let p = qs[x].lerp(qs[y], f) + vec3(0.0, onda, onda);
                            draw_line_3d(prev, p, Color::new(1.0, 0.85, 0.2, 0.8));
                            prev = p;
                        }
                    }
                }
            }
        }
        // Fotón emitido: frente de onda que se aleja, del color de su longitud de onda.
        if let Some(f) = &foton {
            let edad = t - f.t;
            if edad < 3.0 {
                let (r, g, b) = fis::color_visible(f.nm).unwrap_or((0.8, 0.6, 1.0));
                let rad = 0.5 + edad * 6.0;
                draw_sphere_wires(Vec3::ZERO, rad, None, Color::new(r, g, b, (1.0 - edad / 3.0) * 0.5));
            }
        }

        // ── Panel 2D ──
        set_default_camera();
        let (w, h) = (screen_width(), screen_height());
        draw_rectangle(0.0, 0.0, 450.0, h, Color::from_rgba(8, 10, 22, 235));
        let mut y = 40.0;
        let carga = z as i32 - ne as i32;
        let sim = SIMBOLOS[(z - 1) as usize];
        let titulo = match carga {
            0 => format!("{} · {sim}", NOMBRES[(z - 1) as usize]),
            c if c > 0 => format!("Catión {sim}{}⁺", if c > 1 { superindice(c) } else { String::new() }),
            c => format!("Anión {sim}{}⁻", if c < -1 { superindice(-c) } else { String::new() }),
        };
        txt(&titulo, 20.0, y, 32, WHITE);
        y += 28.0;
        txt(&format!("Z = {z}   A = {a}   isótopo {sim}-{a}   {}", if carga == 0 { "átomo neutro" } else { "ion" }), 20.0, y, 17, LIGHTGRAY);
        y += 30.0;
        let fila = |y: &mut f32, c: Color, s: String| {
            draw_circle(28.0, *y - 6.0, 6.0, c);
            txt(&s, 42.0, *y, 17, LIGHTGRAY);
            *y += 22.0;
        };
        fila(&mut y, Color::new(0.92, 0.25, 0.2, 1.0), format!("{z} protones (uud)  carga +1  {:.6} u", fis::M_PROTON_U));
        fila(&mut y, Color::new(0.66, 0.67, 0.72, 1.0), format!("{nn} neutrones (udd)  carga 0  {:.6} u", fis::M_NEUTRON_U));
        fila(&mut y, Color::new(0.35, 0.7, 1.0, 1.0), format!("{ne} electrones  carga −1  {:.6} u", fis::M_ELECTRON_U));
        txt(&format!("carga neta {carga:+}e = {} C", cientifico(carga as f64 * fis::E_CARGA)), 20.0, y, 17, if carga == 0 { GREEN } else { ORANGE });
        y += 30.0;

        // Núcleo y E = mc²
        txt("Núcleo · fuerza nuclear fuerte", 20.0, y, 20, WHITE);
        y += 22.0;
        let b = fis::energia_enlace_mev(z, a);
        txt(&format!("radio R = 1.2·A^⅓ = {:.2} fm ({} m)", fis::radio_nuclear_fm(a), cientifico(r_nuc)), 20.0, y, 16, LIGHTGRAY);
        y += 20.0;
        txt(&format!("energía de enlace B = {b:.1} MeV  ({:.2} MeV/nucleón)", if a > 0 { b / a as f64 } else { 0.0 }), 20.0, y, 16, LIGHTGRAY);
        y += 20.0;
        txt(&format!("defecto de masa Δm = B/c² = {:.4} u", fis::defecto_masa_u(z, a)), 20.0, y, 16, LIGHTGRAY);
        y += 20.0;
        let m_at = fis::masa_atomica_u(z, a);
        txt(&format!("masa atómica {m_at:.4} u  ·  masa molar {m_at:.4} g/mol"), 20.0, y, 16, LIGHTGRAY);
        y += 20.0;
        txt(&format!("1 mol = {} átomos = {m_at:.3} g", cientifico(fis::NA)), 20.0, y, 16, LIGHTGRAY);
        y += 20.0;
        let pct_nucleo = (z as f64 * fis::M_PROTON_U + nn as f64 * fis::M_NEUTRON_U) / m_at.max(1e-9) * 100.0;
        txt(&format!("el núcleo tiene el {:.3}% de la masa", pct_nucleo.min(100.0)), 20.0, y, 16, GRAY);
        y += 30.0;

        // Corteza electrónica
        txt("Corteza electrónica · ψ y |ψ|²", 20.0, y, 20, WHITE);
        y += 22.0;
        let conf_txt: String = config.iter().map(|(n, l, e)| format!("{n}{}{} ", ['s', 'p', 'd', 'f'][*l as usize], superindice(*e as i32))).collect();
        txt(&conf_txt, 20.0, y, 16, Color::new(0.6, 0.85, 1.0, 1.0));
        y += 20.0;
        if let Some(&(n, l, _)) = config.last() {
            let ze = fis::z_efectiva(z, &config, n, l);
            let ei = -fis::energia_nivel_ev(ze, n);
            let r_pico = fis::radio_mas_probable(n, l, ze) * fis::A0;
            txt(&format!("valencia {n}{}: Z_ef (Slater) = {ze:.2}", ['s', 'p', 'd', 'f'][l as usize]), 20.0, y, 16, LIGHTGRAY);
            y += 20.0;
            txt(&format!("E_n = −13.6·Z_ef²/n² ≈ {:.1} eV  ·  r más probable {:.0} pm", -ei, r_pico * 1e12), 20.0, y, 16, LIGHTGRAY);
            y += 20.0;
        }
        let mut desap = 0u32;
        for &(n, l, _) in &config {
            for ml in -(l as i32)..=l as i32 {
                let s: i32 = elec.iter().filter(|e| e.n == n && e.l == l && e.ml == ml).map(|e| e.ms).sum();
                desap += s.unsigned_abs();
            }
        }
        txt(&format!("spin total S = {}  ·  {}", desap as f32 / 2.0, if desap > 0 { "paramagnético" } else { "diamagnético" }), 20.0, y, 16, LIGHTGRAY);
        y += 26.0;

        // Diagrama de orbitales: cada caja es un mₗ; flechas = spin (Pauli + Hund).
        let filas: Vec<(u32, u32)> = config.iter().map(|c| (c.0, c.1)).collect();
        let max_filas = ((h - y - 140.0) / 26.0).max(1.0) as usize;
        let ini = filas.len().saturating_sub(max_filas);
        if ini > 0 {
            txt(&format!("… {ini} subcapas internas llenas", ), 20.0, y, 14, GRAY);
            y += 18.0;
        }
        for &(n, l) in &filas[ini..] {
            txt(&format!("{n}{}", ['s', 'p', 'd', 'f'][l as usize]), 20.0, y + 6.0, 17, LIGHTGRAY);
            for (j, ml) in (-(l as i32)..=l as i32).enumerate() {
                let x = 62.0 + j as f32 * 50.0;
                draw_rectangle_lines(x, y - 11.0, 46.0, 24.0, 1.5, Color::from_rgba(110, 120, 170, 255));
                let mut s = String::new();
                for e in elec.iter().filter(|e| e.n == n && e.l == l && e.ml == ml) {
                    s.push(if e.ms > 0 { '↑' } else { '↓' });
                }
                txt(&s, x + 10.0, y + 7.0, 20, Color::new(0.5, 0.9, 1.0, 1.0));
            }
            y += 26.0;
        }

        // Fotón
        if let Some(f) = &foton {
            let (r, g, bb) = fis::color_visible(f.nm).unwrap_or((0.8, 0.6, 1.0));
            txt(&format!("fotón emitido: ΔE = {:.2} eV → λ = hc/ΔE = {:.0} nm ({})", f.de_ev, f.nm, fis::region_espectro(f.nm)), 20.0, h - 96.0, 15, Color::new(r, g, bb, 1.0));
        }

        // Escala actual y analogía
        let nivel = if dist > 5e-11 { "nube electrónica · 10⁻¹⁰ m" } else if dist > 2e-14 { "entre la nube y el núcleo (casi todo es vacío)" } else if dist > 5e-15 { "núcleo · 10⁻¹⁴ m" } else { "protón/neutrón 10⁻¹⁵ m · quarks < 10⁻¹⁸ m" };
        txt(&format!("Escala: {nivel}"), 470.0, 34.0, 22, WHITE);
        txt("Si un protón midiera 10 cm, el átomo mediría 10 km y un quark o un electrón menos de 0.1 mm.", 470.0, 58.0, 15, GRAY);
        if r_nuc_dibujo < 0.04 {
            txt(&format!("· el punto dorado marca el núcleo: es {:.0} veces más pequeño que el átomo", 1e-10 / r_nuc), 470.0, 80.0, 15, Color::new(1.0, 0.85, 0.3, 1.0));
        }
        if dist < 6e-15 {
            txt("quarks: rojo/verde/azul = carga de color · líneas doradas = gluones (fuerza fuerte) · tamaño de quarks exagerado", 470.0, 80.0, 15, Color::new(1.0, 0.85, 0.3, 1.0));
        }
        if let Some((_, t0)) = &medicion {
            if t - t0 < 2.0 { txt("¡Medición! La función de onda colapsa: cada electrón aparece en un solo punto.", 470.0, 102.0, 16, YELLOW); }
        }
        // Barra de escala
        let por_px = (2.0 * 10.0 * (22.5f32).to_radians().tan() / h) as f64 / k;
        let (valor, etiqueta) = escala_bonita(por_px * 140.0);
        let largo = (valor / por_px) as f32;
        draw_line(w - 40.0 - largo, h - 40.0, w - 40.0, h - 40.0, 3.0, WHITE);
        draw_line(w - 40.0 - largo, h - 46.0, w - 40.0 - largo, h - 34.0, 2.0, WHITE);
        draw_line(w - 40.0, h - 46.0, w - 40.0, h - 34.0, 2.0, WHITE);
        txt(&etiqueta, w - 40.0 - largo / 2.0 - 20.0, h - 50.0, 18, WHITE);
        txt(&format!("cámara a {} m", cientifico(dist)), w - 230.0, h - 14.0, 14, GRAY);

        txt("←/→ elemento · ↑/↓ ±electrón (ion) · N/B ±neutrón (isótopo) · rueda o +/− zoom · 1 átomo 2 núcleo 3 nucleón 4 quarks",
            20.0, h - 44.0, 14, GRAY);
        txt("E excitar (absorbe y emite fotón) · M medir (colapso de ψ) · O nube · arrastrar girar · espacio pausa",
            20.0, h - 22.0, 14, GRAY);
        next_frame().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hund_en_el_nitrogeno() {
        let e = electrones(7);
        let p: Vec<_> = e.iter().filter(|x| x.n == 2 && x.l == 1).collect();
        assert_eq!(p.len(), 3);
        assert!(p.iter().all(|x| x.ms == 1));
        let ml: std::collections::BTreeSet<_> = p.iter().map(|x| x.ml).collect();
        assert_eq!(ml.len(), 3);
    }

    #[test]
    fn pauli_nunca_repite_numeros_cuanticos() {
        let e = electrones(118);
        let unicos: std::collections::BTreeSet<_> = e.iter().map(|x| (x.n, x.l, x.ml, x.ms)).collect();
        assert_eq!(unicos.len(), 118);
    }

    #[test]
    fn nucleo_tiene_los_nucleones_correctos() {
        let mut r = Rng::nuevo(1);
        let v = nucleo(26, 30, &mut r);
        assert_eq!(v.iter().filter(|x| x.1).count(), 26);
        assert_eq!(v.len(), 56);
        let r_max = v.iter().map(|(p, _)| (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt()).fold(0.0, f64::max);
        assert!(r_max <= fis::radio_nuclear_fm(56) * 1e-15);
    }

    #[test]
    fn escala_legible() {
        assert_eq!(escala_bonita(3.2e-15).1, "2 fm");
        assert_eq!(escala_bonita(1.2e-10).1, "100 pm");
    }
}
