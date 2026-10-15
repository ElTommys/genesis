//! Visor de moléculas (`genesis molecula [nombre|SMILES]`): la forma 3D sale de la física
//! (radios covalentes + VSEPR + repulsión). Muestra enlaces simples, dobles y triples,
//! cargas δ±, pares de electrones libres, y los datos químicos de la molécula.

use crate::elementos::{color_cpk, NOMBRES, SIMBOLOS};
use crate::quimica::{self as q, Molecula};
use macroquad::prelude::*;

fn color(z: u32) -> Color {
    let (r, g, b) = color_cpk(z);
    Color::from_rgba(r, g, b, 255)
}

fn v3(p: [f64; 3]) -> Vec3 {
    vec3(p[0] as f32, p[1] as f32, p[2] as f32)
}

/// Barra de enlace como cadena de esferas pequeñas (macroquad no orienta cilindros).
fn barra(a: Vec3, b: Vec3, r: f32, c: Color) {
    let n = ((a - b).length() / (r * 0.9)).ceil().max(2.0) as usize;
    for i in 0..=n {
        draw_sphere(a.lerp(b, i as f32 / n as f32), r, None, c);
    }
}

pub fn ver(inicial: Option<Molecula>) {
    let conf = Conf {
        window_title: "Genesis - moleculas".into(),
        window_width: 1400,
        window_height: 880,
        high_dpi: true,
        sample_count: 4,
        window_resizable: true,
        ..Default::default()
    };
    macroquad::Window::from_config(conf, bucle(inicial));
}

fn cargar(i: usize) -> Molecula {
    let (n, s, _) = q::CATALOGO[i];
    q::leer_smiles(n, s).expect("catálogo válido")
}

