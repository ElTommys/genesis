//! Escala 2: moléculas reales. Se describen en SMILES (la notación estándar de la
//! química), se completan con hidrógenos por valencia y su forma 3D sale de la física:
//! longitudes por radios covalentes, ángulos por VSEPR y repulsión entre átomos.
//! También: tipos de enlace, energías de enlace, masa molar, cargas δ±, dipolo y
//! entalpía de reacción a partir de los enlaces rotos y formados.

use crate::azar::Rng;
use crate::cuantica::Atomo;
use crate::elementos::{z_de_simbolo, ELECTRONEGATIVIDAD, SIMBOLOS};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone)]
pub struct AtomoMol {
    pub z: u32,
    pub carga: i32,
    pub aromatico: bool,
    pub pos: [f64; 3],
    /// Carga parcial estimada δ (fracción de e).
    pub q: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct Enlace {
    pub a: usize,
    pub b: usize,
    /// 1 simple, 2 doble, 3 triple, 1.5 aromático.
    pub orden: f64,
}

#[derive(Debug, Clone)]
pub struct Molecula {
    pub nombre: String,
    pub smiles: String,
    pub atomos: Vec<AtomoMol>,
    pub enlaces: Vec<Enlace>,
}

// ───────────── Datos ─────────────

/// Radio covalente (pm), Cordero et al. 2008.
pub fn radio_covalente_pm(z: u32) -> f64 {
    match z {
        1 => 31.0, 2 => 28.0, 3 => 128.0, 4 => 96.0, 5 => 84.0, 6 => 76.0, 7 => 71.0, 8 => 66.0, 9 => 57.0,
        11 => 166.0, 12 => 141.0, 13 => 121.0, 14 => 111.0, 15 => 107.0, 16 => 105.0, 17 => 102.0,
        19 => 203.0, 20 => 176.0, 26 => 132.0, 29 => 132.0, 30 => 122.0, 35 => 120.0, 53 => 139.0,
        _ => 140.0,
    }
}

/// Radio de van der Waals (pm), Bondi 1964.
pub fn radio_vdw_pm(z: u32) -> f64 {
    match z {
        1 => 120.0, 6 => 170.0, 7 => 155.0, 8 => 152.0, 9 => 147.0, 11 => 227.0, 12 => 173.0,
        15 => 180.0, 16 => 180.0, 17 => 175.0, 19 => 275.0, 35 => 185.0, 53 => 198.0,
        _ => 200.0,
    }
}

/// Peso atómico estándar (IUPAC, g/mol) para los elementos más comunes.
pub fn peso_atomico(z: u32) -> f64 {
    match z {
        1 => 1.008, 6 => 12.011, 7 => 14.007, 8 => 15.999, 9 => 18.998, 11 => 22.990, 12 => 24.305,
        15 => 30.974, 16 => 32.06, 17 => 35.45, 19 => 39.098, 20 => 40.078, 26 => 55.845, 35 => 79.904,
        53 => 126.904,
        _ => Atomo::nuevo(z).masa_u(),
    }
}

/// Energía media de enlace (kJ/mol), tablas estándar de química general.
pub fn energia_enlace_kj(z1: u32, z2: u32, orden: f64) -> f64 {
    let (a, b) = if z1 <= z2 { (z1, z2) } else { (z2, z1) };
    let o = (orden * 2.0).round() as u32; // 2 simple, 3 aromático, 4 doble, 6 triple
    match (a, b, o) {
        (1, 1, 2) => 436.0,
        (1, 6, 2) => 413.0,
        (1, 7, 2) => 391.0,
        (1, 8, 2) => 463.0,
        (1, 9, 2) => 567.0,
        (1, 16, 2) => 339.0,
        (1, 17, 2) => 431.0,
        (1, 35, 2) => 366.0,
        (6, 6, 2) => 348.0,
        (6, 6, 3) => 518.0,
        (6, 6, 4) => 614.0,
        (6, 6, 6) => 839.0,
        (6, 7, 2) => 293.0,
        (6, 7, 3) => 460.0,
        (6, 7, 4) => 615.0,
        (6, 7, 6) => 891.0,
        (6, 8, 2) => 358.0,
        (6, 8, 3) => 500.0,
        (6, 8, 4) => 799.0,
        (6, 9, 2) => 485.0,
        (6, 16, 2) => 259.0,
        (6, 17, 2) => 328.0,
        (7, 7, 2) => 163.0,
        (7, 7, 4) => 418.0,
        (7, 7, 6) => 945.0,
        (7, 8, 2) => 201.0,
        (7, 8, 4) => 607.0,
        (8, 8, 2) => 146.0,
        (8, 8, 4) => 498.0,
        (8, 15, 2) => 335.0,
        (8, 15, 4) => 544.0,
        (9, 9, 2) => 155.0,
        (17, 17, 2) => 242.0,
        _ => 300.0 * orden,
    }
}

pub enum TipoEnlace {
    CovalenteApolar,
    CovalentePolar,
    Ionico,
}

/// Clasificación por diferencia de electronegatividad de Pauling.
pub fn tipo_enlace(z1: u32, z2: u32) -> (TipoEnlace, f64) {
    let d = (ELECTRONEGATIVIDAD[(z1 - 1) as usize] - ELECTRONEGATIVIDAD[(z2 - 1) as usize]).abs();
    let t = if d < 0.4 { TipoEnlace::CovalenteApolar } else if d < 1.7 { TipoEnlace::CovalentePolar } else { TipoEnlace::Ionico };
    (t, d)
}

impl TipoEnlace {
    pub fn nombre(&self) -> &'static str {
        match self {
            TipoEnlace::CovalenteApolar => "covalente apolar",
            TipoEnlace::CovalentePolar => "covalente polar",
            TipoEnlace::Ionico => "iónico",
        }
    }
}

