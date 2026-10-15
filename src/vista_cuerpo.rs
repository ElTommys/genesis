//! Dentro de un ser vivo (`genesis cuerpo [humano|conejo|roble|...]`, o Enter sobre un
//! organismo en la biosfera): zoom continuo con tamaños reales por la jerarquía
//! organismo → órgano → tejido → célula → orgánulo → macromolécula → molécula → átomo.
//!
//! Dos rutas dentro de la célula (tecla R):
//!   energía:  mitocondria (o cloroplasto) → ATP sintasa → ATP → átomo de fósforo
//!   genética: núcleo → ADN (gen real de la insulina humana) → adenina → átomo de nitrógeno

use crate::azar::Rng;
use crate::conocimiento::Reino;
use crate::quimica::{self, Molecula};
use crate::vista_atomo::{construir_nube, electrones};
use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum Plan {
    Humano,
    Cuadrupedo,
    Planta,
    Hongo,
}

#[derive(Clone, Copy, PartialEq)]
enum Tejido {
    Cardiaco,
    Nervioso,
    Epitelial,
    Alveolar,
    Hepatico,
    Sangre,
    Muscular,
    Parenquima,
    Xilema,
    Radicular,
    Hifas,
}

struct Organo {
    nombre: &'static str,
    sistema: &'static str,
    /// Posición y radio como fracción del largo del cuerpo.
    pos: [f64; 3],
    radio: f64,
    color: Color,
    tejido: Tejido,
    dato: &'static str,
}

fn organos(plan: Plan) -> Vec<Organo> {
    let c = |r, g, b| Color::new(r, g, b, 1.0);
    match plan {
        Plan::Humano => vec![
            Organo { nombre: "Corazón", sistema: "circulatorio", pos: [0.02, 0.70, 0.02], radio: 0.034, color: c(0.8, 0.12, 0.15), tejido: Tejido::Cardiaco, dato: "Late ~70 veces por minuto y bombea ~5 litros de sangre por minuto (unos 100.000 latidos al día)." },
            Organo { nombre: "Cerebro", sistema: "nervioso", pos: [0.0, 0.935, 0.0], radio: 0.045, color: c(0.9, 0.7, 0.72), tejido: Tejido::Nervioso, dato: "~86.000 millones de neuronas; consume ~20% de la energía del cuerpo con ~2% de su masa." },
            Organo { nombre: "Pulmones", sistema: "respiratorio", pos: [0.065, 0.715, 0.0], radio: 0.055, color: c(0.95, 0.6, 0.65), tejido: Tejido::Alveolar, dato: "~480 millones de alvéolos: ~70 m² de superficie para intercambiar O₂ y CO₂." },
            Organo { nombre: "Hígado", sistema: "digestivo", pos: [-0.05, 0.61, 0.025], radio: 0.048, color: c(0.5, 0.15, 0.12), tejido: Tejido::Hepatico, dato: "Más de 500 funciones: guarda glucógeno, fabrica bilis, desintoxica la sangre." },
            Organo { nombre: "Estómago", sistema: "digestivo", pos: [0.055, 0.61, 0.03], radio: 0.038, color: c(0.9, 0.55, 0.45), tejido: Tejido::Epitelial, dato: "Ácido clorhídrico (pH 1.5-3.5) y pepsina rompen las proteínas de los alimentos." },
            Organo { nombre: "Riñón", sistema: "excretor", pos: [0.045, 0.58, -0.035], radio: 0.022, color: c(0.6, 0.2, 0.2), tejido: Tejido::Epitelial, dato: "~1 millón de nefronas cada uno; filtran ~180 litros de plasma al día." },
            Organo { nombre: "Sangre (aorta)", sistema: "circulatorio", pos: [-0.01, 0.75, 0.0], radio: 0.012, color: c(0.75, 0.05, 0.08), tejido: Tejido::Sangre, dato: "~5 litros; cada microlitro tiene ~5 millones de glóbulos rojos." },
            Organo { nombre: "Músculo (cuádriceps)", sistema: "muscular", pos: [0.06, 0.33, 0.02], radio: 0.04, color: c(0.7, 0.25, 0.25), tejido: Tejido::Muscular, dato: "~600 músculos esqueléticos; la contracción gasta ATP en cada ciclo de miosina." },
        ],
        Plan::Cuadrupedo => vec![
            Organo { nombre: "Corazón", sistema: "circulatorio", pos: [0.24, 0.42, 0.0], radio: 0.045, color: c(0.8, 0.12, 0.15), tejido: Tejido::Cardiaco, dato: "Corazón de 4 cavidades como el humano; los animales pequeños laten más rápido." },
            Organo { nombre: "Cerebro", sistema: "nervioso", pos: [0.6, 0.63, 0.0], radio: 0.045, color: c(0.9, 0.7, 0.72), tejido: Tejido::Nervioso, dato: "Coordina el movimiento, los sentidos y el instinto." },
            Organo { nombre: "Pulmones", sistema: "respiratorio", pos: [0.2, 0.47, 0.07], radio: 0.07, color: c(0.95, 0.6, 0.65), tejido: Tejido::Alveolar, dato: "Intercambio de gases en los alvéolos." },
            Organo { nombre: "Hígado", sistema: "digestivo", pos: [0.05, 0.42, -0.04], radio: 0.06, color: c(0.5, 0.15, 0.12), tejido: Tejido::Hepatico, dato: "Metaboliza los nutrientes absorbidos en el intestino." },
            Organo { nombre: "Estómago", sistema: "digestivo", pos: [-0.02, 0.45, 0.05], radio: 0.05, color: c(0.9, 0.55, 0.45), tejido: Tejido::Epitelial, dato: "En los rumiantes (venado, reno) tiene 4 cámaras para digerir celulosa." },
            Organo { nombre: "Sangre (aorta)", sistema: "circulatorio", pos: [0.15, 0.5, 0.0], radio: 0.015, color: c(0.75, 0.05, 0.08), tejido: Tejido::Sangre, dato: "Transporta O₂, nutrientes y desechos." },
            Organo { nombre: "Músculo", sistema: "muscular", pos: [-0.28, 0.32, 0.08], radio: 0.05, color: c(0.7, 0.25, 0.25), tejido: Tejido::Muscular, dato: "Las patas traseras dan el impulso para correr." },
        ],
        Plan::Planta => vec![
            Organo { nombre: "Hoja", sistema: "fotosintético", pos: [0.22, 0.82, 0.0], radio: 0.05, color: c(0.2, 0.65, 0.2), tejido: Tejido::Parenquima, dato: "Fotosíntesis: 6CO₂ + 6H₂O + luz → C₆H₁₂O₆ + 6O₂. Los estomas regulan el intercambio de gases." },
            Organo { nombre: "Tallo", sistema: "vascular", pos: [0.0, 0.4, 0.0], radio: 0.04, color: c(0.45, 0.32, 0.18), tejido: Tejido::Xilema, dato: "Xilema: sube agua y minerales; floema: reparte los azúcares de la fotosíntesis." },
            Organo { nombre: "Raíz", sistema: "absorción", pos: [0.0, -0.08, 0.0], radio: 0.04, color: c(0.6, 0.48, 0.3), tejido: Tejido::Radicular, dato: "Los pelos radicales multiplican la superficie para absorber agua y nutrientes." },
        ],
        Plan::Hongo => vec![
            Organo { nombre: "Cuerpo fructífero", sistema: "reproductor", pos: [0.0, 0.6, 0.0], radio: 0.15, color: c(0.85, 0.75, 0.6), tejido: Tejido::Hifas, dato: "Libera millones de esporas; el hongo real es el micelio bajo tierra." },
        ],
    }
}

