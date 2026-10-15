//! La biosfera en 3D (`genesis mundo`): un continente con relieve, clima y biomas, donde
//! la vida corre sola. Con el zoom se recorren los niveles de organización ecológica:
//! organismo → población → comunidad → ecosistema → bioma → biosfera.

use crate::bioma::{Bioma, Dieta, Organismo};
use crate::clima::TipoBioma;
use crate::conocimiento::{Reino, PLANTILLAS};
use crate::vida::RASGOS;
use macroquad::prelude::*;

/// Exageración vertical del relieve (el mundo está comprimido en horizontal).
const ESC_V: f32 = 0.12;

#[derive(Clone, Copy, PartialEq)]
enum Nivel {
    Organismo,
    Poblacion,
    Comunidad,
    Ecosistema,
    Bioma,
    Biosfera,
}

impl Nivel {
    fn de(dist: f32) -> Nivel {
        match dist {
            d if d < 14.0 => Nivel::Organismo,
            d if d < 90.0 => Nivel::Poblacion,
            d if d < 380.0 => Nivel::Comunidad,
            d if d < 1200.0 => Nivel::Ecosistema,
            d if d < 3200.0 => Nivel::Bioma,
            _ => Nivel::Biosfera,
        }
    }
    fn nombre(self) -> &'static str {
        match self {
            Nivel::Organismo => "ORGANISMO",
            Nivel::Poblacion => "POBLACIÓN",
            Nivel::Comunidad => "COMUNIDAD",
            Nivel::Ecosistema => "ECOSISTEMA",
            Nivel::Bioma => "BIOMA",
            Nivel::Biosfera => "BIOSFERA",
        }
    }
    fn definicion(self) -> &'static str {
        match self {
            Nivel::Organismo => "Un ser vivo individual completo: todos sus sistemas trabajando en equilibrio.",
            Nivel::Poblacion => "Individuos de la misma especie que viven en un lugar y pueden cruzarse entre sí.",
            Nivel::Comunidad => "Todas las poblaciones de especies distintas que conviven e interactúan en un lugar.",
            Nivel::Ecosistema => "La comunidad más su entorno físico: suelo, agua, luz, temperatura (bióticos + abióticos).",
            Nivel::Bioma => "Gran región con clima parecido y vegetación característica (clasificación de Whittaker).",
            Nivel::Biosfera => "Todos los ecosistemas del planeta: la capa donde existe la vida.",
        }
    }
    fn distancia(self) -> f32 {
        match self {
            Nivel::Organismo => 9.0,
            Nivel::Poblacion => 50.0,
            Nivel::Comunidad => 220.0,
            Nivel::Ecosistema => 700.0,
            Nivel::Bioma => 2000.0,
            Nivel::Biosfera => 4200.0,
        }
    }
}

fn color_especie(o: &Organismo) -> Color {
    let h = (o.especie as f32 * 0.37).fract();
    match o.dieta() {
        Dieta::Productor => Color::new(0.12 + 0.25 * h, 0.45 + 0.35 * (1.0 - h), 0.12 + 0.15 * h, 1.0),
        Dieta::Herbivoro => Color::new(0.55 + 0.35 * h, 0.45 + 0.25 * h, 0.3, 1.0),
        Dieta::Omnivoro => Color::new(0.35 + 0.2 * h, 0.25, 0.2 + 0.1 * h, 1.0),
        Dieta::Carnivoro => Color::new(0.75 + 0.2 * h, 0.35 + 0.3 * h, 0.15, 1.0),
        Dieta::Descomponedor => Color::new(0.9, 0.85, 0.7, 1.0),
    }
}

fn color_dieta(d: Dieta) -> Color {
    match d {
        Dieta::Productor => Color::new(0.2, 0.8, 0.3, 1.0),
        Dieta::Herbivoro => Color::new(0.35, 0.65, 1.0, 1.0),
        Dieta::Omnivoro => Color::new(0.75, 0.45, 0.95, 1.0),
        Dieta::Carnivoro => Color::new(1.0, 0.3, 0.2, 1.0),
        Dieta::Descomponedor => Color::new(0.9, 0.65, 0.3, 1.0),
    }
}