/// Catálogo: nombre → SMILES (estructuras de PubChem).
pub const CATALOGO: &[(&str, &str, &str)] = &[
    ("hidrógeno", "[H][H]", "H2"),
    ("oxígeno", "O=O", "O2"),
    ("nitrógeno", "N#N", "N2"),
    ("agua", "O", "H2O"),
    ("dióxido de carbono", "O=C=O", "CO2"),
    ("metano", "C", "CH4"),
    ("amoníaco", "N", "NH3"),
    ("ácido clorhídrico", "Cl", "HCl"),
    ("cloruro de sodio", "[Na+].[Cl-]", "NaCl"),
    ("etileno", "C=C", "C2H4"),
    ("acetileno", "C#C", "C2H2"),
    ("etanol", "CCO", "C2H6O"),
    ("ácido acético", "CC(=O)O", "C2H4O2"),
    ("urea", "NC(=O)N", "CH4N2O"),
    ("benceno", "c1ccccc1", "C6H6"),
    ("glucosa", "OCC1OC(O)C(O)C(O)C1O", "C6H12O6"),
    ("glicina", "NCC(=O)O", "C2H5NO2"),
    ("alanina", "CC(N)C(=O)O", "C3H7NO2"),
    ("adenina", "Nc1ncnc2[nH]cnc12", "C5H5N5"),
    ("timina", "Cc1c[nH]c(=O)[nH]c1=O", "C5H6N2O2"),
    ("guanina", "Nc1nc2[nH]cnc2c(=O)[nH]1", "C5H5N5O"),
    ("citosina", "Nc1cc[nH]c(=O)n1", "C4H5N3O"),
    ("cafeína", "Cn1cnc2c1c(=O)n(C)c(=O)n2C", "C8H10N4O2"),
    ("aspirina", "CC(=O)Oc1ccccc1C(=O)O", "C9H8O4"),
    ("ATP", "Nc1ncnc2c1ncn2C3OC(COP(=O)(O)OP(=O)(O)OP(=O)(O)O)C(O)C3O", "C10H16N5O13P3"),
];

pub fn buscar(nombre: &str) -> Option<(&'static str, &'static str)> {
    let n = nombre.to_lowercase();
    CATALOGO
        .iter()
        .find(|(a, _, f)| a.to_lowercase() == n || f.to_lowercase() == n)
        .map(|(a, s, _)| (*a, *s))
}

