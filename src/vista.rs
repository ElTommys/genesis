//! Visores en ventana: química en 3D (`genesis ver`) y bioma vivo en 2D (`genesis vivir`).

use crate::bioma::{Bioma, Dieta, CELDA_SUELO};
use crate::conocimiento::Reino;
use crate::elementos::{color_cpk, SIMBOLOS};
use crate::mundo::Mundo;
use crate::vida::RASGOS;
use macroquad::prelude::*;

fn conf(titulo: &str) -> Conf {
    Conf {
        window_title: titulo.to_string(),
        window_width: 1280,
        window_height: 800,
        high_dpi: true,
        sample_count: 4,
        window_resizable: true,
        ..Default::default()
    }
}

/// La fuente por defecto no trae tildes: se escriben sin ellas en pantalla.
fn plano(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'á' => 'a', 'é' => 'e', 'í' => 'i', 'ó' => 'o', 'ú' => 'u', 'ñ' => 'n',
            'Á' => 'A', 'É' => 'E', 'Í' => 'I', 'Ó' => 'O', 'Ú' => 'U', 'Ñ' => 'N',
            '₂' => '2', '³' => '3', '°' => 'o', '−' => '-', 'δ' => 'd',
            c => c,
        })
        .collect()
}

fn texto(s: &str, x: f32, y: f32, tam: f32, c: Color) {
    draw_text(&plano(s), x, y, tam, c);
}

// ───────────────────────────── Química 3D ─────────────────────────────

pub fn ver(m: Mundo) {
    macroquad::Window::from_config(conf("Genesis - quimica 3D"), bucle_quimica(m));
}