fn sistemas(reino: Reino, clase: &str) -> &'static str {
    match reino {
        Reino::Animalia if clase.contains("Mamífero") => "Sistemas: nervioso, circulatorio (corazón de 4 cavidades), respiratorio (pulmones), digestivo, excretor (riñones), endocrino, inmune, muscular, óseo, tegumentario (pelo), reproductor (placenta).",
        Reino::Animalia => "Sistemas: nervioso, circulatorio, respiratorio, digestivo, excretor, muscular, reproductor.",
        Reino::Plantae => "Órganos: raíz, tallo, hojas y estructuras reproductoras. Tejidos: meristemos, parénquima con clorofila (fotosíntesis), xilema (agua), floema (azúcares), epidermis con estomas (intercambio de gases).",
        Reino::Fungi => "Cuerpo: micelio de hifas con pared de quitina; cuerpo fructífero que libera esporas. Digiere por fuera y absorbe.",
    }
}

fn plantilla_de(b: &Bioma, o: &Organismo) -> Option<&'static crate::conocimiento::Plantilla> {
    let mut e = o.especie;
    loop {
        if let Some(n) = b.especies[e].plantilla {
            return PLANTILLAS.iter().find(|p| p.nombre == n);
        }
        e = b.especies[e].ancestro?;
    }
}

/// Malla del terreno coloreada por bioma (se rehace si cambia el clima global).
fn malla_terreno(b: &Bioma) -> Vec<Mesh> {
    let c = b.clima.as_ref().unwrap();
    let (cols, filas) = (c.cols, c.filas);
    let (ox, oz) = (c.ancho / 2.0, c.alto / 2.0);
    let alt = |cc: usize, ff: usize| c.celdas[ff.min(filas - 1) * cols + cc.min(cols - 1)].altitud;
    // Se parte en franjas para no pasar de 65.535 vértices por malla.
    let mut mallas = Vec::new();
    let franja = 40;
    let mut f0 = 0;
    while f0 < filas - 1 {
        let f1 = (f0 + franja).min(filas - 1);
        let mut v = Vec::new();
        let mut idx = Vec::new();
        for f in f0..=f1 {
            for cc in 0..cols {
                let a = alt(cc, f);
                let pos = [cc as f32 * c.celda + c.celda / 2.0, f as f32 * c.celda + c.celda / 2.0];
                let bio = c.bioma(pos);
                let (r, g, bl) = if a < 0.0 {
                    let d = (-a / 2000.0).min(1.0);
                    (0.45 - 0.3 * d, 0.42 - 0.25 * d, 0.32)
                } else {
                    bio.color()
                };
                // Sombreado por pendiente (luz desde el noroeste).
                let dx = alt(cc + 1, f) - alt(cc.saturating_sub(1), f);
                let dz = alt(cc, f + 1) - alt(cc, f.saturating_sub(1));
                let luz = (1.0 - (dx + dz) * 0.0012).clamp(0.55, 1.25);
                let nieve = if a > 1800.0 { ((a - 1800.0) / 700.0).min(0.8) } else { 0.0 };
                let col = Color::new(
                    (r * luz) * (1.0 - nieve) + nieve,
                    (g * luz) * (1.0 - nieve) + nieve,
                    (bl * luz) * (1.0 - nieve) + nieve,
                    1.0,
                );
                v.push(Vertex::new(pos[0] - ox, a.max(-400.0) * ESC_V, pos[1] - oz, 0.0, 0.0, col));
            }
        }
        for f in 0..(f1 - f0) {
            for cc in 0..cols - 1 {
                let i = (f * cols + cc) as u16;
                let d = cols as u16;
                idx.extend_from_slice(&[i, i + 1, i + d, i + 1, i + d + 1, i + d]);
            }
        }
        mallas.push(Mesh { vertices: v, indices: idx, texture: None });
        f0 = f1;
    }
    mallas
}

pub fn ver(b: Bioma) {
    let conf = Conf {
        window_title: "Genesis - biosfera".into(),
        window_width: 1440,
        window_height: 900,
        high_dpi: true,
        sample_count: 4,
        window_resizable: true,
        ..Default::default()
    };
    macroquad::Window::from_config(conf, bucle(b));
}