// ───────────── SMILES ─────────────

struct Crudo {
    z: u32,
    aromatico: bool,
    carga: i32,
    h: Option<u32>,
}

/// Lee SMILES: átomos (también aromáticos y entre corchetes), enlaces - = # :,
/// ramas ( ), cierres de anillo 1-9 y %nn, y componentes separados por '.'.
pub fn leer_smiles(nombre: &str, s: &str) -> Result<Molecula, String> {
    let c: Vec<char> = s.chars().collect();
    let mut crudos: Vec<Crudo> = Vec::new();
    let mut enlaces: Vec<(usize, usize, Option<f64>)> = Vec::new();
    let mut prev: Option<usize> = None;
    let mut pila: Vec<Option<usize>> = Vec::new();
    let mut orden: Option<f64> = None;
    let mut anillos: HashMap<u32, (usize, Option<f64>)> = HashMap::new();
    let mut i = 0;
    let mut unir = |nuevo: usize, prev: &mut Option<usize>, orden: &mut Option<f64>, enlaces: &mut Vec<(usize, usize, Option<f64>)>| {
        if let Some(p) = *prev {
            enlaces.push((p, nuevo, orden.take()));
        }
        *orden = None;
        *prev = Some(nuevo);
    };
    while i < c.len() {
        let ch = c[i];
        match ch {
            '(' => pila.push(prev),
            ')' => prev = pila.pop().ok_or("paréntesis ')' sin abrir")?,
            '-' => orden = Some(1.0),
            '=' => orden = Some(2.0),
            '#' => orden = Some(3.0),
            ':' => orden = Some(1.5),
            '.' => prev = None,
            '0'..='9' | '%' => {
                let num = if ch == '%' {
                    let n: String = c[i + 1..(i + 3).min(c.len())].iter().collect();
                    i += 2;
                    n.parse().map_err(|_| "anillo %nn inválido")?
                } else {
                    ch.to_digit(10).unwrap()
                };
                let p = prev.ok_or("cierre de anillo sin átomo")?;
                if let Some((q, o)) = anillos.remove(&num) {
                    enlaces.push((q, p, orden.take().or(o)));
                } else {
                    anillos.insert(num, (p, orden.take()));
                }
            }
            '[' => {
                let fin = c[i..].iter().position(|&x| x == ']').ok_or("falta ']'")? + i;
                let dentro: String = c[i + 1..fin].iter().collect();
                let at = leer_corchete(&dentro)?;
                crudos.push(at);
                unir(crudos.len() - 1, &mut prev, &mut orden, &mut enlaces);
                i = fin;
            }
            _ if ch.is_ascii_alphabetic() => {
                let dos: String = c[i..(i + 2).min(c.len())].iter().collect();
                let (sim, largo) = if dos == "Cl" || dos == "Br" { (dos.clone(), 2) } else { (ch.to_string(), 1) };
                let aromatico = sim.chars().next().unwrap().is_ascii_lowercase();
                let z = z_de_simbolo(&sim).ok_or(format!("elemento desconocido '{sim}'"))?;
                if !matches!(z, 5 | 6 | 7 | 8 | 9 | 15 | 16 | 17 | 35 | 53) {
                    return Err(format!("'{sim}' debe ir entre corchetes, p. ej. [{sim}]"));
                }
                crudos.push(Crudo { z, aromatico, carga: 0, h: None });
                unir(crudos.len() - 1, &mut prev, &mut orden, &mut enlaces);
                i += largo - 1;
            }
            ' ' => {}
            otro => return Err(format!("carácter no soportado '{otro}'")),
        }
        i += 1;
    }
    if !anillos.is_empty() {
        return Err("hay anillos sin cerrar".into());
    }
    // Orden por defecto: aromático entre dos aromáticos, simple en otro caso.
    let mut enlaces: Vec<Enlace> = enlaces
        .into_iter()
        .map(|(a, b, o)| Enlace { a, b, orden: o.unwrap_or(if crudos[a].aromatico && crudos[b].aromatico { 1.5 } else { 1.0 }) })
        .collect();
    let mut atomos: Vec<AtomoMol> = crudos
        .iter()
        .map(|r| AtomoMol { z: r.z, carga: r.carga, aromatico: r.aromatico, pos: [0.0; 3], q: 0.0 })
        .collect();
    // Hidrógenos implícitos según la valencia normal del elemento.
    for (idx, r) in crudos.iter().enumerate() {
        let suma: f64 = enlaces.iter().filter(|e| e.a == idx || e.b == idx).map(|e| e.orden).sum();
        let nh = match r.h {
            Some(h) => h,
            None => {
                let valencias: &[u32] = match r.z {
                    5 => &[3],
                    6 => &[4],
                    7 => &[3, 5],
                    8 => &[2],
                    15 => &[3, 5],
                    16 => &[2, 4, 6],
                    _ => &[1],
                };
                let s = (suma + 1e-6).floor() as u32;
                if r.aromatico {
                    // Un átomo aromático no "expande" su valencia (n aromático nunca es N⁵).
                    valencias[0].saturating_sub(s)
                } else {
                valencias.iter().find(|&&v| v >= s).map_or(0, |&v| v - s)
                }
            }
        };
        for _ in 0..nh {
            atomos.push(AtomoMol { z: 1, carga: 0, aromatico: false, pos: [0.0; 3], q: 0.0 });
            enlaces.push(Enlace { a: idx, b: atomos.len() - 1, orden: 1.0 });
        }
    }
    let mut m = Molecula { nombre: nombre.to_string(), smiles: s.to_string(), atomos, enlaces };
    m.calcular_cargas();
    m.geometria(7);
    Ok(m)
}