async fn bucle_quimica(mut m: Mundo) {
    let (mut yaw, mut pitch, mut dist) = (0.6f32, 0.4f32, m.p.caja as f32 * 1.9);
    let mut pausa = false;
    let mut por_cargas = false;
    let mut pasos_frame = 20u64;
    let mut ultimo_raton = mouse_position();
    loop {
        if is_key_pressed(KeyCode::Space) { pausa = !pausa; }
        if is_key_pressed(KeyCode::C) { por_cargas = !por_cargas; }
        if is_key_pressed(KeyCode::Up) { m.p.temperatura += 0.1; }
        if is_key_pressed(KeyCode::Down) { m.p.temperatura = (m.p.temperatura - 0.1).max(0.01); }
        if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) { pasos_frame = (pasos_frame * 2).min(640); }
        if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) { pasos_frame = (pasos_frame / 2).max(1); }
        let raton = mouse_position();
        if is_mouse_button_down(MouseButton::Left) {
            yaw += (raton.0 - ultimo_raton.0) * 0.008;
            pitch = (pitch + (raton.1 - ultimo_raton.1) * 0.008).clamp(-1.5, 1.5);
        }
        ultimo_raton = raton;
        dist = (dist * (1.0 - mouse_wheel().1 * 0.05)).clamp(3.0, 200.0);
        if !pausa {
            m.avanzar(pasos_frame);
        }

        clear_background(Color::from_rgba(8, 10, 20, 255));
        let l = m.p.caja as f32;
        let centro = vec3(l / 2.0, l / 2.0, l / 2.0);
        let ojo = centro + vec3(pitch.cos() * yaw.cos(), pitch.sin(), pitch.cos() * yaw.sin()) * dist;
        set_camera(&Camera3D { position: ojo, target: centro, up: vec3(0.0, 1.0, 0.0), ..Default::default() });
        draw_cube_wires(centro, vec3(l, l, l), Color::from_rgba(60, 70, 110, 255));
        let v3 = |p: &[f64; 3]| vec3(p[0] as f32, p[1] as f32, p[2] as f32);
        for (i, b) in m.cuerpos.iter().enumerate() {
            for &j in &m.enlaces[i] {
                if j > i {
                    let d = crate::mundo::delta(&m.p, &m.cuerpos[j].pos, &b.pos);
                    let a = v3(&b.pos);
                    draw_line_3d(a, a + vec3(d[0] as f32, d[1] as f32, d[2] as f32), Color::from_rgba(200, 200, 200, 255));
                }
            }
        }
        for b in &m.cuerpos {
            let color = if por_cargas {
                let q = (b.q as f32).clamp(-1.0, 1.0);
                if q < 0.0 { Color::new(1.0, 1.0 + q, 1.0 + q, 1.0) } else { Color::new(1.0 - q, 1.0 - q, 1.0, 1.0) }
            } else {
                let (r, g, bl) = color_cpk(b.z);
                Color::from_rgba(r, g, bl, 255)
            };
            let radio = if b.z == 1 { 0.22 } else { 0.32 + 0.02 * (b.z.min(20) as f32 / 4.0) };
            draw_sphere(v3(&b.pos), radio, None, color);
        }

        set_default_camera();
        let mut censo: Vec<_> = m.censo().into_iter().collect();
        censo.sort_by(|a, b| b.1.cmp(&a.1));
        draw_rectangle(10.0, 10.0, 360.0, 300.0, Color::from_rgba(0, 0, 0, 160));
        texto("GENESIS  -  quimica", 20.0, 36.0, 28.0, WHITE);
        texto(&format!("paso {}   T = {:.2}   enlaces {}", m.paso, m.temperatura(), m.num_enlaces()), 20.0, 62.0, 20.0, LIGHTGRAY);
        if let Some(a) = m.angulo_medio(8) {
            texto(&format!("angulo H-O-H medio: {a:.1} grados"), 20.0, 84.0, 20.0, LIGHTGRAY);
        }
        for (k, (f, c)) in censo.iter().take(8).enumerate() {
            texto(&format!("{f:<10} x{c}"), 20.0, 112.0 + k as f32 * 20.0, 20.0, YELLOW);
        }
        texto(if por_cargas { "color: carga (rojo d-, azul d+)" } else { "color: elemento (CPK)" }, 20.0, 290.0, 18.0, GRAY);
        texto("arrastrar: girar | rueda: zoom | flechas: temperatura | C: cargas | +/-: velocidad | espacio: pausa",
            10.0, screen_height() - 12.0, 18.0, GRAY);
        next_frame().await;
    }
}

// ───────────────────────────── Bioma 2D ─────────────────────────────

fn color_dieta(d: Dieta, especie: usize) -> Color {
    let v = (especie % 5) as f32 * 0.08;
    match d {
        Dieta::Productor => Color::new(0.15 + v, 0.75 - v, 0.2, 1.0),
        Dieta::Herbivoro => Color::new(0.3, 0.6 + v, 1.0, 1.0),
        Dieta::Omnivoro => Color::new(0.75, 0.4 + v, 0.95, 1.0),
        Dieta::Carnivoro => Color::new(1.0, 0.2 + v, 0.15, 1.0),
        Dieta::Descomponedor => Color::new(0.85, 0.6 + v, 0.3, 1.0),
    }
}

pub fn vivir(b: Bioma) {
    macroquad::Window::from_config(conf("Genesis - bioma"), bucle_bioma(b));
}