async fn bucle(inicial: Option<Molecula>) {
    let fuente = std::fs::read("/System/Library/Fonts/Supplemental/Arial Unicode.ttf")
        .ok()
        .and_then(|b| load_ttf_font_from_bytes(&b).ok());
    let txt = |s: &str, x: f32, y: f32, tam: u16, c: Color| {
        draw_text_ex(s, x, y, TextParams { font: fuente.as_ref(), font_size: tam, color: c, ..Default::default() });
    };
    let mut indice = 3usize; // agua
    let mut m = inicial.unwrap_or_else(|| cargar(indice));
    let (mut yaw, mut pitch, mut dist) = (0.6f32, 0.3f32, 8.0f32);
    let mut ultimo = mouse_position();
    let mut relleno = false;
    let mut por_cargas = false;
    let mut pares = true;
    let mut etiquetas = true;
    let mut girar = true;
    let mut t = 0.0f32;
    loop {
        if is_key_pressed(KeyCode::Right) { indice = (indice + 1) % q::CATALOGO.len(); m = cargar(indice); }
        if is_key_pressed(KeyCode::Left) { indice = (indice + q::CATALOGO.len() - 1) % q::CATALOGO.len(); m = cargar(indice); }
        if is_key_pressed(KeyCode::V) { relleno = !relleno; }
        if is_key_pressed(KeyCode::C) { por_cargas = !por_cargas; }
        if is_key_pressed(KeyCode::P) { pares = !pares; }
        if is_key_pressed(KeyCode::L) { etiquetas = !etiquetas; }
        if is_key_pressed(KeyCode::Space) { girar = !girar; }
        let raton = mouse_position();
        if is_mouse_button_down(MouseButton::Left) && raton.0 > 440.0 {
            yaw += (raton.0 - ultimo.0) * 0.008;
            pitch = (pitch + (raton.1 - ultimo.1) * 0.008).clamp(-1.5, 1.5);
        }
        ultimo = raton;
        dist = (dist * (1.0 - mouse_wheel().1.clamp(-3.0, 3.0) * 0.05)).clamp(1.5, 120.0);
        if is_key_down(KeyCode::Equal) { dist *= 0.98; }
        if is_key_down(KeyCode::Minus) { dist *= 1.02; }
        t += get_frame_time();
        if girar { yaw += get_frame_time() * 0.25; }

        clear_background(Color::from_rgba(6, 8, 16, 255));
        let ojo = vec3(pitch.cos() * yaw.cos(), pitch.sin(), pitch.cos() * yaw.sin()) * dist;
        let camara = Camera3D { position: ojo, target: Vec3::ZERO, up: Vec3::Y, ..Default::default() };
        set_camera(&camara);

        let col = |i: usize| -> Color {
            if por_cargas {
                let qq = (m.atomos[i].q as f32 * 2.0).clamp(-1.0, 1.0);
                if qq < 0.0 { Color::new(1.0, 1.0 + qq, 1.0 + qq, 1.0) } else { Color::new(1.0 - qq, 1.0 - qq, 1.0, 1.0) }
            } else {
                color(m.atomos[i].z)
            }
        };
        // Enlaces: 1, 2 o 3 barras paralelas; aromático = una barra + una punteada.
        if !relleno {
            for e in &m.enlaces {
                let (a, b) = (v3(m.atomos[e.a].pos), v3(m.atomos[e.b].pos));
                let eje = (b - a).normalize_or_zero();
                let perp = eje.cross((ojo - a).normalize_or_zero()).normalize_or_zero() * 0.11;
                let medio = a.lerp(b, 0.5);
                let barras: Vec<Vec3> = match (e.orden * 2.0).round() as u32 {
                    4 => vec![perp, -perp],
                    6 => vec![Vec3::ZERO, perp * 1.6, -perp * 1.6],
                    _ => vec![Vec3::ZERO],
                };
                for d in &barras {
                    barra(a + *d, medio + *d, 0.05, col(e.a));
                    barra(medio + *d, b + *d, 0.05, col(e.b));
                }
                if (e.orden - 1.5).abs() < 0.01 {
                    for k in 0..6 {
                        let f = 0.2 + k as f32 * 0.12;
                        draw_sphere(a.lerp(b, f) + perp * 1.4, 0.03, None, Color::new(0.8, 0.8, 0.9, 0.8));
                    }
                }
            }
        }
        // Átomos
        for (i, a) in m.atomos.iter().enumerate() {
            let r = if relleno { q::radio_vdw_pm(a.z) / 100.0 } else { q::radio_covalente_pm(a.z) / 100.0 * 0.42 + 0.08 };
            draw_sphere(v3(a.pos), r as f32, None, col(i));
        }
        // Pares de electrones libres: dos puntos en la dirección opuesta a los enlaces (VSEPR).
        if pares && !relleno {
            for i in 0..m.atomos.len() {
                let lp = m.pares_libres(i);
                if lp == 0 {
                    continue;
                }
                let c = v3(m.atomos[i].pos);
                let vec = m.vecinos(i);
                let mut opuesto = Vec3::ZERO;
                for (j, _) in &vec {
                    opuesto -= (v3(m.atomos[*j].pos) - c).normalize_or_zero();
                }
                let dir = if opuesto.length() < 1e-3 { Vec3::Y } else { opuesto.normalize() };
                let p1 = dir.any_orthonormal_vector();
                let p2 = dir.cross(p1);
                for k in 0..lp {
                    let ang = k as f32 * std::f32::consts::TAU / lp as f32 + t * 0.5;
                    let d = (dir * 0.8 + (p1 * ang.cos() + p2 * ang.sin()) * if lp > 1 { 0.55 } else { 0.0 }).normalize() * 0.55;
                    let s = d.cross(Vec3::Y).normalize_or_zero() * 0.06;
                    draw_sphere(c + d + s, 0.045, None, Color::new(1.0, 0.95, 0.4, 0.95));
                    draw_sphere(c + d - s, 0.045, None, Color::new(1.0, 0.95, 0.4, 0.95));
                }
            }
        }

        // ── Panel ──
        set_default_camera();
        if etiquetas && !relleno {
            for a in &m.atomos {
                if a.z != 1 || m.atomos.len() < 12 {
                    let pr = camara.matrix().project_point3(v3(a.pos));
                    if pr.z > 1.0 { continue; }
                    let s = vec2((pr.x + 1.0) * 0.5 * screen_width(), (1.0 - pr.y) * 0.5 * screen_height());
                    let qtxt = if a.q.abs() > 0.05 { if a.q < 0.0 { "δ−" } else { "δ+" } } else { "" };
                    let carga = match a.carga { 0 => String::new(), 1 => "⁺".into(), -1 => "⁻".into(), c => format!("{c:+}") };
                    txt(&format!("{}{carga}{qtxt}", SIMBOLOS[(a.z - 1) as usize]), s.x + 8.0, s.y - 8.0, 16, WHITE);
                }
            }
        }
        let h = screen_height();
        draw_rectangle(0.0, 0.0, 430.0, h, Color::from_rgba(8, 10, 22, 235));
        let mut y = 40.0;
        txt(&m.nombre, 20.0, y, 32, WHITE);
        y += 28.0;
        txt(&format!("{}   ·   SMILES {}", m.formula(), m.smiles), 20.0, y, 15, LIGHTGRAY);
        y += 30.0;
        let masa = m.masa_molar();
        txt(&format!("masa molar M = {masa:.3} g/mol"), 20.0, y, 17, LIGHTGRAY);
        y += 22.0;
        txt(&format!("1 mol = 6.022×10²³ moléculas = {masa:.2} g"), 20.0, y, 17, LIGHTGRAY);
        y += 22.0;
        txt(&format!("1 g contiene {:.3}×10²¹ moléculas", 6.022e23 / masa / 1e21), 20.0, y, 17, LIGHTGRAY);
        y += 22.0;
        txt(&format!("{} átomos · {} enlaces", m.atomos.len(), m.enlaces.len()), 20.0, y, 17, LIGHTGRAY);
        y += 22.0;
        txt(&format!("energía de todos los enlaces: {:.0} kJ/mol", m.energia_enlaces_kj()), 20.0, y, 17, LIGHTGRAY);
        y += 22.0;
        let mu = m.dipolo_debye();
        txt(&format!("momento dipolar μ ≈ {mu:.2} D → {}", if mu > 0.4 { "molécula polar" } else { "molécula apolar" }), 20.0, y, 17, if mu > 0.4 { ORANGE } else { SKYBLUE });
        y += 32.0;

        // Tipos de enlace presentes.
        txt("Enlaces", 20.0, y, 20, WHITE);
        y += 22.0;
        let mut tipos: Vec<(String, usize, f64, f64, u32, u32)> = Vec::new();
        for e in &m.enlaces {
            let (za, zb) = (m.atomos[e.a].z.min(m.atomos[e.b].z), m.atomos[e.a].z.max(m.atomos[e.b].z));
            let sim = match (e.orden * 2.0).round() as u32 { 3 => "⋯", 4 => "=", 6 => "≡", _ => "–" };
            let clave = format!("{}{sim}{}", SIMBOLOS[(za - 1) as usize], SIMBOLOS[(zb - 1) as usize]);
            let l = m.distancia(e.a, e.b);
            match tipos.iter_mut().find(|x| x.0 == clave) {
                Some(x) => x.1 += 1,
                None => tipos.push((clave, 1, q::energia_enlace_kj(za, zb, e.orden), l, za, zb)),
            }
        }
        for (k, n, en, l, za, zb) in tipos.iter().take(9) {
            let (tipo, d) = q::tipo_enlace(*za, *zb);
            txt(&format!("{k} ×{n}  {:.0} pm  {en:.0} kJ/mol  {} (Δχ {d:.2})", l * 100.0, tipo.nombre()), 20.0, y, 15, LIGHTGRAY);
            y += 19.0;
        }
        y += 14.0;
        // Geometría VSEPR de los átomos centrales.
        txt("Geometría (VSEPR)", 20.0, y, 20, WHITE);
        y += 22.0;
        let mut mostrados = 0;
        for i in 0..m.atomos.len() {
            let v = m.vecinos(i);
            if v.len() >= 2 && mostrados < 6 {
                let ang = m.angulo(v[0].0, i, v[1].0);
                txt(&format!("{} #{i}: {} · {} pares libres · ángulo {:.1}°", SIMBOLOS[(m.atomos[i].z - 1) as usize], m.geometria_de(i), m.pares_libres(i), ang), 20.0, y, 15, LIGHTGRAY);
                y += 19.0;
                mostrados += 1;
            }
        }
        y += 14.0;
        txt("Elementos", 20.0, y, 20, WHITE);
        y += 22.0;
        for (z, n) in m.conteo() {
            draw_circle(28.0, y - 5.0, 6.0, color(z));
            txt(&format!("{} ({}) ×{n}  ·  {:.3} g/mol c/u", NOMBRES[(z - 1) as usize], SIMBOLOS[(z - 1) as usize], q::peso_atomico(z)), 42.0, y, 15, LIGHTGRAY);
            y += 19.0;
        }
        txt("←/→ otra molécula · V esferas reales (van der Waals) · C cargas δ± · P pares libres · L etiquetas",
            20.0, h - 40.0, 14, GRAY);
        txt("rueda o +/− zoom · arrastrar girar · espacio giro automático · amarillo = pares de electrones libres",
            20.0, h - 18.0, 14, GRAY);
        next_frame().await;
    }
}