async fn bucle(mut b: Bioma) {
    let fuente = std::fs::read("/System/Library/Fonts/Supplemental/Arial Unicode.ttf")
        .ok()
        .and_then(|x| load_ttf_font_from_bytes(&x).ok());
    let txt = |s: &str, x: f32, y: f32, tam: u16, c: Color| {
        draw_text_ex(s, x, y, TextParams { font: fuente.as_ref(), font_size: tam, color: c, ..Default::default() });
    };
    let (ancho, alto) = (b.p.ancho, b.p.alto);
    let altura = |b: &Bioma, p: [f32; 2]| b.clima.as_ref().unwrap().altitud(p).max(0.0) * ESC_V;
    let mut mallas = malla_terreno(&b);
    let mut clima_dibujado = (0.0f32, 1.0f32);
    let mut objetivo = vec2(ancho / 2.0, alto / 2.0); // en coordenadas del mundo (m)
    let (mut yaw, mut pitch, mut dist) = (0.9f32, 0.75f32, 4200.0f32);
    let mut dist_obj = dist;
    let mut ultimo = mouse_position();
    let mut pausa = false;
    let mut ticks_frame = 1u64;
    let mut elegido: Option<u64> = None;
    let mut seguir = false;

    loop {
        // ── Controles ──
        if is_key_pressed(KeyCode::Space) { pausa = !pausa; }
        if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) { ticks_frame = (ticks_frame * 2).clamp(1, 16); }
        if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) { ticks_frame = (ticks_frame / 2).max(1); }
        if is_key_pressed(KeyCode::T) { b.clima.as_mut().unwrap().delta_t += 1.0; }
        if is_key_pressed(KeyCode::G) { b.clima.as_mut().unwrap().delta_t -= 1.0; }
        if is_key_pressed(KeyCode::Y) { b.clima.as_mut().unwrap().factor_lluvia *= 1.15; }
        if is_key_pressed(KeyCode::H) { b.clima.as_mut().unwrap().factor_lluvia /= 1.15; }
        if is_key_pressed(KeyCode::M) { b.p.tasa_mutacion = (b.p.tasa_mutacion * 2.0).min(0.1); }
        if is_key_pressed(KeyCode::N) { b.p.tasa_mutacion /= 2.0; }
        if is_key_pressed(KeyCode::F) { seguir = !seguir; }
        for (k, nivel) in [
            (KeyCode::Key1, Nivel::Organismo),
            (KeyCode::Key2, Nivel::Poblacion),
            (KeyCode::Key3, Nivel::Comunidad),
            (KeyCode::Key4, Nivel::Ecosistema),
            (KeyCode::Key5, Nivel::Bioma),
            (KeyCode::Key6, Nivel::Biosfera),
        ] {
            if is_key_pressed(k) {
                dist_obj = nivel.distancia();
            }
        }
        let raton = mouse_position();
        let en_panel = raton.0 < 440.0;
        if is_mouse_button_down(MouseButton::Left) && !en_panel {
            yaw += (raton.0 - ultimo.0) * 0.006;
            pitch = (pitch + (raton.1 - ultimo.1) * 0.006).clamp(0.08, 1.52);
        }
        let adelante = vec2(-yaw.cos(), -yaw.sin());
        let derecha = vec2(-adelante.y, adelante.x);
        let vel = dist * 0.02;
        if is_mouse_button_down(MouseButton::Right) {
            let d = vec2(raton.0 - ultimo.0, raton.1 - ultimo.1) * dist * 0.0018;
            objetivo -= derecha * d.x - adelante * d.y;
            seguir = false;
        }
        if is_key_down(KeyCode::W) { objetivo += adelante * vel; seguir = false; }
        if is_key_down(KeyCode::S) { objetivo -= adelante * vel; seguir = false; }
        if is_key_down(KeyCode::A) { objetivo -= derecha * vel; seguir = false; }
        if is_key_down(KeyCode::D) { objetivo += derecha * vel; seguir = false; }
        objetivo = objetivo.clamp(Vec2::ZERO, vec2(ancho, alto));
        ultimo = raton;
        let rueda = mouse_wheel().1.clamp(-3.0, 3.0);
        if rueda != 0.0 && !en_panel {
            dist_obj *= 1.12f32.powf(-rueda);
        }
        dist_obj = dist_obj.clamp(2.5, 5200.0);
        dist *= (dist_obj / dist).powf(0.15);
        if !pausa {
            b.avanzar(ticks_frame);
        }
        if seguir {
            if let Some(o) = elegido.and_then(|id| b.orgs.iter().find(|o| o.id == id)) {
                objetivo = vec2(o.pos[0], o.pos[1]);
            }
        }
        let clima_ahora = { let c = b.clima.as_ref().unwrap(); (c.delta_t, c.factor_lluvia) };
        if clima_ahora != clima_dibujado {
            mallas = malla_terreno(&b);
            clima_dibujado = clima_ahora;
        }

        // ── Escena 3D ──
        let cielo = Color::from_rgba(150, 190, 230, 255);
        clear_background(if dist > 3000.0 { Color::from_rgba(8, 10, 20, 255) } else { cielo });
        let (ox, oz) = (ancho / 2.0, alto / 2.0);
        let suelo_y = altura(&b, [objetivo.x, objetivo.y]);
        let centro = vec3(objetivo.x - ox, suelo_y, objetivo.y - oz);
        let ojo = centro + vec3(pitch.cos() * yaw.cos(), pitch.sin(), pitch.cos() * yaw.sin()) * dist;
        let cam = Camera3D {
            position: ojo,
            target: centro,
            up: Vec3::Y,
            z_near: (dist * 0.002).clamp(0.02, 5.0),
            z_far: dist * 12.0 + 9000.0,
            ..Default::default()
        };
        set_camera(&cam);
        for m in &mallas {
            draw_mesh(m);
        }
        draw_plane(vec3(0.0, 0.0, 0.0), vec2(ancho * 0.75, alto * 0.75), None, Color::new(0.12, 0.35, 0.62, 0.72));

        // Organismos: detalle completo cerca del centro; lejos, solo los animales como puntos.
        let radio_detalle = (dist * 1.6).clamp(30.0, 900.0);
        let mut dibujados = 0;
        let mut proyectables: Vec<(u64, Vec3)> = Vec::new();
        for o in &b.orgs {
            let dx = o.pos[0] - objetivo.x;
            let dz = o.pos[1] - objetivo.y;
            let d2 = dx * dx + dz * dz;
            let y = altura(&b, o.pos);
            let p = vec3(o.pos[0] - ox, y, o.pos[1] - oz);
            if d2 < radio_detalle * radio_detalle && dibujados < 4500 {
                dibujados += 1;
                let c = color_especie(o);
                let t = o.tam;
                match o.reino {
                    Reino::Plantae if t > 5.0 => {
                        // Árbol: tronco + copa (conífera en cono, frondosa en esfera).
                        let conifera = b.especies[o.especie].t_opt < 5.0;
                        draw_cylinder(p, t * 0.08, t * 0.12, t * 1.1, None, Color::new(0.35, 0.24, 0.14, 1.0));
                        if conifera {
                            draw_cylinder(p + vec3(0.0, t * 0.5, 0.0), 0.0, t * 0.55, t * 1.6, None, c);
                        } else {
                            draw_sphere(p + vec3(0.0, t * 1.4, 0.0), t * 0.55, None, c);
                        }
                    }
                    Reino::Plantae if t > 4.0 && b.especies[o.especie].lluvia_min < 100.0 => {
                        // Cactus columnar.
                        draw_cylinder(p, t * 0.12, t * 0.14, t * 1.2, None, c);
                        draw_cylinder(p + vec3(t * 0.15, t * 0.5, 0.0), t * 0.06, t * 0.06, t * 0.35, None, c);
                    }
                    Reino::Plantae => {
                        draw_cylinder(p, t * 0.05, t * 0.25, t * 0.35, None, c);
                    }
                    Reino::Fungi => {
                        draw_cylinder(p, t * 0.05, t * 0.06, t * 0.2, None, Color::new(0.95, 0.92, 0.85, 1.0));
                        draw_cylinder(p + vec3(0.0, t * 0.2, 0.0), 0.0, t * 0.18, t * 0.1, None, Color::new(0.75, 0.55, 0.35, 1.0));
                    }
                    Reino::Animalia => {
                        let r = t * 0.32;
                        let cuerpo = p + vec3(0.0, r * 1.1, 0.0);
                        let dir = vec3(o.dir.cos(), 0.0, o.dir.sin());
                        draw_sphere(cuerpo, r, None, c);
                        draw_sphere(cuerpo + dir * r * 1.1 + vec3(0.0, r * 0.35, 0.0), r * 0.5, None, c);
                        if o.gemelo_de.is_some() {
                            draw_sphere(cuerpo + vec3(0.0, r * 1.4, 0.0), r * 0.15, None, PINK);
                        }
                    }
                }
                if Some(o.id) == elegido {
                    draw_sphere_wires(p + vec3(0.0, t * 0.5, 0.0), t * 1.2, None, YELLOW);
                }
                proyectables.push((o.id, p));
            } else if o.reino == Reino::Animalia {
                let s = (dist * 0.004).max(1.5);
                draw_cube(p + vec3(0.0, s, 0.0), Vec3::splat(s), None, color_dieta(o.dieta()));
            }
        }

        // Clic: elige el organismo más cercano al puntero en pantalla.
        let vp = cam.matrix();
        if is_mouse_button_pressed(MouseButton::Left) && !en_panel {
            let (mx, my) = mouse_position();
            elegido = proyectables
                .iter()
                .filter_map(|(id, p)| {
                    let q = vp.project_point3(*p);
                    if q.z > 1.0 { return None; }
                    let s = vec2((q.x + 1.0) * 0.5 * screen_width(), (1.0 - q.y) * 0.5 * screen_height());
                    let d = s.distance(vec2(mx, my));
                    (d < 30.0).then_some((*id, d))
                })
                .min_by(|a, c| a.1.total_cmp(&c.1))
                .map(|x| x.0)
                .or(elegido);
        }

        // ── Panel ──
        set_default_camera();
        let (w, h) = (screen_width(), screen_height());
        let nivel = Nivel::de(dist);
        draw_rectangle(0.0, 0.0, 440.0, h, Color::from_rgba(8, 10, 22, 238));
        let mut y = 36.0;
        txt("GÉNESIS · biosfera", 20.0, y, 26, WHITE);
        y += 22.0;
        let c = b.clima.as_ref().unwrap();
        txt(&format!("tick {}  ·  ×{}  ·  ΔT global {:+.0} °C  ·  lluvia ×{:.2}{}", b.tick, ticks_frame, c.delta_t, c.factor_lluvia, if pausa { "  ·  PAUSA" } else { "" }), 20.0, y, 14, GRAY);
        y += 34.0;
        txt(nivel.nombre(), 20.0, y, 30, Color::new(1.0, 0.85, 0.35, 1.0));
        y += 22.0;
        for linea in partir(nivel.definicion(), 58) {
            txt(&linea, 20.0, y, 15, LIGHTGRAY);
            y += 18.0;
        }
        // Escalera de niveles.
        y += 4.0;
        let niveles = [Nivel::Organismo, Nivel::Poblacion, Nivel::Comunidad, Nivel::Ecosistema, Nivel::Bioma, Nivel::Biosfera];
        for (k, n) in niveles.iter().enumerate() {
            let activo = *n == nivel;
            let x = 20.0 + k as f32 * 68.0;
            draw_rectangle(x, y, 64.0, 18.0, if activo { Color::new(1.0, 0.85, 0.35, 1.0) } else { Color::from_rgba(40, 45, 70, 255) });
            txt(&format!("{} {}", k + 1, &n.nombre()[..3]), x + 6.0, y + 14.0, 13, if activo { BLACK } else { GRAY });
        }
        y += 40.0;

        let r_vista = (dist * 0.7).max(6.0);
        let cerca: Vec<&Organismo> = b
            .orgs
            .iter()
            .filter(|o| {
                let dx = o.pos[0] - objetivo.x;
                let dz = o.pos[1] - objetivo.y;
                dx * dx + dz * dz < r_vista * r_vista
            })
            .collect();
        let foco: Option<&Organismo> = elegido.and_then(|id| b.orgs.iter().find(|o| o.id == id)).or_else(|| {
            cerca.iter().copied().filter(|o| o.reino == Reino::Animalia).min_by(|a, c| {
                let da = (a.pos[0] - objetivo.x).powi(2) + (a.pos[1] - objetivo.y).powi(2);
                let dc = (c.pos[0] - objetivo.x).powi(2) + (c.pos[1] - objetivo.y).powi(2);
                da.total_cmp(&dc)
            })
        });

        match nivel {
            Nivel::Organismo => {
                if let Some(o) = foco {
                    let e = &b.especies[o.especie];
                    let pl = plantilla_de(&b, o);
                    txt(&e.nombre, 20.0, y, 24, WHITE);
                    y += 20.0;
                    txt(&format!("{} · {} · {}", e.cientifico, pl.map_or("", |p| p.clase), o.dieta().nombre()), 20.0, y, 14, LIGHTGRAY);
                    y += 22.0;
                    txt(&format!("edad {} / {} ticks  ·  generación {}  ·  energía {:.1}", o.edad, o.longevidad, o.generacion, o.energia), 20.0, y, 15, LIGHTGRAY);
                    y += 19.0;
                    let ido = b.idoneidad(o.especie, o.pos);
                    txt(&format!("clima aquí: {:.0} °C, {:.0} mm → idoneidad {:.0}%", b.temperatura_en(o.pos), b.lluvia_en(o.pos), ido * 100.0), 20.0, y, 15, if ido > 0.5 { GREEN } else { ORANGE });
                    y += 19.0;
                    if let Some(p) = pl {
                        txt(&format!("hábitat natural: {}  ·  {} kg  ·  vive ~{} años", p.habitat, p.masa_kg, p.vida_anios), 20.0, y, 14, GRAY);
                        y += 19.0;
                    }
                    if o.gemelo_de.is_some() {
                        txt("gemelo idéntico (mismo genoma que su hermano)", 20.0, y, 15, PINK);
                        y += 19.0;
                    }
                    y += 6.0;
                    for (i, nombre) in RASGOS.iter().enumerate() {
                        let v = o.genoma.rasgo(i) as f32;
                        txt(nombre, 20.0, y, 14, GRAY);
                        draw_rectangle(110.0, y - 10.0, v * 220.0, 10.0, SKYBLUE);
                        y += 16.0;
                    }
                    y += 6.0;
                    let adn = o.genoma.texto();
                    for k in 0..3 {
                        let trozo: String = adn.chars().skip(k * 48).take(48).collect();
                        txt(&trozo, 20.0, y, 13, Color::new(0.5, 0.85, 0.6, 1.0));
                        y += 15.0;
                    }
                    y += 8.0;
                    for linea in partir(sistemas(o.reino, pl.map_or("", |p| p.clase)), 60) {
                        txt(&linea, 20.0, y, 14, LIGHTGRAY);
                        y += 17.0;
                    }
                } else {
                    txt("Haz clic sobre un organismo para examinarlo.", 20.0, y, 16, GRAY);
                }
            }
            Nivel::Poblacion => {
                if let Some(o) = foco {
                    let e = &b.especies[o.especie];
                    let pob: Vec<&&Organismo> = cerca.iter().filter(|x| x.especie == o.especie).collect();
                    let ha = std::f32::consts::PI * r_vista * r_vista / 10_000.0;
                    txt(&format!("Población de {}", e.nombre), 20.0, y, 22, WHITE);
                    y += 24.0;
                    txt(&format!("{} individuos en {:.1} ha → densidad {:.1} por hectárea", pob.len(), ha, pob.len() as f32 / ha), 20.0, y, 15, LIGHTGRAY);
                    y += 19.0;
                    txt(&format!("población total en el mundo: {}", e.poblacion), 20.0, y, 15, LIGHTGRAY);
                    y += 26.0;
                    let jov = pob.iter().filter(|x| (x.edad as f32) < x.longevidad as f32 * 0.12).count();
                    let vie = pob.iter().filter(|x| (x.edad as f32) > x.longevidad as f32 * 0.7).count();
                    let adu = pob.len() - jov - vie;
                    txt("Estructura de edades (pirámide)", 20.0, y, 17, WHITE);
                    y += 20.0;
                    let maxc = jov.max(adu).max(vie).max(1) as f32;
                    for (nombre, n) in [("viejos", vie), ("adultos", adu), ("jóvenes", jov)] {
                        let ancho_b = n as f32 / maxc * 260.0;
                        draw_rectangle(150.0 + (260.0 - ancho_b) / 2.0 - 60.0, y - 12.0, ancho_b, 14.0, SKYBLUE);
                        txt(&format!("{nombre} {n}"), 20.0, y, 14, LIGHTGRAY);
                        y += 18.0;
                    }
                    y += 10.0;
                    let em = pob.iter().map(|x| x.energia).sum::<f32>() / pob.len().max(1) as f32;
                    txt(&format!("energía media {em:.1}  ·  gemelos aquí {}", pob.iter().filter(|x| x.gemelo_de.is_some()).count()), 20.0, y, 15, LIGHTGRAY);
                    y += 19.0;
                    txt("Se regula por comida, depredadores y densidad (capacidad de carga).", 20.0, y, 14, GRAY);
                }
            }
            Nivel::Comunidad => {
                let mut cuenta: std::collections::BTreeMap<usize, usize> = Default::default();
                for o in &cerca {
                    *cuenta.entry(o.especie).or_default() += 1;
                }
                let n_tot = cerca.len().max(1) as f32;
                let s = cuenta.len();
                let shannon: f32 = cuenta.values().map(|&n| { let p = n as f32 / n_tot; -p * p.ln() }).sum();
                let equidad = if s > 1 { shannon / (s as f32).ln() } else { 0.0 };
                txt(&format!("Riqueza: {s} especies · {} individuos", cerca.len()), 20.0, y, 18, WHITE);
                y += 22.0;
                txt(&format!("Índice de Shannon H' = {shannon:.2}  ·  equitatividad J = {equidad:.2}"), 20.0, y, 15, LIGHTGRAY);
                y += 18.0;
                txt("(H' = −Σ pᵢ ln pᵢ; más alto = comunidad más diversa)", 20.0, y, 13, GRAY);
                y += 24.0;
                let mut lista: Vec<(usize, usize)> = cuenta.into_iter().collect();
                lista.sort_by(|a, c| c.1.cmp(&a.1));
                let maxn = lista.first().map_or(1, |x| x.1) as f32;
                for (e, n) in lista.iter().take(14) {
                    let muestra = b.orgs.iter().find(|o| o.especie == *e).unwrap();
                    draw_rectangle(180.0, y - 11.0, *n as f32 / maxn * 220.0, 12.0, color_dieta(muestra.dieta()));
                    txt(&format!("{} {}", b.especies[*e].nombre, n), 20.0, y, 14, LIGHTGRAY);
                    y += 17.0;
                }
                y += 8.0;
                txt("Interacciones: depredación, herbivoría, competencia por luz y espacio, descomposición.", 20.0, y, 13, GRAY);
            }
            Nivel::Ecosistema => {
                let pos = [objetivo.x, objetivo.y];
                let celda = c.en(pos);
                txt("Factores abióticos", 20.0, y, 18, WHITE);
                y += 21.0;
                txt(&format!("temperatura {:.1} °C · lluvia {:.0} mm/año · altitud {:.0} m", b.temperatura_en(pos), b.lluvia_en(pos), celda.altitud), 20.0, y, 15, LIGHTGRAY);
                y += 19.0;
                let k = ((pos[1] / crate::bioma::CELDA_SUELO) as usize).min(b.filas - 1) * b.cols + ((pos[0] / crate::bioma::CELDA_SUELO) as usize).min(b.cols - 1);
                txt(&format!("nutrientes del suelo {:.1} · luz solar {:.1}", b.suelo[k], b.p.luz), 20.0, y, 15, LIGHTGRAY);
                y += 28.0;
                txt("Factores bióticos · pirámide de biomasa", 20.0, y, 18, WHITE);
                y += 24.0;
                let mut masa = [0.0f32; 5];
                let mut num = [0usize; 5];
                for o in &cerca {
                    let k = Dieta::TODAS.iter().position(|&d| d == o.dieta()).unwrap();
                    masa[k] += o.cuerpo() + o.energia.max(0.0);
                    num[k] += 1;
                }
                let maxm = masa.iter().cloned().fold(1.0, f32::max);
                for &k in &[3usize, 2, 1, 0, 4] {
                    let ancho_b = (masa[k] / maxm).sqrt() * 260.0;
                    draw_rectangle(20.0 + (260.0 - ancho_b) / 2.0, y - 13.0, ancho_b.max(2.0), 16.0, color_dieta(Dieta::TODAS[k]));
                    txt(&format!("{} {}", Dieta::TODAS[k].nombre(), num[k]), 295.0, y, 14, LIGHTGRAY);
                    y += 21.0;
                }
                y += 6.0;
                txt("Flujo de energía: sol → productores → herbívoros → carnívoros;", 20.0, y, 13, GRAY);
                y += 16.0;
                txt("los descomponedores devuelven los nutrientes al suelo (ciclo cerrado).", 20.0, y, 13, GRAY);
            }
            Nivel::Bioma => {
                let pos = [objetivo.x, objetivo.y];
                let bio = c.bioma(pos);
                let (r, g, bl) = bio.color();
                draw_rectangle(20.0, y - 16.0, 18.0, 18.0, Color::new(r, g, bl, 1.0));
                txt(bio.nombre(), 46.0, y, 22, WHITE);
                y += 22.0;
                for linea in partir(bio.descripcion(), 58) {
                    txt(&linea, 20.0, y, 14, LIGHTGRAY);
                    y += 17.0;
                }
                y += 6.0;
                txt(&format!("clima: {:.1} °C de media · {:.0} mm de lluvia al año", b.temperatura_en(pos), b.lluvia_en(pos)), 20.0, y, 15, LIGHTGRAY);
                y += 19.0;
                let area = c.areas().iter().find(|x| x.0 == bio).map_or(0.0, |x| x.1);
                txt(&format!("ocupa el {:.1}% del mundo", area * 100.0), 20.0, y, 15, LIGHTGRAY);
                y += 26.0;
                txt("Especies que viven en este bioma", 20.0, y, 17, WHITE);
                y += 20.0;
                let mut cuenta: std::collections::BTreeMap<usize, usize> = Default::default();
                for o in &b.orgs {
                    if c.bioma(o.pos) == bio {
                        *cuenta.entry(o.especie).or_default() += 1;
                    }
                }
                let mut lista: Vec<(usize, usize)> = cuenta.into_iter().collect();
                lista.sort_by(|a, c| c.1.cmp(&a.1));
                for (e, n) in lista.iter().take(12) {
                    txt(&format!("{} · {} · {}", b.especies[*e].nombre, b.especies[*e].cientifico, n), 20.0, y, 14, LIGHTGRAY);
                    y += 17.0;
                }
            }
            Nivel::Biosfera => {
                txt(&format!("{} organismos · {} especies vivas · {} especies nuevas", b.orgs.len(), b.especies_vivas(), b.est.especiaciones), 20.0, y, 15, LIGHTGRAY);
                y += 19.0;
                txt(&format!("nacimientos {} · extinciones {} · migraciones {}", b.est.nacimientos, b.est.extinciones, b.est.migraciones), 20.0, y, 15, LIGHTGRAY);
                y += 19.0;
                txt(&format!("gemelos {} · mellizos {} · clones {} · mutaciones {}", b.est.gemelos, b.est.mellizos, b.est.clones, b.est.mutaciones.total()), 20.0, y, 15, LIGHTGRAY);
                y += 28.0;
                txt("Biomas del planeta (Whittaker)", 20.0, y, 18, WHITE);
                y += 22.0;
                for (bio, a) in c.areas() {
                    let (r, g, bl) = bio.color();
                    draw_rectangle(20.0, y - 12.0, 14.0, 14.0, Color::new(r, g, bl, 1.0));
                    draw_rectangle(200.0, y - 11.0, a * 400.0, 12.0, Color::new(r, g, bl, 0.8));
                    txt(&format!("{} {:.1}%", bio.nombre(), a * 100.0), 42.0, y, 14, LIGHTGRAY);
                    y += 18.0;
                }
                y += 10.0;
                // Gráfica de poblaciones por nivel trófico.
                let (gx, gy, gw, gh) = (20.0, y, 400.0, 100.0);
                draw_rectangle_lines(gx, gy, gw, gh, 1.0, DARKGRAY);
                let hist = &b.historial;
                if hist.len() > 1 {
                    let maxp = hist.iter().flat_map(|v| v.iter()).cloned().max().unwrap_or(1).max(1) as f32;
                    for k in 0..5 {
                        for t in 1..hist.len() {
                            let xa = gx + (t - 1) as f32 / (hist.len() - 1) as f32 * gw;
                            let xb = gx + t as f32 / (hist.len() - 1) as f32 * gw;
                            let ya = gy + gh - (hist[t - 1][k] as f32 / maxp).sqrt() * gh;
                            let yb = gy + gh - (hist[t][k] as f32 / maxp).sqrt() * gh;
                            draw_line(xa, ya, xb, yb, 1.5, color_dieta(Dieta::TODAS[k]));
                        }
                    }
                }
            }
        }
        let _ = TipoBioma::Oceano;

        txt("rueda: subir/bajar de nivel · 1-6 saltar a un nivel · arrastrar: girar · clic der. o WASD: moverse · clic: elegir · F: seguir",
            20.0, h - 38.0, 13, GRAY);
        txt("T/G: ±1 °C global · Y/H: ±lluvia · M/N: ±mutación · +/−: velocidad · espacio: pausa",
            20.0, h - 18.0, 13, GRAY);
        txt(&format!("cámara a {:.0} m", dist), w - 150.0, h - 18.0, 14, Color::new(0.1, 0.1, 0.15, 1.0));
        next_frame().await;
    }
}

fn partir(s: &str, n: usize) -> Vec<String> {
    let mut lineas = Vec::new();
    let mut actual = String::new();
    for p in s.split_whitespace() {
        if actual.chars().count() + p.chars().count() + 1 > n && !actual.is_empty() {
            lineas.push(std::mem::take(&mut actual));
        }
        if !actual.is_empty() {
            actual.push(' ');
        }
        actual.push_str(p);
    }
    if !actual.is_empty() {
        lineas.push(actual);
    }
    lineas
}