fn leer_corchete(s: &str) -> Result<Crudo, String> {
    let c: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < c.len() && c[i].is_ascii_digit() {
        i += 1; // isótopo: se ignora
    }
    let ini = i;
    i += 1;
    if i < c.len() && c[i].is_ascii_lowercase() && c[ini].is_ascii_uppercase() {
        i += 1;
    }
    let sim: String = c[ini..i].iter().collect();
    let aromatico = sim.chars().next().is_some_and(|x| x.is_ascii_lowercase());
    let z = z_de_simbolo(&sim).ok_or(format!("elemento desconocido '{sim}'"))?;
    let mut h = 0;
    let mut carga = 0;
    while i < c.len() {
        match c[i] {
            'H' => {
                h += 1;
                if i + 1 < c.len() && c[i + 1].is_ascii_digit() {
                    h += c[i + 1].to_digit(10).unwrap() - 1;
                    i += 1;
                }
            }
            '+' | '-' => {
                let signo = if c[i] == '+' { 1 } else { -1 };
                if i + 1 < c.len() && c[i + 1].is_ascii_digit() {
                    carga += signo * c[i + 1].to_digit(10).unwrap() as i32;
                    i += 1;
                } else {
                    carga += signo;
                }
            }
            _ => {}
        }
        i += 1;
    }
    Ok(Crudo { z, aromatico, carga, h: Some(h) })
}

// ───────────── Propiedades y geometría ─────────────

impl Molecula {
    pub fn vecinos(&self, i: usize) -> Vec<(usize, f64)> {
        self.enlaces
            .iter()
            .filter_map(|e| if e.a == i { Some((e.b, e.orden)) } else if e.b == i { Some((e.a, e.orden)) } else { None })
            .collect()
    }

    /// Pares solitarios del átomo i: (electrones de valencia − carga − electrones en enlaces) / 2.
    pub fn pares_libres(&self, i: usize) -> u32 {
        let a = &self.atomos[i];
        if a.z == 1 {
            return 0;
        }
        let at = Atomo::nuevo(a.z);
        if !matches!(at.bloque(), 's' | 'p') {
            return 0;
        }
        let ve = at.electrones_valencia() as i32 - a.carga;
        let usados: f64 = self.vecinos(i).iter().map(|v| v.1).sum();
        ((ve - (usados + 1e-6).floor() as i32).max(0) / 2) as u32
    }