async fn bucle_bioma(mut b: Bioma) {
    let mut pausa = false;
    let mut ticks_frame = 2u64;
    let mut elegido: Option<u64> = None;
    loop {
        if is_key_pressed(KeyCode::Space) { pausa = !pausa; }
        if is_key_pressed(KeyCode::Up) { b.p.luz = (b.p.luz + 0.1).min(3.0); }
        if is_key_pressed(KeyCode::Down) { b.p.luz = (b.p.luz - 0.1).max(0.0); }
        if is_key_pressed(KeyCode::Right) { b.p.temperatura += 2.0; }
        if is_key_pressed(KeyCode::Left) { b.p.temperatura -= 2.0; }
        if is_key_pressed(KeyCode::M) { b.p.tasa_mutacion = (b.p.tasa_mutacion * 2.0).min(0.2); }
        if is_key_pressed(KeyCode::N) { b.p.tasa_mutacion /= 2.0; }
        if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) { ticks_frame = (ticks_frame * 2).min(64); }
        if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) { ticks_frame = (ticks_frame / 2).max(1); }

        let panel = 380.0;
        let escala = ((screen_width() - panel) / b.p.ancho).min(screen_height() / b.p.alto);
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            let (wx, wy) = (mx / escala, my / escala);
            elegido = b
                .orgs
                .iter()
                .map(|o| (o.id, (o.pos[0] - wx).powi(2) + (o.pos[1] - wy).powi(2)))
                .filter(|&(_, d)| d < 150.0)
                .min_by(|a, c| a.1.total_cmp(&c.1))
                .map(|(id, _)| id);
        }
        if !pausa {
            b.avanzar(ticks_frame);
        }

        clear_background(Color::from_rgba(12, 10, 8, 255));
        // Suelo: más oscuro y verde = más nutrientes.
        for f in 0..b.filas {
            for c in 0..b.cols {
                let n = (b.suelo[f * b.cols + c] / 10.0).min(1.0);
                draw_rectangle(c as f32 * CELDA_SUELO * escala, f as f32 * CELDA_SUELO * escala, CELDA_SUELO * escala + 1.0, CELDA_SUELO * escala + 1.0,
                    Color::new(0.16 + 0.05 * n, 0.12 + 0.12 * n, 0.07, 1.0));
            }
        }
        for c in &b.cadaveres {
            draw_circle(c.pos[0] * escala, c.pos[1] * escala, 1.5 * escala, Color::from_rgba(120, 110, 100, 200));
        }
        for o in &b.orgs {
            let col = color_dieta(o.dieta(), o.especie);
            let (x, y, r) = (o.pos[0] * escala, o.pos[1] * escala, o.tam * escala * 0.8);
            if o.reino == Reino::Animalia {
                draw_circle(x, y, r, col);
                draw_line(x, y, x + o.dir.cos() * r * 1.6, y + o.dir.sin() * r * 1.6, 1.5, WHITE);
            } else if o.reino == Reino::Fungi {
                draw_poly(x, y, 6, r, 0.0, col);
            } else {
                draw_circle(x, y, r * 0.7, col);
            }
            if Some(o.id) == elegido {
                draw_circle_lines(x, y, r + 4.0, 2.0, YELLOW);
                draw_circle_lines(x, y, o.vision * escala, 1.0, Color::new(1.0, 1.0, 0.0, 0.3));
            }
        }

        // Panel de datos.
        let x0 = screen_width() - panel;
        draw_rectangle(x0, 0.0, panel, screen_height(), Color::from_rgba(14, 16, 26, 255));
        let mut y = 34.0;
        let mut linea = |s: &str, tam: f32, c: Color, y: &mut f32| {
            texto(s, x0 + 16.0, *y, tam, c);
            *y += tam + 4.0;
        };
        linea("GENESIS  -  bioma", 28.0, WHITE, &mut y);
        linea(&format!("tick {}  gen. max {}  especies {}", b.tick, b.est.generacion_max, b.especies_vivas()), 18.0, LIGHTGRAY, &mut y);
        linea(&format!("luz {:.1}  temp {:.0} C  mutacion {:.4}", b.p.luz, b.p.temperatura, b.p.tasa_mutacion), 18.0, LIGHTGRAY, &mut y);
        y += 6.0;
        // Pirámide de biomasa (de productores abajo a carnívoros arriba).
        let m = b.biomasa_por_nivel();
        let pob = b.por_nivel();
        let max = m.iter().cloned().fold(1.0, f32::max);
        linea("Piramide de biomasa", 20.0, WHITE, &mut y);
        for &k in &[3usize, 2, 1, 0, 4] {
            let w = (m[k] / max).sqrt() * (panel - 150.0);
            let col = color_dieta(Dieta::TODAS[k], 0);
            draw_rectangle(x0 + 16.0 + (panel - 150.0 - w) / 2.0, y - 14.0, w.max(2.0), 16.0, col);
            texto(&format!("{} {}", Dieta::TODAS[k].nombre(), pob[k]), x0 + panel - 128.0, y, 16.0, LIGHTGRAY);
            y += 20.0;
        }
        y += 6.0;
        // Gráfica de poblaciones en el tiempo.
        let (gx, gy, gw, gh) = (x0 + 16.0, y, panel - 32.0, 110.0);
        draw_rectangle_lines(gx, gy, gw, gh, 1.0, DARKGRAY);
        let h = &b.historial;
        if h.len() > 1 {
            let maxp = h.iter().flat_map(|v| v.iter()).cloned().max().unwrap_or(1).max(1) as f32;
            for k in 0..5 {
                let col = color_dieta(Dieta::TODAS[k], 0);
                for t in 1..h.len() {
                    let xa = gx + (t - 1) as f32 / (h.len() - 1) as f32 * gw;
                    let xb = gx + t as f32 / (h.len() - 1) as f32 * gw;
                    let ya = gy + gh - (h[t - 1][k] as f32 / maxp).sqrt() * gh;
                    let yb = gy + gh - (h[t][k] as f32 / maxp).sqrt() * gh;
                    draw_line(xa, ya, xb, yb, 1.5, col);
                }
            }
        }
        y += gh + 24.0;
        let e = &b.est;
        linea(&format!("nacimientos {}  clones {}", e.nacimientos, e.clones), 17.0, LIGHTGRAY, &mut y);
        linea(&format!("gemelos identicos {}  mellizos {}", e.gemelos, e.mellizos), 17.0, LIGHTGRAY, &mut y);
        linea(&format!("depredacion {}  hambre {}  vejez {}", e.depredaciones, e.muertes_hambre, e.muertes_vejez), 17.0, LIGHTGRAY, &mut y);
        linea(&format!("mutaciones {}  especies nuevas {}  extintas {}", e.mutaciones.total(), e.especiaciones, e.extinciones), 17.0, LIGHTGRAY, &mut y);
        y += 6.0;
        if let Some(o) = elegido.and_then(|id| b.orgs.iter().find(|o| o.id == id)) {
            let esp = &b.especies[o.especie];
            linea(&esp.nombre, 22.0, YELLOW, &mut y);
            linea(&format!("{}  -  {}", esp.cientifico, o.dieta().nombre()), 16.0, LIGHTGRAY, &mut y);
            linea(&format!("generacion {}  edad {}/{}  energia {:.1}", o.generacion, o.edad, o.longevidad, o.energia), 16.0, LIGHTGRAY, &mut y);
            if o.gemelo_de.is_some() {
                linea("gemelo identico", 16.0, PINK, &mut y);
            }
            for (i, nombre) in RASGOS.iter().enumerate() {
                let v = o.genoma.rasgo(i) as f32;
                texto(nombre, x0 + 16.0, y, 15.0, GRAY);
                draw_rectangle(x0 + 110.0, y - 10.0, v * 200.0, 10.0, SKYBLUE);
                y += 15.0;
            }
            let adn = o.genoma.texto();
            for k in 0..3 {
                let trozo: String = adn.chars().skip(k * 40).take(40).collect();
                linea(&trozo, 14.0, Color::from_rgba(120, 200, 140, 255), &mut y);
            }
        } else {
            linea("clic sobre un organismo para ver su ADN", 16.0, GRAY, &mut y);
        }
        texto("espacio: pausa | flechas: luz y temperatura | M/N: mutacion | +/-: velocidad",
            10.0, screen_height() - 10.0, 16.0, GRAY);
        let _ = SIMBOLOS;
        next_frame().await;
    }
}