impl Tejido {
    fn nombre(self) -> &'static str {
        match self {
            Tejido::Cardiaco => "Tejido muscular cardíaco",
            Tejido::Nervioso => "Tejido nervioso",
            Tejido::Epitelial => "Tejido epitelial",
            Tejido::Alveolar => "Epitelio alveolar + capilares",
            Tejido::Hepatico => "Tejido hepático (hepatocitos)",
            Tejido::Sangre => "Tejido sanguíneo (conectivo líquido)",
            Tejido::Muscular => "Tejido muscular esquelético",
            Tejido::Parenquima => "Parénquima clorofílico y estomas",
            Tejido::Xilema => "Tejido vascular: xilema y floema",
            Tejido::Radicular => "Epidermis radicular",
            Tejido::Hifas => "Micelio: hifas",
        }
    }
    fn dato(self) -> &'static str {
        match self {
            Tejido::Cardiaco => "Células ramificadas unidas por discos intercalares: se contraen todas juntas.",
            Tejido::Nervioso => "Neuronas (impulsos eléctricos a ~100 m/s) y células gliales que las sostienen.",
            Tejido::Epitelial => "Células muy juntas que recubren y protegen; aquí secretan y absorben.",
            Tejido::Alveolar => "Paredes de 0.2 µm: el O₂ pasa por difusión a la sangre y el CO₂ sale.",
            Tejido::Hepatico => "Placas de hepatocitos entre sinusoides por donde corre la sangre.",
            Tejido::Sangre => "Glóbulos rojos (sin núcleo, llevan hemoglobina), glóbulos blancos (defensa) y plaquetas (coagulación).",
            Tejido::Muscular => "Fibras largas con estrías: actina y miosina se deslizan gastando ATP.",
            Tejido::Parenquima => "Células llenas de cloroplastos; los estomas se abren para tomar CO₂ y soltar O₂ y vapor.",
            Tejido::Xilema => "Tubos de células muertas con lignina que suben agua por capilaridad y transpiración.",
            Tejido::Radicular => "Pelos radicales: prolongaciones finas que absorben agua por ósmosis.",
            Tejido::Hifas => "Filamentos de células alargadas con pared de quitina.",
        }
    }
}

/// Inicio del gen humano de la insulina (INS, secuencia codificante real).
const GEN_INSULINA: &str = "ATGGCCCTGTGGATGCGCCTCCTGCCCCTGCTGGCGCTGCTGGCCCTCTGGGGACCTGACCCAGCCGCAGCCTTTGTGAACCAACACCTGTGCGGCTCACACCTGGTGGAAGCTCTCTACCTAGTGTGCGGGGAACGAGGCTTCTTCTACACACCCAAGACC";

#[derive(Clone, Copy, PartialEq)]
enum Nivel {
    Organismo,
    Organo,
    Tejido,
    Celula,
    Organulo,
    Macromolecula,
    Molecula,
    Atomo,
}

const NIVELES: [Nivel; 8] = [Nivel::Organismo, Nivel::Organo, Nivel::Tejido, Nivel::Celula, Nivel::Organulo, Nivel::Macromolecula, Nivel::Molecula, Nivel::Atomo];

impl Nivel {
    fn nombre(self) -> &'static str {
        match self {
            Nivel::Organismo => "ORGANISMO",
            Nivel::Organo => "ÓRGANO · SISTEMA",
            Nivel::Tejido => "TEJIDO",
            Nivel::Celula => "CÉLULA",
            Nivel::Organulo => "ORGÁNULO",
            Nivel::Macromolecula => "MACROMOLÉCULA",
            Nivel::Molecula => "MOLÉCULA",
            Nivel::Atomo => "ÁTOMO",
        }
    }
}

pub struct Interior {
    pub plan: Plan,
    pub nombre: String,
    pub cientifico: String,
    largo: f64,
    organos: Vec<Organo>,
    organo: usize,
    ruta_energia: bool,
    dist: f64,
    dist_obj: f64,
    yaw: f32,
    pitch: f32,
    t: f32,
    atp: Molecula,
    adenina: Molecula,
    nubes: Vec<(u32, Vec<[f64; 3]>)>,
    ultimo: (f32, f32),
}

impl Interior {
    pub fn desde(nombre: &str, cientifico: &str, clase: &str, reino: Reino, masa_kg: f64) -> Interior {
        let plan = match reino {
            Reino::Plantae => Plan::Planta,
            Reino::Fungi => Plan::Hongo,
            Reino::Animalia if cientifico.starts_with("Homo") || clase.contains("primate") => Plan::Humano,
            Reino::Animalia => Plan::Cuadrupedo,
        };
        // Largo corporal ≈ 0.42·m^(1/3) (alometría): humano de 70 kg → 1.73 m.
        let largo = match plan {
            Plan::Planta => (masa_kg.max(0.01).cbrt() * 0.9).clamp(0.3, 40.0),
            Plan::Hongo => 0.1,
            _ => (0.42 * masa_kg.max(0.5).cbrt()).clamp(0.2, 4.0),
        };
        let atp = quimica::leer_smiles("ATP", quimica::buscar("ATP").unwrap().1).unwrap();
        let adenina = quimica::leer_smiles("adenina", quimica::buscar("adenina").unwrap().1).unwrap();
        let mut rng = Rng::nuevo(5);
        let nubes = [7u32, 15].iter().map(|&z| {
            let n = construir_nube(z, &electrones(z), &mut rng);
            (z, n.puntos.iter().map(|p| p.0).collect())
        }).collect();
        Interior {
            plan,
            nombre: nombre.to_string(),
            cientifico: cientifico.to_string(),
            largo,
            organos: organos(plan),
            organo: 0,
            ruta_energia: true,
            dist: largo * 2.2,
            dist_obj: largo * 2.2,
            yaw: 0.8,
            pitch: 0.25,
            t: 0.0,
            atp,
            adenina,
            nubes,
            ultimo: (0.0, 0.0),
        }
    }

    pub fn humano() -> Interior {
        Interior::desde("Ser humano", "Homo sapiens", "Mamífero primate", Reino::Animalia, 70.0)
    }

    fn vegetal(&self) -> bool {
        self.plan == Plan::Planta
    }

    /// Tamaño característico (m) de cada nivel para el órgano y la ruta actuales.
    fn tamano(&self, n: Nivel) -> f64 {
        let o = &self.organos[self.organo];
        match n {
            Nivel::Organismo => self.largo,
            Nivel::Organo => o.radio * 2.0 * self.largo,
            Nivel::Tejido => 4e-4,
            Nivel::Celula => if self.vegetal() { 4e-5 } else { 2.5e-5 },
            Nivel::Organulo => if self.ruta_energia { if self.vegetal() { 6e-6 } else { 2.5e-6 } } else { 7e-6 },
            Nivel::Macromolecula => 2.5e-8,
            Nivel::Molecula => 1.6e-9,
            Nivel::Atomo => 3e-10,
        }
    }