    /// Geometría electrónica/molecular por VSEPR alrededor del átomo i.
    pub fn geometria_de(&self, i: usize) -> &'static str {
        let b = self.vecinos(i).len();
        let lp = self.pares_libres(i) as usize;
        match (b, lp) {
            (0, _) | (1, _) => "terminal",
            (2, 0) => "lineal",
            (2, 1) | (2, 2) => "angular",
            (3, 0) => "trigonal plana",
            (3, 1) => "piramidal trigonal",
            (4, 0) => "tetraédrica",
            (4, 1) => "balancín",
            (5, 0) => "bipirámide trigonal",
            (6, 0) => "octaédrica",
            _ => "compleja",
        }
    }

    fn angulo_objetivo(&self, i: usize) -> Option<f64> {
        let b = self.vecinos(i).len() as u32;
        let lp = self.pares_libres(i);
        let base = match b + lp {
            2 => 180.0,
            3 => 120.0,
            4 => 109.47,
            _ => return None,
        };
        let ajuste = if b + lp >= 3 { 2.5 * lp as f64 } else { 0.0 };
        Some((base - ajuste).to_radians())
    }

    /// Longitud de enlace (Å) a partir de radios covalentes, acortada por el orden.
    pub fn longitud(&self, e: &Enlace) -> f64 {
        let r = (radio_covalente_pm(self.atomos[e.a].z) + radio_covalente_pm(self.atomos[e.b].z)) / 100.0;
        let factor = match (e.orden * 2.0).round() as u32 {
            3 => 0.91,
            4 => 0.87,
            6 => 0.78,
            _ => 1.0,
        };
        r * factor
    }

    /// Cargas parciales: el átomo más electronegativo atrae la densidad electrónica.
    fn calcular_cargas(&mut self) {
        for i in 0..self.atomos.len() {
            let xi = ELECTRONEGATIVIDAD[(self.atomos[i].z - 1) as usize];
            let d: f64 = self.vecinos(i).iter().map(|&(j, o)| (ELECTRONEGATIVIDAD[(self.atomos[j].z - 1) as usize] - xi) * o.min(2.0)).sum();
            self.atomos[i].q = self.atomos[i].carga as f64 + 0.26 * d;
        }
    }

    /// Coloca los átomos y relaja: resortes de enlace, ángulos VSEPR y repulsión.
    pub fn geometria(&mut self, semilla: u64) {
        let n = self.atomos.len();
        let mut rng = Rng::nuevo(semilla);
        let mut puesto = vec![false; n];
        // Siembra por recorrido en anchura desde cada componente.
        let mut desplaz = 0.0;
        for s in 0..n {
            if puesto[s] {
                continue;
            }
            self.atomos[s].pos = [desplaz, 0.0, 0.0];
            desplaz += 3.0;
            puesto[s] = true;
            let mut cola = std::collections::VecDeque::from([s]);
            while let Some(i) = cola.pop_front() {
                for (j, _) in self.vecinos(i) {
                    if !puesto[j] {
                        let d = [rng.normal(), rng.normal(), rng.normal()];
                        let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-6);
                        let p = self.atomos[i].pos;
                        self.atomos[j].pos = [p[0] + 1.2 * d[0] / l, p[1] + 1.2 * d[1] / l, p[2] + 1.2 * d[2] / l];
                        puesto[j] = true;
                        cola.push_back(j);
                    }
                }
            }
        }
        let vecinos: Vec<Vec<(usize, f64)>> = (0..n).map(|i| self.vecinos(i)).collect();
        let angulos: Vec<Option<f64>> = (0..n).map(|i| self.angulo_objetivo(i)).collect();
        let longitudes: Vec<f64> = self.enlaces.iter().map(|e| self.longitud(e)).collect();
        let cerca = |i: usize, j: usize| vecinos[i].iter().any(|v| v.0 == j) || vecinos[i].iter().any(|v| vecinos[v.0].iter().any(|w| w.0 == j));
        let excluir: Vec<Vec<bool>> = (0..n).map(|i| (0..n).map(|j| i == j || cerca(i, j)).collect()).collect();
        for paso in 0..6000 {
            let mut f = vec![[0.0f64; 3]; n];
            for (k, e) in self.enlaces.iter().enumerate() {
                let d = resta(self.atomos[e.b].pos, self.atomos[e.a].pos);
                let r = norma(d).max(1e-6);
                let fr = 40.0 * (r - longitudes[k]) / r;
                for x in 0..3 {
                    f[e.a][x] += fr * d[x];
                    f[e.b][x] -= fr * d[x];
                }
            }
            for j in 0..n {
                let v = &vecinos[j];
                let Some(t0) = angulos[j] else {
                    // Número estérico ≥ 5: los vecinos simplemente se repelen entre sí.
                    for x in 0..v.len() {
                        for y in x + 1..v.len() {
                            repeler(&self.atomos, &mut f, v[x].0, v[y].0, 3.0, 4.0);
                        }
                    }
                    continue;
                };
                for x in 0..v.len() {
                    for y in x + 1..v.len() {
                        let (a, c) = (v[x].0, v[y].0);
                        let ra = resta(self.atomos[a].pos, self.atomos[j].pos);
                        let rc = resta(self.atomos[c].pos, self.atomos[j].pos);
                        let (la, lc) = (norma(ra).max(1e-6), norma(rc).max(1e-6));
                        let cos = (punto(ra, rc) / (la * lc)).clamp(-0.9999, 0.9999);
                        let sen = (1.0 - cos * cos).sqrt();
                        let pref = 8.0 * (cos.acos() - t0) / sen;
                        for k in 0..3 {
                            let fa = pref * (rc[k] / (la * lc) - cos * ra[k] / (la * la));
                            let fc = pref * (ra[k] / (la * lc) - cos * rc[k] / (lc * lc));
                            f[a][k] += fa;
                            f[c][k] += fc;
                            f[j][k] -= fa + fc;
                        }
                    }
                }
            }
            for i in 0..n {
                for j in i + 1..n {
                    if !excluir[i][j] {
                        let umbral = (radio_vdw_pm(self.atomos[i].z) + radio_vdw_pm(self.atomos[j].z)) / 100.0 * 0.75;
                        repeler(&self.atomos, &mut f, i, j, umbral, 6.0);
                    }
                }
            }
            // Iones: atracción de Coulomb entre cargas opuestas sin enlace.
            for i in 0..n {
                for j in i + 1..n {
                    let qq = (self.atomos[i].carga * self.atomos[j].carga) as f64;
                    if qq < 0.0 {
                        let d = resta(self.atomos[j].pos, self.atomos[i].pos);
                        let r = norma(d).max(0.5);
                        for x in 0..3 {
                            f[i][x] += 2.0 * d[x] / (r * r * r);
                            f[j][x] -= 2.0 * d[x] / (r * r * r);
                        }
                    }
                }
            }
            let paso_max = if paso < 3000 { 0.05 } else { 0.01 };
            for i in 0..n {
                let m = norma(f[i]);
                let s = (0.004 * m).min(paso_max) / m.max(1e-9);
                for x in 0..3 {
                    self.atomos[i].pos[x] += f[i][x] * s;
                }
            }
        }
        // Centra la molécula.
        let c = (0..3).map(|x| self.atomos.iter().map(|a| a.pos[x]).sum::<f64>() / n as f64).collect::<Vec<_>>();
        for a in &mut self.atomos {
            for x in 0..3 {
                a.pos[x] -= c[x];
            }
        }
    }

    pub fn angulo(&self, a: usize, j: usize, c: usize) -> f64 {
        let ra = resta(self.atomos[a].pos, self.atomos[j].pos);
        let rc = resta(self.atomos[c].pos, self.atomos[j].pos);
        (punto(ra, rc) / (norma(ra) * norma(rc))).clamp(-1.0, 1.0).acos().to_degrees()
    }

    pub fn distancia(&self, a: usize, b: usize) -> f64 {
        norma(resta(self.atomos[a].pos, self.atomos[b].pos))
    }

    pub fn conteo(&self) -> BTreeMap<u32, usize> {
        let mut c = BTreeMap::new();
        for a in &self.atomos {
            *c.entry(a.z).or_default() += 1;
        }
        c
    }

    pub fn formula(&self) -> String {
        let carga: i32 = self.atomos.iter().map(|a| a.carga).sum();
        crate::mundo::formula(&self.conteo(), carga)
    }

    pub fn masa_molar(&self) -> f64 {
        self.atomos.iter().map(|a| peso_atomico(a.z)).sum()
    }

    /// Suma de energías de enlace (kJ/mol): lo que costaría romperla en átomos sueltos.
    pub fn energia_enlaces_kj(&self) -> f64 {
        self.enlaces.iter().map(|e| energia_enlace_kj(self.atomos[e.a].z, self.atomos[e.b].z, e.orden)).sum()
    }

    /// Momento dipolar estimado (debye): μ = Σ qᵢ·rᵢ, con 1 e·Å = 4.803 D.
    pub fn dipolo_debye(&self) -> f64 {
        let mut mu = [0.0; 3];
        for a in &self.atomos {
            if a.carga == 0 {
                for x in 0..3 {
                    mu[x] += a.q * a.pos[x];
                }
            }
        }
        norma(mu) * 4.803
    }
}

fn resta(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn punto(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn norma(a: [f64; 3]) -> f64 {
    punto(a, a).sqrt()
}
fn repeler(at: &[AtomoMol], f: &mut [[f64; 3]], i: usize, j: usize, umbral: f64, k: f64) {
    let d = resta(at[j].pos, at[i].pos);
    let r = norma(d).max(1e-6);
    if r < umbral {
        let fr = k * (umbral - r) / r;
        for x in 0..3 {
            f[i][x] -= fr * d[x];
            f[j][x] += fr * d[x];
        }
    }
}

// ───────────── Reacciones ─────────────

pub struct Reaccion {
    pub reactivos: Vec<(u32, Molecula)>,
    pub productos: Vec<(u32, Molecula)>,
}

fn lado(s: &str) -> Result<Vec<(u32, Molecula)>, String> {
    s.split('+')
        .map(|t| {
            let t = t.trim();
            let digitos: String = t.chars().take_while(|c| c.is_ascii_digit()).collect();
            let coef = if digitos.is_empty() { 1 } else { digitos.parse().unwrap() };
            let nombre = t[digitos.len()..].trim();
            let (n, smiles) = buscar(nombre).ok_or(format!("no conozco '{nombre}' (usa un nombre o fórmula del catálogo)"))?;
            Ok((coef, leer_smiles(n, smiles)?))
        })
        .collect()
}

pub fn leer_reaccion(s: &str) -> Result<Reaccion, String> {
    let (izq, der) = s.split_once("->").or_else(|| s.split_once('→')).ok_or("falta '->'")?;
    Ok(Reaccion { reactivos: lado(izq)?, productos: lado(der)? })
}

impl Reaccion {
    fn atomos(lado: &[(u32, Molecula)]) -> BTreeMap<u32, usize> {
        let mut c = BTreeMap::new();
        for (k, m) in lado {
            for (z, n) in m.conteo() {
                *c.entry(z).or_default() += n * *k as usize;
            }
        }
        c
    }

    /// Ley de conservación de la masa (Lavoisier): los mismos átomos a ambos lados.
    pub fn balanceada(&self) -> bool {
        Self::atomos(&self.reactivos) == Self::atomos(&self.productos)
    }

    /// ΔH ≈ Σ enlaces rotos − Σ enlaces formados (kJ por mol de reacción).
    pub fn entalpia_kj(&self) -> f64 {
        let e = |l: &[(u32, Molecula)]| l.iter().map(|(k, m)| *k as f64 * m.energia_enlaces_kj()).sum::<f64>();
        e(&self.reactivos) - e(&self.productos)
    }

    pub fn conteo_texto(lado: &[(u32, Molecula)]) -> String {
        Self::atomos(lado).iter().map(|(z, n)| format!("{}:{n}", SIMBOLOS[(*z - 1) as usize])).collect::<Vec<_>>().join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mol(n: &str) -> Molecula {
        let (a, s) = buscar(n).unwrap();
        leer_smiles(a, s).unwrap()
    }

    #[test]
    fn formulas_del_catalogo() {
        // Compara átomos, no texto: Hill escribe H3N lo que se suele escribir NH3.
        for (n, s, f) in CATALOGO {
            let m = leer_smiles(n, s).unwrap();
            let mut esperado = BTreeMap::new();
            let c: Vec<char> = f.chars().collect();
            let mut i = 0;
            while i < c.len() {
                let mut sim = c[i].to_string();
                i += 1;
                if i < c.len() && c[i].is_ascii_lowercase() { sim.push(c[i]); i += 1; }
                let mut num = String::new();
                while i < c.len() && c[i].is_ascii_digit() { num.push(c[i]); i += 1; }
                *esperado.entry(z_de_simbolo(&sim).unwrap()).or_insert(0usize) += num.parse::<usize>().unwrap_or(1);
            }
            assert_eq!(m.conteo(), esperado, "{n}");
        }
    }

    #[test]
    fn agua_angulo_y_longitud() {
        let m = mol("agua");
        let ang = m.angulo(1, 0, 2);
        assert!((ang - 104.5).abs() < 2.0, "H–O–H = {ang}");
        let d = m.distancia(0, 1);
        assert!((d - 0.97).abs() < 0.03, "O–H = {d} Å");
        assert_eq!(m.geometria_de(0), "angular");
        assert!((m.masa_molar() - 18.015).abs() < 0.01);
        let mu = m.dipolo_debye();
        assert!((mu - 1.85).abs() < 0.3, "μ = {mu} D");
    }

    #[test]
    fn co2_lineal_y_apolar() {
        let m = mol("CO2");
        assert!((m.angulo(0, 1, 2) - 180.0).abs() < 2.0);
        assert_eq!(m.geometria_de(1), "lineal");
        assert!(m.dipolo_debye() < 0.1, "los dipolos se cancelan");
    }

    #[test]
    fn metano_tetraedrico_y_amoniaco_piramidal() {
        assert_eq!(mol("metano").geometria_de(0), "tetraédrica");
        let a = mol("amoníaco");
        assert_eq!(a.geometria_de(0), "piramidal trigonal");
        assert!((a.angulo(1, 0, 2) - 107.0).abs() < 2.0);
    }

    #[test]
    fn benceno_plano_y_aromatico() {
        let m = mol("benceno");
        assert_eq!(m.formula(), "C6H6");
        // Todos los C–C–C a ~120°.
        for i in 0..6 {
            let a = m.angulo((i + 5) % 6, i, (i + 1) % 6);
            assert!((a - 120.0).abs() < 3.0, "C–C–C = {a}");
        }
    }

    #[test]
    fn combustion_del_hidrogeno() {
        let r = leer_reaccion("2H2 + O2 -> 2H2O").unwrap();
        assert!(r.balanceada());
        let dh = r.entalpia_kj();
        assert!((dh + 482.0).abs() < 5.0, "ΔH = {dh}");
    }

    #[test]
    fn combustion_del_metano() {
        let r = leer_reaccion("CH4 + 2O2 -> CO2 + 2H2O").unwrap();
        assert!(r.balanceada());
        assert!((r.entalpia_kj() + 802.0).abs() < 10.0);
    }

    #[test]
    fn detecta_desbalance() {
        assert!(!leer_reaccion("H2 + O2 -> H2O").unwrap().balanceada());
    }

    #[test]
    fn tipos_de_enlace() {
        assert_eq!(tipo_enlace(1, 1).0.nombre(), "covalente apolar");
        assert_eq!(tipo_enlace(1, 8).0.nombre(), "covalente polar");
        assert_eq!(tipo_enlace(11, 17).0.nombre(), "iónico");
    }

    #[test]
    fn atp_tiene_tres_fosfatos() {
        let m = mol("ATP");
        assert_eq!(m.conteo()[&15], 3);
        assert!((m.masa_molar() - 507.18).abs() < 0.5);
    }
}
#[cfg(test)]
mod diag_q {
    #[test]
    #[ignore]
    fn diag_atp() {
        let (a, s) = super::buscar("ATP").unwrap();
        let m = super::leer_smiles(a, s).unwrap();
        eprintln!("{} {}", m.formula(), m.masa_molar());
    }
}