    // Focos anidados (en metros): cada nivel está dentro del anterior.
    fn foco_organo(&self) -> [f64; 3] {
        let o = &self.organos[self.organo];
        [o.pos[0] * self.largo, o.pos[1] * self.largo, o.pos[2] * self.largo]
    }
    fn desplaz_organulo(&self) -> [f64; 3] {
        if self.ruta_energia { [4e-6, -1e-6, 1.5e-6] } else { [-3e-6, 0.5e-6, 0.0] }
    }
    fn desplaz_macro(&self) -> [f64; 3] {
        if self.ruta_energia { [0.3e-6, 0.18e-6, 0.0] } else { [0.8e-6, 0.4e-6, 0.3e-6] }
    }
    fn mol(&self) -> &Molecula {
        if self.ruta_energia { &self.atp } else { &self.adenina }
    }
    /// Átomo foco dentro de la molécula: el primer P del ATP o el primer N de la adenina.
    fn atomo_foco(&self) -> (u32, [f64; 3]) {
        let z = if self.ruta_energia { 15 } else { 7 };
        let a = self.mol().atomos.iter().find(|a| a.z == z).unwrap();
        (z, [a.pos[0] * 1e-10, a.pos[1] * 1e-10, a.pos[2] * 1e-10])
    }
    fn focos(&self) -> [[f64; 3]; 8] {
        let o = self.foco_organo();
        let suma = |a: [f64; 3], b: [f64; 3]| [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
        let org = suma(o, self.desplaz_organulo());
        let mac = suma(org, self.desplaz_macro());
        let mol = suma(mac, [3e-9, 2e-9, 0.0]);
        let at = suma(mol, self.atomo_foco().1);
        let centro = match self.plan {
            Plan::Humano => [0.0, 0.5 * self.largo, 0.0],
            Plan::Cuadrupedo => [0.1 * self.largo, 0.4 * self.largo, 0.0],
            Plan::Planta => [0.0, 0.45 * self.largo, 0.0],
            Plan::Hongo => [0.0, 0.5 * self.largo, 0.0],
        };
        [centro, o, o, o, org, mac, mol, at]
    }

    fn nivel_actual(&self) -> (usize, f64) {
        // Índice fraccionario en escala logarítmica entre tamaños de niveles.
        let s: Vec<f64> = NIVELES.iter().map(|&n| self.tamano(n) * 1.3).collect();
        let d = self.dist;
        for i in 0..7 {
            if d > s[i + 1] {
                let f = ((s[i] / d).ln() / (s[i] / s[i + 1]).ln()).clamp(0.0, 1.0);
                return (i, f);
            }
        }
        (7, 0.0)
    }

    /// Un fotograma. Devuelve `false` si el usuario quiere salir (Esc).
    pub fn frame(&mut self, fuente: Option<&Font>) -> bool {
        let txt = |s: &str, x: f32, y: f32, tam: u16, c: Color| {
            draw_text_ex(s, x, y, TextParams { font: fuente, font_size: tam, color: c, ..Default::default() });
        };
        if is_key_pressed(KeyCode::Escape) {
            return false;
        }
        if is_key_pressed(KeyCode::Right) { self.organo = (self.organo + 1) % self.organos.len(); }
        if is_key_pressed(KeyCode::Left) { self.organo = (self.organo + self.organos.len() - 1) % self.organos.len(); }
        if is_key_pressed(KeyCode::R) { self.ruta_energia = !self.ruta_energia; }
        for (k, i) in [(KeyCode::Key1, 0), (KeyCode::Key2, 1), (KeyCode::Key3, 2), (KeyCode::Key4, 3), (KeyCode::Key5, 4), (KeyCode::Key6, 5), (KeyCode::Key7, 6), (KeyCode::Key8, 7)] {
            if is_key_pressed(k) {
                self.dist_obj = self.tamano(NIVELES[i]) * if i == 0 { 2.2 } else { 1.6 };
            }
        }
        let raton = mouse_position();
        if is_mouse_button_down(MouseButton::Left) && raton.0 > 440.0 {
            self.yaw += (raton.0 - self.ultimo.0) * 0.007;
            self.pitch = (self.pitch + (raton.1 - self.ultimo.1) * 0.007).clamp(-1.4, 1.4);
        }
        self.ultimo = raton;
        let rueda = mouse_wheel().1.clamp(-3.0, 3.0);
        if rueda != 0.0 && raton.0 > 440.0 {
            self.dist_obj *= 1.15f64.powf(-rueda as f64);
        }
        if is_key_down(KeyCode::Equal) { self.dist_obj *= 0.96; }
        if is_key_down(KeyCode::Minus) { self.dist_obj *= 1.04; }
        self.dist_obj = self.dist_obj.clamp(2.5e-10, self.largo * 4.0);
        self.dist *= (self.dist_obj / self.dist).powf(0.12);
        self.t += get_frame_time();

        let (nivel_i, f) = self.nivel_actual();
        let focos = self.focos();
        let a = focos[nivel_i];
        let b = focos[(nivel_i + 1).min(7)];
        let objetivo = [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f, a[2] + (b[2] - a[2]) * f];
        let k = 10.0 / self.dist;
        let v = |p: [f64; 3]| vec3(((p[0] - objetivo[0]) * k) as f32, ((p[1] - objetivo[1]) * k) as f32, ((p[2] - objetivo[2]) * k) as f32);
        let r = |m: f64| (m * k) as f32;

        clear_background(Color::from_rgba(10, 8, 14, 255));
        let ojo = vec3(self.pitch.cos() * self.yaw.cos(), self.pitch.sin(), self.pitch.cos() * self.yaw.sin()) * 10.0;
        set_camera(&Camera3D { position: ojo, target: Vec3::ZERO, up: Vec3::Y, z_near: 0.01, z_far: 3000.0, ..Default::default() });
        // Solo se dibujan los niveles cuya escala tiene sentido desde aquí.
        let visible = |n: Nivel| {
            let s = self.tamano(n);
            s / self.dist < 40.0 && s / self.dist > 0.004
        };
        let t = self.t;
        let mut rng = Rng::nuevo(99);

        // ── Organismo ──
        if visible(Nivel::Organismo) {
            let l = self.largo;
            let piel = Color::new(0.95, 0.78, 0.65, 0.16);
            let p = |x: f64, y: f64, z: f64| v([x * l, y * l, z * l]);
            match self.plan {
                Plan::Humano => {
                    draw_sphere(p(0.0, 0.93, 0.0), r(0.065 * l), None, piel);
                    barra(p(0.0, 0.52, 0.0), p(0.0, 0.8, 0.0), r(0.1 * l), piel);
                    for s in [-1.0, 1.0] {
                        barra(p(0.12 * s, 0.79, 0.0), p(0.16 * s, 0.42, 0.02), r(0.026 * l), piel);
                        barra(p(0.055 * s, 0.5, 0.0), p(0.06 * s, 0.02, 0.0), r(0.042 * l), piel);
                    }
                }
                Plan::Cuadrupedo => {
                    barra(p(-0.35, 0.45, 0.0), p(0.32, 0.45, 0.0), r(0.15 * l), piel);
                    barra(p(0.3, 0.5, 0.0), p(0.52, 0.6, 0.0), r(0.06 * l), piel);
                    draw_sphere(p(0.6, 0.62, 0.0), r(0.08 * l), None, piel);
                    for (x, z) in [(0.25, 0.08), (0.25, -0.08), (-0.27, 0.08), (-0.27, -0.08)] {
                        barra(p(x, 0.38, z), p(x, 0.0, z), r(0.035 * l), piel);
                    }
                }
                Plan::Planta => {
                    let tronco = Color::new(0.45, 0.32, 0.18, 0.5);
                    barra(p(0.0, -0.1, 0.0), p(0.0, 0.7, 0.0), r(0.03 * l), tronco);
                    draw_sphere(p(0.0, 0.82, 0.0), r(0.26 * l), None, Color::new(0.2, 0.6, 0.2, 0.18));
                    for i in 0..6 {
                        let a = i as f64 * 1.05;
                        barra(p(0.0, 0.0, 0.0), p(0.2 * a.cos(), -0.15, 0.2 * a.sin()), r(0.008 * l), tronco);
                    }
                }
                Plan::Hongo => {
                    barra(p(0.0, 0.0, 0.0), p(0.0, 0.55, 0.0), r(0.08 * l), Color::new(0.95, 0.92, 0.85, 0.4));
                    draw_sphere(p(0.0, 0.62, 0.0), r(0.25 * l), None, Color::new(0.75, 0.55, 0.35, 0.3));
                }
            }
            // Órganos: el corazón late (70 lpm) y los pulmones respiran (15 por minuto).
            for (i, o) in self.organos.iter().enumerate() {
                let mut rad = o.radio * l;
                if o.nombre == "Corazón" { rad *= 1.0 + 0.08 * (t * std::f32::consts::TAU * 70.0 / 60.0).sin().max(0.0) as f64; }
                if o.nombre == "Pulmones" { rad *= 1.0 + 0.06 * (t * std::f32::consts::TAU * 15.0 / 60.0).sin() as f64; }
                let mut c = o.color;
                c.a = if i == self.organo { 1.0 } else { 0.55 };
                let pos = [o.pos[0] * l, o.pos[1] * l, o.pos[2] * l];
                draw_sphere(v(pos), r(rad), None, c);
                if o.nombre == "Pulmones" || o.nombre == "Riñón" {
                    draw_sphere(v([pos[0] - 2.0 * o.pos[0] * l, pos[1], pos[2]]), r(rad), None, c);
                }
                if i == self.organo {
                    draw_sphere_wires(v(pos), r(rad * 1.25), None, Color::new(1.0, 1.0, 0.5, 0.35));
                }
            }
        }

        // ── Tejido (centrado en el órgano) ──
        let fo = focos[1];
        let tejido = self.organos[self.organo].tejido;
        if visible(Nivel::Tejido) {
            dibujar_tejido(tejido, fo, &v, &r, t, &mut rng);
        }
        // ── Célula ──
        if visible(Nivel::Celula) {
            dibujar_celula(self.vegetal(), fo, self.desplaz_organulo(), &v, &r, t, &mut rng);
        }
        // ── Orgánulo ──
        let forg = focos[4];
        if visible(Nivel::Organulo) {
            if self.ruta_energia {
                dibujar_mitocondria(self.vegetal(), forg, self.desplaz_macro(), &v, &r);
            } else {
                dibujar_nucleo(forg, self.desplaz_macro(), &v, &r, t);
            }
        }
        // ── Macromolécula ──
        let fmac = focos[5];
        if visible(Nivel::Macromolecula) {
            if self.ruta_energia {
                dibujar_atp_sintasa(fmac, &v, &r, t);
            } else {
                dibujar_adn(fmac, &v, &r, t);
            }
        }
        // ── Molécula ──
        let fmol = focos[6];
        if visible(Nivel::Molecula) {
            let m = self.mol();
            let p = |q: [f64; 3]| v([fmol[0] + q[0] * 1e-10, fmol[1] + q[1] * 1e-10, fmol[2] + q[2] * 1e-10]);
            for e in &m.enlaces {
                barra(p(m.atomos[e.a].pos), p(m.atomos[e.b].pos), r(0.12e-10), Color::new(0.8, 0.8, 0.85, 1.0));
            }
            for at in &m.atomos {
                let (cr, cg, cb) = crate::elementos::color_cpk(at.z);
                draw_sphere(p(at.pos), r(quimica::radio_covalente_pm(at.z) * 0.45e-12 + 0.08e-10), None, Color::from_rgba(cr, cg, cb, 255));
            }
        }
        // ── Átomo ──
        let fat = focos[7];
        if visible(Nivel::Atomo) {
            let z = self.atomo_foco().0;
            if let Some((_, pts)) = self.nubes.iter().find(|n| n.0 == z) {
                for q in pts.iter().step_by(2) {
                    let pos = v([fat[0] + q[0], fat[1] + q[1], fat[2] + q[2]]);
                    if pos.length() < 400.0 {
                        draw_cube(pos, Vec3::splat(0.02 * pos.length().clamp(1.0, 20.0)), None, Color::new(0.4, 0.75, 1.0, 0.3));
                    }
                }
                draw_sphere(v(fat), 0.03, None, Color::new(1.0, 0.85, 0.3, 1.0));
            }
        }

        // ── Panel ──
        set_default_camera();
        let h = screen_height();
        draw_rectangle(0.0, 0.0, 440.0, h, Color::from_rgba(10, 10, 22, 238));
        let mut y = 34.0;
        txt(&format!("{} · {}", self.nombre, self.cientifico), 20.0, y, 22, WHITE);
        y += 20.0;
        txt(&format!("ruta: {}   ·   órgano: {}", if self.ruta_energia { "ENERGÍA (ATP)" } else { "GENÉTICA (ADN)" }, self.organos[self.organo].nombre), 20.0, y, 14, GRAY);
        y += 24.0;
        for (i, n) in NIVELES.iter().enumerate() {
            let activo = i == nivel_i;
            let x = 20.0 + (i % 4) as f32 * 101.0;
            let yy = y + (i / 4) as f32 * 22.0;
            draw_rectangle(x, yy, 97.0, 19.0, if activo { Color::new(1.0, 0.85, 0.35, 1.0) } else { Color::from_rgba(40, 45, 70, 255) });
            let corto: String = n.nombre().chars().take(11).collect();
            txt(&format!("{} {}", i + 1, corto), x + 5.0, yy + 14.0, 12, if activo { BLACK } else { GRAY });
        }
        y += 60.0;
        let nivel = NIVELES[nivel_i];
        txt(nivel.nombre(), 20.0, y, 28, Color::new(1.0, 0.85, 0.35, 1.0));
        y += 20.0;
        txt(&format!("tamaño típico: {}", metros(self.tamano(nivel))), 20.0, y, 15, LIGHTGRAY);
        y += 26.0;
        let o = &self.organos[self.organo];
        let lineas: Vec<String> = match nivel {
            Nivel::Organismo => vec![
                format!("Largo corporal: {}.", metros(self.largo)),
                match self.plan {
                    Plan::Humano => "Un humano adulto tiene ~37 billones (3.7×10¹³) de células de más de 200 tipos.".into(),
                    Plan::Cuadrupedo => "Mamífero: los mismos sistemas que el humano, con otras proporciones.".into(),
                    Plan::Planta => "Planta: crece toda la vida desde sus meristemos; no tiene órganos móviles.".into(),
                    Plan::Hongo => "Hongo: lo que vemos es solo el cuerpo fructífero.".into(),
                },
                format!("Órganos aquí: {}.", self.organos.iter().map(|o| o.nombre).collect::<Vec<_>>().join(", ")),
                "Usa ←/→ para elegir órgano; la rueda baja al interior.".into(),
            ],
            Nivel::Organo => vec![
                format!("{} · sistema {}.", o.nombre, o.sistema),
                o.dato.into(),
                format!("Está hecho de: {}.", tejido.nombre().to_lowercase()),
            ],
            Nivel::Tejido => vec![tejido.nombre().into(), tejido.dato().into(), "Un tejido = células parecidas que trabajan juntas.".into()],
            Nivel::Celula => if self.vegetal() {
                vec![
                    "Célula vegetal (~40 µm): pared de celulosa, gran vacuola central, cloroplastos.".into(),
                    "Fotosíntesis en los cloroplastos; respiración en las mitocondrias (también las tiene).".into(),
                    "La vacuola guarda agua y mantiene la célula turgente (presión de turgencia).".into(),
                ]
            } else {
                vec![
                    "Célula animal (~20 µm): membrana, núcleo, mitocondrias, retículo, Golgi, lisosomas.".into(),
                    "Bomba Na⁺/K⁺ (morado): por cada ATP saca 3 Na⁺ (amarillo) y mete 2 K⁺ (violeta).".into(),
                    "Esa bomba gasta ~20-25% del ATP en reposo (hasta ~70% en neuronas).".into(),
                    "Ribosomas (puntos): sintetizan proteínas a ~6 aminoácidos por segundo.".into(),
                ]
            },
            Nivel::Organulo => if self.ruta_energia {
                if self.vegetal() {
                    vec!["Cloroplasto (~5 µm): tilacoides apilados en granas.".into(), "La luz excita la clorofila → cadena de electrones → bombea H⁺.".into(), "La ATP sintasa usa ese gradiente para fabricar ATP; el ciclo de Calvin fija CO₂ en azúcar.".into()]
                } else {
                    vec![
                        "Mitocondria (~1-2 µm): la central de energía. Tiene su propio ADN (herencia materna).".into(),
                        "Glucólisis (citoplasma): glucosa → 2 piruvato + 2 ATP.".into(),
                        "Ciclo de Krebs (matriz): cada acetil-CoA da 3 NADH, 1 FADH₂, 1 GTP y 2 CO₂.".into(),
                        "Cadena respiratoria (crestas): bombea H⁺; el O₂ recibe los electrones y forma H₂O.".into(),
                        "Balance: ~30-32 ATP por cada glucosa.".into(),
                    ]
                }
            } else {
                vec![
                    "Núcleo (~6 µm): guarda el ADN; poros nucleares dejan salir el ARN mensajero.".into(),
                    "Humano: 46 cromosomas, ~3.100 millones de pares de bases, ~2 m de ADN por célula.".into(),
                    "Transcripción: ADN → ARNm; luego los ribosomas lo traducen a proteína.".into(),
                ]
            },
            Nivel::Macromolecula => if self.ruta_energia {
                vec![
                    "ATP sintasa (~10 nm): un motor rotatorio molecular.".into(),
                    "Los protones que pasan hacen girar el rotor ~100-150 vueltas por segundo.".into(),
                    "Cada vuelta fabrica 3 ATP (ADP + Pi → ATP). Nobel de Química 1997.".into(),
                ]
            } else {
                let prot = crate::vida::traducir(GEN_INSULINA);
                vec![
                    "ADN: doble hélice de 2 nm de ancho, 3.4 nm por vuelta, ~10.5 pares por vuelta.".into(),
                    "Pares A–T (2 puentes de hidrógeno) y G–C (3). Aquí: el gen real de la insulina.".into(),
                    format!("Traducido con el código genético: {}…", &prot[..prot.len().min(40)]),
                    "Esa proteína regula el azúcar en la sangre.".into(),
                ]
            },
            Nivel::Molecula => {
                let m = self.mol();
                vec![
                    format!("{} · {} · {:.2} g/mol", m.nombre, m.formula(), m.masa_molar()),
                    if self.ruta_energia { "ATP: la moneda energética. Romper el último fosfato libera ~30.5 kJ/mol.".into() } else { "Adenina: base púrica del ADN, siempre se empareja con la timina.".into() },
                    "Geometría calculada con radios covalentes y VSEPR.".into(),
                ]
            }
            Nivel::Atomo => {
                let (z, _) = self.atomo_foco();
                let a = crate::cuantica::Atomo::nuevo(z);
                vec![
                    format!("{} ({}): Z = {}, {}.", a.nombre(), a.simbolo(), z, a.configuracion()),
                    "La nube es |ψ|²: dónde es probable encontrar cada electrón.".into(),
                    "Para seguir hacia el núcleo y los quarks: genesis atomo.".into(),
                ]
            }
        };
        for l in lineas {
            for p in partir(&l, 60) {
                txt(&p, 20.0, y, 15, LIGHTGRAY);
                y += 18.0;
            }
            y += 4.0;
        }
        txt("rueda: bajar o subir · 1-8 saltar a un nivel · ←/→ órgano · R ruta energía/genética · arrastrar: girar · Esc: volver",
            20.0, h - 16.0, 13, GRAY);
        true
    }
}

fn metros(m: f64) -> String {
    match m {
        x if x >= 1.0 => format!("{x:.2} m"),
        x if x >= 1e-2 => format!("{:.1} cm", x * 100.0),
        x if x >= 1e-3 => format!("{:.1} mm", x * 1e3),
        x if x >= 1e-6 => format!("{:.0} µm", x * 1e6),
        x if x >= 1e-9 => format!("{:.0} nm", x * 1e9),
        x => format!("{:.1} Å", x * 1e10),
    }
}

/// Barra 3D como cadena de esferas (macroquad no orienta cilindros).
fn barra(a: Vec3, b: Vec3, r: f32, c: Color) {
    if r <= 0.0 || (a.length() > 2000.0 && b.length() > 2000.0) {
        return;
    }
    let n = ((a - b).length() / (r * 0.8).max(1e-4)).ceil().clamp(2.0, 60.0) as usize;
    for i in 0..=n {
        draw_sphere(a.lerp(b, i as f32 / n as f32), r, None, c);
    }
}

fn dibujar_tejido(tejido: Tejido, c: [f64; 3], v: &dyn Fn([f64; 3]) -> Vec3, r: &dyn Fn(f64) -> f32, t: f32, rng: &mut Rng) {
    let p = |x: f64, y: f64, z: f64| v([c[0] + x, c[1] + y, c[2] + z]);
    let um = 1e-6;
    match tejido {
        Tejido::Cardiaco | Tejido::Muscular => {
            let largo = if tejido == Tejido::Cardiaco { 100.0 } else { 300.0 };
            let ancho = if tejido == Tejido::Cardiaco { 18.0 } else { 50.0 };
            let latido = if tejido == Tejido::Cardiaco { 1.0 - 0.05 * (t * 7.33).sin().max(0.0) as f64 } else { 1.0 };
            for fila in -3..=3 {
                for col in -2..=2 {
                    let y = fila as f64 * ancho * 1.15 * um;
                    let x = (col as f64 * largo * 1.05 + if fila % 2 == 0 { 0.0 } else { largo * 0.5 }) * um;
                    barra(p(x - largo * 0.5 * latido * um, y, 0.0), p(x + largo * 0.5 * latido * um, y, 0.0), r(ancho * 0.5 * um), Color::new(0.75, 0.2, 0.22, 1.0));
                    for s in 0..8 {
                        let xs = x + (s as f64 - 3.5) * largo / 8.0 * um;
                        barra(p(xs, y - ancho * 0.52 * um, ancho * 0.3 * um), p(xs, y + ancho * 0.52 * um, ancho * 0.3 * um), r(0.6 * um), Color::new(0.45, 0.08, 0.1, 1.0));
                    }
                    if tejido == Tejido::Cardiaco {
                        draw_sphere(p(x + largo * 0.5 * um, y, 0.0), r(ancho * 0.25 * um), None, Color::new(1.0, 0.9, 0.9, 1.0));
                    }
                }
            }
        }
        Tejido::Nervioso => {
            for i in 0..14 {
                let cx = (rng.unif() - 0.5) * 350.0 * um;
                let cy = (rng.unif() - 0.5) * 350.0 * um;
                let cz = (rng.unif() - 0.5) * 120.0 * um;
                let soma = if i == 0 { [0.0, 0.0, 0.0] } else { [cx, cy, cz] };
                draw_sphere(p(soma[0], soma[1], soma[2]), r(10.0 * um), None, Color::new(0.85, 0.75, 0.9, 1.0));
                for _ in 0..6 {
                    let a = rng.unif() * std::f64::consts::TAU;
                    let l = (30.0 + rng.unif() * 60.0) * um;
                    let fin = [soma[0] + a.cos() * l, soma[1] + a.sin() * l, soma[2] + (rng.unif() - 0.5) * l];
                    barra(p(soma[0], soma[1], soma[2]), p(fin[0], fin[1], fin[2]), r(1.2 * um), Color::new(0.7, 0.6, 0.85, 1.0));
                }
                // Axón con impulso eléctrico viajando.
                let fin = [soma[0] + 180.0 * um, soma[1] - 40.0 * um, soma[2]];
                barra(p(soma[0], soma[1], soma[2]), p(fin[0], fin[1], fin[2]), r(1.5 * um), Color::new(0.95, 0.9, 0.6, 1.0));
                let f = ((t * 0.8 + i as f32 * 0.37).fract()) as f64;
                draw_sphere(p(soma[0] + (fin[0] - soma[0]) * f, soma[1] + (fin[1] - soma[1]) * f, soma[2]), r(4.0 * um), None, Color::new(1.0, 1.0, 0.3, 1.0));
            }
        }
        Tejido::Epitelial | Tejido::Hepatico | Tejido::Radicular => {
            let lado = if tejido == Tejido::Hepatico { 25.0 } else { 20.0 };
            let color = match tejido {
                Tejido::Hepatico => Color::new(0.6, 0.25, 0.2, 1.0),
                Tejido::Radicular => Color::new(0.8, 0.72, 0.5, 1.0),
                _ => Color::new(0.95, 0.65, 0.6, 1.0),
            };
            for i in -6..=6 {
                for j in -6..=6 {
                    let x = (i as f64 + if j % 2 == 0 { 0.0 } else { 0.5 }) * lado * 1.08 * um;
                    let z = j as f64 * lado * 0.95 * um;
                    draw_cube(p(x, 0.0, z), Vec3::new(r(lado * um), r(lado * 1.6 * um), r(lado * 0.9 * um)), None, color);
                    draw_sphere(p(x, lado * 0.3 * um, z), r(lado * 0.22 * um), None, Color::new(0.35, 0.25, 0.55, 1.0));
                    if tejido == Tejido::Radicular && (i + j) % 3 == 0 {
                        barra(p(x, -lado * 0.8 * um, z), p(x + 15.0 * um, -lado * 6.0 * um, z), r(2.0 * um), color);
                    }
                }
            }
            if tejido == Tejido::Hepatico {
                for j in -6..=6 {
                    let z = (j as f64 + 0.5) * 25.0 * 0.95 * um;
                    barra(p(-170.0 * um, 0.0, z), p(170.0 * um, 0.0, z), r(4.0 * um), Color::new(0.8, 0.1, 0.12, 0.8));
                }
            }
        }
        Tejido::Alveolar => {
            for i in 0..7 {
                let a = i as f64 * 0.9;
                let centro = if i == 0 { [0.0, 0.0, 0.0] } else { [a.cos() * 200.0 * um, a.sin() * 150.0 * um, (i as f64 - 3.0) * 40.0 * um] };
                draw_sphere_wires(p(centro[0], centro[1], centro[2]), r(100.0 * um), None, Color::new(1.0, 0.7, 0.75, 0.6));
                for k in 0..5 {
                    let b = k as f64 * 1.3 + a;
                    barra(p(centro[0] + b.cos() * 100.0 * um, centro[1] - 100.0 * um, centro[2] + b.sin() * 100.0 * um),
                          p(centro[0] + b.cos() * 100.0 * um, centro[1] + 100.0 * um, centro[2] + b.sin() * 100.0 * um),
                          r(4.0 * um), Color::new(0.85, 0.1, 0.15, 0.9));
                }
            }
        }
        Tejido::Sangre => {
            // Glóbulos rojos (discos de 7.5 µm), blancos y plaquetas fluyendo por el vaso.
            for i in 0..90 {
                let x = (rng.unif() - 0.5) * 120.0 * um;
                let z = (rng.unif() - 0.5) * 120.0 * um;
                let y0 = (rng.unif() - 0.5) * 300.0 * um;
                let y = ((y0 / um + t as f64 * 40.0 + 150.0).rem_euclid(300.0) - 150.0) * um;
                let pos = p(x, y, z);
                if i == 0 {
                    draw_sphere(p(0.0, 0.0, 0.0), r(6.0 * um), None, Color::new(0.92, 0.92, 1.0, 1.0));
                } else if i % 30 == 0 {
                    draw_sphere(pos, r(6.0 * um), None, Color::new(0.92, 0.92, 1.0, 1.0));
                } else if i % 11 == 0 {
                    draw_sphere(pos, r(1.2 * um), None, Color::new(0.95, 0.85, 0.5, 1.0));
                } else {
                    draw_cylinder(pos, r(3.75 * um), r(3.75 * um), r(2.0 * um), None, Color::new(0.8, 0.08, 0.1, 1.0));
                }
            }
        }
        Tejido::Parenquima | Tejido::Xilema => {
            for i in -4..=4 {
                for j in -4..=4 {
                    let x = i as f64 * 42.0 * um;
                    let z = j as f64 * 42.0 * um;
                    if tejido == Tejido::Xilema && i % 3 == 0 {
                        draw_cylinder(p(x, -200.0 * um, z), r(18.0 * um), r(18.0 * um), r(400.0 * um), None, Color::new(0.55, 0.4, 0.2, 0.45));
                        let fy = ((t * 0.3 + j as f32 * 0.2).fract() as f64 - 0.5) * 400.0 * um;
                        draw_sphere(p(x, fy, z), r(6.0 * um), None, Color::new(0.4, 0.7, 1.0, 1.0));
                        continue;
                    }
                    draw_cube_wires(p(x, 0.0, z), Vec3::splat(r(40.0 * um)), Color::new(0.3, 0.55, 0.25, 1.0));
                    for k in 0..6 {
                        let a = k as f64 * 1.05;
                        draw_sphere(p(x + a.cos() * 14.0 * um, (k as f64 - 2.5) * 5.0 * um, z + a.sin() * 14.0 * um), r(3.0 * um), None, Color::new(0.15, 0.7, 0.2, 1.0));
                    }
                }
            }
            if tejido == Tejido::Parenquima {
                // Estoma: dos células oclusivas que se abren y cierran.
                let abre = (0.5 + 0.5 * (t * 0.5).sin()) as f64 * 6.0;
                for s in [-1.0, 1.0] {
                    draw_sphere(p(s * (8.0 + abre) * um, 30.0 * um, 0.0), r(9.0 * um), None, Color::new(0.4, 0.75, 0.35, 1.0));
                }
            }
        }
        Tejido::Hifas => {
            for i in 0..20 {
                let a = i as f64 * 0.6;
                let mut prev = [0.0, 0.0, 0.0];
                for s in 1..8 {
                    let sig = [prev[0] + (a + s as f64 * 0.3).cos() * 30.0 * um, prev[1] + (a * 1.7 + s as f64).sin() * 10.0 * um, prev[2] + (a + s as f64 * 0.3).sin() * 30.0 * um];
                    barra(p(prev[0], prev[1], prev[2]), p(sig[0], sig[1], sig[2]), r(2.5 * um), Color::new(0.9, 0.85, 0.75, 1.0));
                    prev = sig;
                }
            }
        }
    }
}

fn dibujar_celula(vegetal: bool, c: [f64; 3], foco: [f64; 3], v: &dyn Fn([f64; 3]) -> Vec3, r: &dyn Fn(f64) -> f32, t: f32, rng: &mut Rng) {
    let um = 1e-6;
    let p = |x: f64, y: f64, z: f64| v([c[0] + x * um, c[1] + y * um, c[2] + z * um]);
    let radio = if vegetal { 18.0 } else { 11.0 };
    if vegetal {
        draw_cube_wires(p(0.0, 0.0, 0.0), Vec3::new(r(40.0 * um), r(28.0 * um), r(28.0 * um)), Color::new(0.4, 0.75, 0.3, 1.0));
        draw_sphere(p(4.0, 0.0, -2.0), r(9.0 * um), None, Color::new(0.4, 0.6, 1.0, 0.2)); // vacuola
        for i in 0..18 {
            let a = i as f64 * 0.35;
            let pos = if i == 0 { [foco[0] / um, foco[1] / um, foco[2] / um] } else { [a.cos() * 16.0, (i as f64 - 9.0) * 1.3, a.sin() * 11.0] };
            draw_sphere(p(pos[0], pos[1], pos[2]), r(2.5 * um), None, Color::new(0.15, 0.6, 0.2, 1.0));
            draw_sphere(p(pos[0] + 1.5, pos[1], pos[2]), r(2.2 * um), None, Color::new(0.15, 0.6, 0.2, 1.0));
        }
    } else {
        draw_sphere_wires(p(0.0, 0.0, 0.0), r(radio * um), None, Color::new(0.9, 0.7, 0.75, 0.35));
        // Bomba sodio-potasio: 3 Na⁺ salen, 2 K⁺ entran por cada ATP.
        for i in 0..40 {
            let th = (i as f64 * 2.399).rem_euclid(std::f64::consts::PI);
            let ph = i as f64 * 1.3;
            let d = [th.sin() * ph.cos(), th.cos(), th.sin() * ph.sin()];
            draw_sphere(p(d[0] * radio, d[1] * radio, d[2] * radio), r(0.6 * um), None, Color::new(0.65, 0.3, 0.9, 1.0));
            let f = ((t * 0.7 + i as f32 * 0.13).fract()) as f64;
            let na = radio - 2.0 + 4.0 * f;
            let k = radio + 2.0 - 4.0 * f;
            draw_sphere(p(d[0] * na, d[1] * na, d[2] * na), r(0.35 * um), None, Color::new(1.0, 0.9, 0.2, 1.0));
            draw_sphere(p(d[0] * k, d[1] * k, d[2] * k), r(0.35 * um), None, Color::new(0.75, 0.45, 1.0, 1.0));
        }
        // Golgi: sacos aplanados apilados.
        for s in 0..5 {
            draw_cylinder(p(3.0, 4.0 + s as f64 * 0.5, -3.0), r(2.0 * um), r(2.0 * um), r(0.25 * um), None, Color::new(0.95, 0.75, 0.3, 1.0));
        }
        for i in 0..6 {
            let a = i as f64 * 1.1;
            draw_sphere(p(a.cos() * 7.0, -4.0 + a.sin() * 2.0, a.sin() * 6.0), r(0.6 * um), None, Color::new(0.8, 0.4, 0.3, 1.0));
        }
        // Mitocondrias (la primera es la que sigue la ruta de energía).
        for i in 0..12 {
            let a = i as f64 * 0.52;
            let pos = if i == 0 { [foco[0] / um, foco[1] / um, foco[2] / um] } else { [a.cos() * 7.5, (i as f64 - 6.0) * 1.2, a.sin() * 7.0] };
            barra(p(pos[0] - 0.9, pos[1], pos[2]), p(pos[0] + 0.9, pos[1], pos[2]), r(0.5 * um), Color::new(0.9, 0.45, 0.35, 1.0));
        }
    }
    // Núcleo con nucléolo (en la ruta genética es el foco).
    let nuc = if vegetal { [-10.0, 2.0, 4.0] } else { [-3.0, 0.5, 0.0] };
    draw_sphere(p(nuc[0], nuc[1], nuc[2]), r(3.0 * um), None, Color::new(0.45, 0.35, 0.7, 0.85));
    draw_sphere(p(nuc[0] + 0.8, nuc[1] + 0.5, nuc[2]), r(0.9 * um), None, Color::new(0.3, 0.2, 0.5, 1.0));
    // Retículo endoplasmático rugoso con ribosomas alrededor del núcleo.
    for i in 0..160 {
        let a = rng.unif() * std::f64::consts::TAU;
        let b = rng.unif() * std::f64::consts::PI;
        let rr = 4.0 + rng.unif() * 2.5;
        draw_sphere(p(nuc[0] + rr * b.sin() * a.cos(), nuc[1] + rr * b.cos(), nuc[2] + rr * b.sin() * a.sin()), r(0.12 * um), None, if i % 4 == 0 { Color::new(0.4, 0.7, 1.0, 1.0) } else { Color::new(0.9, 0.9, 0.95, 0.9) });
    }
}

fn dibujar_mitocondria(vegetal: bool, c: [f64; 3], foco: [f64; 3], v: &dyn Fn([f64; 3]) -> Vec3, r: &dyn Fn(f64) -> f32) {
    let um = 1e-6;
    let p = |x: f64, y: f64, z: f64| v([c[0] + x, c[1] + y, c[2] + z]);
    if vegetal {
        // Cloroplasto: granas (pilas de tilacoides).
        draw_sphere_wires(p(0.0, 0.0, 0.0), r(2.6 * um), None, Color::new(0.3, 0.8, 0.3, 0.5));
        for g in 0..7 {
            let x = (g as f64 - 3.0) * 0.6 * um;
            for s in 0..6 {
                draw_cylinder(p(x, (s as f64 - 3.0) * 0.08 * um, 0.0), r(0.25 * um), r(0.25 * um), r(0.04 * um), None, Color::new(0.1, 0.55, 0.15, 1.0));
            }
        }
    } else {
        // Membrana externa, crestas de la interna, y ATP sintasas en ellas.
        barra(p(-1.0 * um, 0.0, 0.0), p(1.0 * um, 0.0, 0.0), r(0.5 * um), Color::new(0.9, 0.5, 0.4, 0.25));
        for i in 0..9 {
            let x = (i as f64 - 4.0) * 0.2 * um;
            draw_cube(p(x, 0.0, 0.0), Vec3::new(r(0.02 * um), r(0.7 * um), r(0.6 * um)), None, Color::new(0.95, 0.65, 0.45, 0.9));
            for s in 0..4 {
                draw_sphere(p(x + 0.03 * um, (s as f64 - 1.5) * 0.15 * um, 0.32 * um), r(0.02 * um), None, Color::new(1.0, 0.9, 0.3, 1.0));
            }
        }
    }
    draw_sphere(v([c[0] + foco[0], c[1] + foco[1], c[2] + foco[2]]), r(0.03 * um), None, Color::new(1.0, 0.95, 0.4, 1.0));
}

fn dibujar_nucleo(c: [f64; 3], foco: [f64; 3], v: &dyn Fn([f64; 3]) -> Vec3, r: &dyn Fn(f64) -> f32, t: f32) {
    let um = 1e-6;
    let p = |x: f64, y: f64, z: f64| v([c[0] + x * um, c[1] + y * um, c[2] + z * um]);
    draw_sphere_wires(p(0.0, 0.0, 0.0), r(3.0 * um), None, Color::new(0.6, 0.5, 0.9, 0.5));
    // Poros nucleares.
    for i in 0..30 {
        let th = (i as f64 * 2.399).rem_euclid(std::f64::consts::PI);
        let ph = i as f64 * 1.7;
        draw_sphere(p(3.0 * th.sin() * ph.cos(), 3.0 * th.cos(), 3.0 * th.sin() * ph.sin()), r(0.08 * um), None, Color::new(0.9, 0.85, 1.0, 1.0));
    }
    // Cromatina: fibras enrolladas.
    for f in 0..10 {
        let mut prev = [0.0, 0.0, 0.0];
        for s in 1..20 {
            let a = s as f64 * 0.7 + f as f64;
            let sig = [prev[0] + a.cos() * 0.25, prev[1] + (a * 1.3).sin() * 0.25, prev[2] + (a * 0.7).sin() * 0.25];
            barra(p(prev[0], prev[1], prev[2]), p(sig[0], sig[1], sig[2]), r(0.03 * um), Color::new(0.75, 0.4, 0.85, 1.0));
            prev = sig;
        }
    }
    let _ = t;
    draw_sphere(v([c[0] + foco[0], c[1] + foco[1], c[2] + foco[2]]), r(0.03 * um), None, Color::new(1.0, 0.95, 0.4, 1.0));
}

fn dibujar_atp_sintasa(c: [f64; 3], v: &dyn Fn([f64; 3]) -> Vec3, r: &dyn Fn(f64) -> f32, t: f32) {
    let nm = 1e-9;
    let p = |x: f64, y: f64, z: f64| v([c[0] + x * nm, c[1] + y * nm, c[2] + z * nm]);
    // Membrana interna.
    draw_cube(p(0.0, -4.0, 0.0), Vec3::new(r(40.0 * nm), r(4.0 * nm), r(40.0 * nm)), None, Color::new(0.9, 0.7, 0.5, 0.35));
    // Rotor F₀ (anillo c) girando con el flujo de protones.
    let giro = t as f64 * 2.0;
    for i in 0..10 {
        let a = giro + i as f64 * std::f64::consts::TAU / 10.0;
        draw_sphere(p(a.cos() * 3.0, -4.0, a.sin() * 3.0), r(0.9 * nm), None, Color::new(0.4, 0.6, 0.95, 1.0));
    }
    barra(p(0.0, -3.0, 0.0), p(0.0, 4.0, 0.0), r(0.7 * nm), Color::new(0.85, 0.85, 0.3, 1.0));
    // Cabeza F₁: 3 α + 3 β alternadas (aquí se fabrica el ATP).
    for i in 0..6 {
        let a = i as f64 * std::f64::consts::TAU / 6.0;
        let col = if i % 2 == 0 { Color::new(0.9, 0.35, 0.35, 1.0) } else { Color::new(0.95, 0.6, 0.6, 1.0) };
        draw_sphere(p(a.cos() * 3.2, 6.5, a.sin() * 3.2), r(2.2 * nm), None, col);
    }
    // Protones (H⁺) que bajan por el canal.
    for i in 0..8 {
        let f = ((t * 0.9 + i as f32 * 0.125).fract()) as f64;
        draw_sphere(p(4.5, 4.0 - 10.0 * f, 0.0), r(0.3 * nm), None, Color::new(1.0, 1.0, 1.0, 1.0));
    }
}

fn dibujar_adn(c: [f64; 3], v: &dyn Fn([f64; 3]) -> Vec3, r: &dyn Fn(f64) -> f32, t: f32) {
    let nm = 1e-9;
    let giro = t as f64 * 0.3;
    let p = |x: f64, y: f64, z: f64| v([c[0] + x * nm, c[1] + y * nm, c[2] + z * nm]);
    let bases = GEN_INSULINA.as_bytes();
    for (i, &b) in bases.iter().enumerate().take(60) {
        // 10.5 pares por vuelta, 0.34 nm entre pares, radio 1 nm.
        let y = (i as f64 - 30.0) * 0.34;
        let a = i as f64 * std::f64::consts::TAU / 10.5 + giro;
        let (x1, z1) = (a.cos(), a.sin());
        let (x2, z2) = ((a + 2.4).cos(), (a + 2.4).sin());
        draw_sphere(p(x1, y, z1), r(0.22 * nm), None, Color::new(0.9, 0.85, 0.5, 1.0));
        draw_sphere(p(x2, y, z2), r(0.22 * nm), None, Color::new(0.9, 0.85, 0.5, 1.0));
        let (ca, cb) = match b {
            b'A' => (Color::new(0.95, 0.3, 0.3, 1.0), Color::new(0.3, 0.5, 1.0, 1.0)),
            b'T' => (Color::new(0.3, 0.5, 1.0, 1.0), Color::new(0.95, 0.3, 0.3, 1.0)),
            b'G' => (Color::new(0.3, 0.85, 0.35, 1.0), Color::new(1.0, 0.85, 0.2, 1.0)),
            _ => (Color::new(1.0, 0.85, 0.2, 1.0), Color::new(0.3, 0.85, 0.35, 1.0)),
        };
        let medio = [(x1 + x2) / 2.0, y, (z1 + z2) / 2.0];
        barra(p(x1, y, z1), p(medio[0], y, medio[2]), r(0.1 * nm), ca);
        barra(p(medio[0], y, medio[2]), p(x2, y, z2), r(0.1 * nm), cb);
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

/// Ventana propia (`genesis cuerpo`).
pub fn ver(interior: Interior) {
    let conf = Conf {
        window_title: "Genesis - dentro de un ser vivo".into(),
        window_width: 1440,
        window_height: 900,
        high_dpi: true,
        sample_count: 4,
        window_resizable: true,
        ..Default::default()
    };
    macroquad::Window::from_config(conf, async move {
        let fuente = std::fs::read("/System/Library/Fonts/Supplemental/Arial Unicode.ttf").ok().and_then(|b| load_ttf_font_from_bytes(&b).ok());
        let mut i = interior;
        while i.frame(fuente.as_ref()) {
            next_frame().await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_insulina_sale_del_gen() {
        let p = crate::vida::traducir(GEN_INSULINA);
        assert!(p.starts_with("MALWMRLLPLLALLALWGPDPAAA"), "{p}");
        assert!(p.contains("FVNQHLCGSHLVEALYLVCGERGFFYTPKT"), "cadena B de la insulina: {p}");
    }

    #[test]
    fn humano_mide_lo_que_debe() {
        let h = Interior::humano();
        assert!((h.largo - 1.73).abs() < 0.05);
        assert!(h.plan == Plan::Humano);
    }

    #[test]
    fn los_niveles_van_de_grande_a_pequeno() {
        let h = Interior::humano();
        for w in NIVELES.windows(2) {
            assert!(h.tamano(w[0]) > h.tamano(w[1]));
        }
    }

    #[test]
    fn el_atp_tiene_fosforo_y_la_adenina_nitrogeno() {
        let mut h = Interior::humano();
        assert_eq!(h.atomo_foco().0, 15);
        h.ruta_energia = false;
        assert_eq!(h.atomo_foco().0, 7);
    }
}
