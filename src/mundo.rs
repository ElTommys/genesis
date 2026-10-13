//! Escala 2-3: el mundo químico. Átomos en una caja 3D periódica que se mueven y
//! reaccionan solos, como en el Juego de la Vida: no hay guion, solo reglas locales.
//!
//! Unidades reducidas: longitud σ ≈ 1 Å, energía ε, masa en u.
//! - Átomos no enlazados: Lennard-Jones (repulsión de Pauli + atracción de van der Waals).
//! - Cargas: iones (protones − electrones) + cargas parciales δ+/δ− que salen de la
//!   diferencia de electronegatividad de Pauling en cada enlace → puentes de hidrógeno.
//! - Enlaces: resorte armónico; se forman si dos átomos con valencia libre se rozan y
//!   se rompen por agitación térmica (factor de Boltzmann) o estiramiento.
//! - Geometría: ángulo de enlace por VSEPR (pares solitarios), p. ej. H₂O → 104.5°.

use crate::azar::Rng;
use crate::cuantica::Atomo;
use crate::elementos::ELECTRONEGATIVIDAD;
use std::collections::BTreeMap;

/// Todas las perillas que se pueden editar para experimentar.
#[derive(Debug, Clone)]
pub struct Parametros {
    pub caja: f64,
    pub temperatura: f64,
    pub dt: f64,
    pub corte_lj: f64,
    pub epsilon_lj: f64,
    pub k_enlace: f64,
    pub r_enlace: f64,
    pub r_formacion: f64,
    pub r_ruptura: f64,
    pub energia_enlace: f64,
    pub prob_reaccion: f64,
    pub tasa_ruptura: f64,
    pub k_coulomb: f64,
    pub k_polar: f64,
    pub k_angulo: f64,
    pub semilla: u64,
}

impl Default for Parametros {
    fn default() -> Self {
        Parametros {
            caja: 20.0,
            temperatura: 1.0,
            dt: 0.002,
            corte_lj: 2.5,
            epsilon_lj: 1.0,
            k_enlace: 200.0,
            r_enlace: 1.0,
            r_formacion: 1.25,
            r_ruptura: 1.8,
            energia_enlace: 8.0,
            prob_reaccion: 0.5,
            tasa_ruptura: 1.0,
            k_coulomb: 5.0,
            k_polar: 0.4,
            k_angulo: 60.0,
            semilla: 42,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cuerpo {
    pub z: u32,
    pub carga: i32,
    /// Carga efectiva = carga iónica + δ parcial de sus enlaces.
    pub q: f64,
    pub masa: f64,
    pub valencia: u32,
    pub pares_libres: u32,
    pub electroneg: f64,
    pub pos: [f64; 3],
    pub vel: [f64; 3],
    fuerza: [f64; 3],
}

pub struct Mundo {
    pub p: Parametros,
    pub cuerpos: Vec<Cuerpo>,
    pub enlaces: Vec<Vec<usize>>,
    rng: Rng,
    pub paso: u64,
}

impl Mundo {
    pub fn nuevo(p: Parametros) -> Self {
        let rng = Rng::nuevo(p.semilla);
        Mundo { p, cuerpos: Vec::new(), enlaces: Vec::new(), rng, paso: 0 }
    }

    /// Siembra `cantidad` átomos del elemento `z` con `electrones_extra`
    /// (−1 = catión, +1 = anión) en posiciones aleatorias sin solaparse.
    pub fn sembrar(&mut self, z: u32, cantidad: usize, electrones_extra: i32) {
        let atomo = Atomo::nuevo(z);
        let electrones = z as i32 + electrones_extra;
        let carga = z as i32 - electrones;
        let valencia = if carga != 0 { 0 } else { atomo.valencia_enlace() };
        let pares_libres = match atomo.bloque() {
            's' | 'p' if carga == 0 && z > 2 => (atomo.electrones_valencia().saturating_sub(valencia)) / 2,
            _ => 0,
        };
        let sigma_v = self.p.temperatura.sqrt();
        for _ in 0..cantidad {
            let pos = self.posicion_libre();
            let masa = atomo.masa_u();
            let s = sigma_v / masa.sqrt();
            let vel = [self.rng.normal() * s, self.rng.normal() * s, self.rng.normal() * s];
            self.cuerpos.push(Cuerpo {
                z,
                carga,
                q: carga as f64,
                masa,
                valencia,
                pares_libres,
                electroneg: ELECTRONEGATIVIDAD[(z - 1) as usize],
                pos,
                vel,
                fuerza: [0.0; 3],
            });
            self.enlaces.push(Vec::new());
        }
    }

    fn posicion_libre(&mut self) -> [f64; 3] {
        let l = self.p.caja;
        for _ in 0..1000 {
            let c = [self.rng.unif() * l, self.rng.unif() * l, self.rng.unif() * l];
            if self.cuerpos.iter().all(|b| dist2(&self.p, &b.pos, &c) > 0.81) {
                return c;
            }
        }
        panic!("La caja está demasiado llena: aumenta --caja");
    }

    /// Coloca una molécula enlazada a mano (para experimentos controlados).
    pub fn enlazar(&mut self, i: usize, j: usize) {
        self.enlaces[i].push(j);
        self.enlaces[j].push(i);
        self.actualizar_cargas();
    }

    pub fn avanzar(&mut self, pasos: u64) {
        if self.paso == 0 {
            self.actualizar_cargas();
            self.calcular_fuerzas();
        }
        for _ in 0..pasos {
            self.verlet();
            self.reacciones();
            self.termostato();
            self.paso += 1;
        }
    }

    fn verlet(&mut self) {
        let dt = self.p.dt;
        let l = self.p.caja;
        for b in &mut self.cuerpos {
            for k in 0..3 {
                b.vel[k] += 0.5 * dt * b.fuerza[k] / b.masa;
                b.pos[k] = (b.pos[k] + dt * b.vel[k]).rem_euclid(l);
            }
        }
        self.calcular_fuerzas();
        for b in &mut self.cuerpos {
            for k in 0..3 {
                b.vel[k] += 0.5 * dt * b.fuerza[k] / b.masa;
            }
        }
    }

    /// δ de cada átomo: el más electronegativo del enlace se queda con densidad electrónica.
    fn actualizar_cargas(&mut self) {
        for i in 0..self.cuerpos.len() {
            let xi = self.cuerpos[i].electroneg;
            let delta: f64 = self.enlaces[i].iter().map(|&j| self.cuerpos[j].electroneg - xi).sum();
            self.cuerpos[i].q = self.cuerpos[i].carga as f64 + self.p.k_polar * delta;
        }
    }

    /// Ángulo de equilibrio por VSEPR: número estérico = enlaces + pares solitarios;
    /// cada par solitario cierra el ángulo ~2.5° (H₂O 104.5°, NH₃ 107°, CH₄ 109.5°).
    pub fn angulo_objetivo(&self, i: usize) -> f64 {
        let b = &self.cuerpos[i];
        let base = match b.valencia + b.pares_libres {
            0..=2 => 180.0,
            3 => 120.0,
            4 => 109.47,
            _ => 90.0,
        };
        (base - 2.5 * b.pares_libres as f64).to_radians()
    }

    fn excluido(&self, i: usize, j: usize) -> bool {
        self.enlaces[i].contains(&j) || self.enlaces[i].iter().any(|&k| self.enlaces[k].contains(&j))
    }

    /// Lista de celdas: cada átomo solo mira a sus vecinos cercanos → O(N) en vez de O(N²).
    fn pares_cercanos(&self, corte: f64) -> Vec<(usize, usize, [f64; 3], f64)> {
        let l = self.p.caja;
        let nc = ((l / corte).floor() as usize).max(1);
        let tam = l / nc as f64;
        let idx = |x: f64| ((x / tam) as usize).min(nc - 1);
        let mut celdas: Vec<Vec<usize>> = vec![Vec::new(); nc * nc * nc];
        for (i, b) in self.cuerpos.iter().enumerate() {
            celdas[(idx(b.pos[0]) * nc + idx(b.pos[1])) * nc + idx(b.pos[2])].push(i);
        }
        let mut pares = Vec::new();
        let corte2 = corte * corte;
        let rango: Vec<isize> = if nc >= 3 { vec![-1, 0, 1] } else { (0..nc as isize).collect() };
        for cx in 0..nc {
            for cy in 0..nc {
                for cz in 0..nc {
                    let a = &celdas[(cx * nc + cy) * nc + cz];
                    for &dx in &rango {
                        for &dy in &rango {
                            for &dz in &rango {
                                let (ox, oy, oz) = if nc >= 3 {
                                    (wrap(cx, dx, nc), wrap(cy, dy, nc), wrap(cz, dz, nc))
                                } else {
                                    (dx as usize, dy as usize, dz as usize)
                                };
                                let b = &celdas[(ox * nc + oy) * nc + oz];
                                for &i in a {
                                    for &j in b {
                                        if j <= i {
                                            continue;
                                        }
                                        let d = delta(&self.p, &self.cuerpos[i].pos, &self.cuerpos[j].pos);
                                        let r2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
                                        if r2 < corte2 {
                                            pares.push((i, j, d, r2));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        pares
    }

    fn calcular_fuerzas(&mut self) {
        for b in &mut self.cuerpos {
            b.fuerza = [0.0; 3];
        }
        let p = self.p.clone();
        let corte = p.corte_lj.max(p.r_formacion);
        let lj_c2 = p.corte_lj * p.corte_lj;
        for (i, j, d, r2) in self.pares_cercanos(corte) {
            if self.excluido(i, j) {
                continue;
            }
            let mut f_r = 0.0; // fuerza escalar / r
            if r2 < lj_c2 {
                let inv2 = 1.0 / r2.max(0.36);
                let inv6 = inv2 * inv2 * inv2;
                f_r += 24.0 * p.epsilon_lj * inv2 * (2.0 * inv6 * inv6 - inv6);
            }
            let qq = self.cuerpos[i].q * self.cuerpos[j].q;
            if qq != 0.0 {
                let r = r2.sqrt().max(0.6);
                f_r += p.k_coulomb * qq / (r * r * r);
            }
            aplicar(&mut self.cuerpos, i, j, d, f_r);
        }
        // Coulomb de largo alcance entre iones (pocos, se hace directo).
        let iones: Vec<usize> = (0..self.cuerpos.len()).filter(|&i| self.cuerpos[i].carga != 0).collect();
        for (a, &i) in iones.iter().enumerate() {
            for &j in &iones[a + 1..] {
                let d = delta(&p, &self.cuerpos[i].pos, &self.cuerpos[j].pos);
                let r2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
                if r2 >= corte * corte {
                    let r = r2.sqrt();
                    let qq = (self.cuerpos[i].carga * self.cuerpos[j].carga) as f64;
                    aplicar(&mut self.cuerpos, i, j, d, p.k_coulomb * qq * (-r / p.caja).exp() / (r * r * r));
                }
            }
        }
        for i in 0..self.cuerpos.len() {
            let vecinos = self.enlaces[i].clone();
            for &j in &vecinos {
                if j > i {
                    let d = delta(&p, &self.cuerpos[i].pos, &self.cuerpos[j].pos);
                    let r = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                    aplicar(&mut self.cuerpos, i, j, d, -p.k_enlace * (r - p.r_enlace) / r);
                }
            }
            // Ángulos: átomo central i con cada par de vecinos (a, c).
            let theta0 = self.angulo_objetivo(i);
            for x in 0..vecinos.len() {
                for y in x + 1..vecinos.len() {
                    self.fuerza_angulo(vecinos[x], i, vecinos[y], theta0);
                }
            }
        }
    }

    fn fuerza_angulo(&mut self, a: usize, j: usize, c: usize, theta0: f64) {
        let p = &self.p;
        let ra = delta(p, &self.cuerpos[a].pos, &self.cuerpos[j].pos);
        let rc = delta(p, &self.cuerpos[c].pos, &self.cuerpos[j].pos);
        let la = norma(&ra);
        let lc = norma(&rc);
        let cos = (punto(&ra, &rc) / (la * lc)).clamp(-0.9999, 0.9999);
        let sen = (1.0 - cos * cos).sqrt();
        let pref = p.k_angulo * (cos.acos() - theta0) / sen;
        for k in 0..3 {
            let fa = pref * (rc[k] / (la * lc) - cos * ra[k] / (la * la));
            let fc = pref * (ra[k] / (la * lc) - cos * rc[k] / (lc * lc));
            self.cuerpos[a].fuerza[k] += fa;
            self.cuerpos[c].fuerza[k] += fc;
            self.cuerpos[j].fuerza[k] -= fa + fc;
        }
    }

    fn reacciones(&mut self) {
        let p = self.p.clone();
        let mut cambio = false;
        // Ruptura: estiramiento o agitación térmica (factor de Boltzmann).
        let prob_termica = p.tasa_ruptura * p.dt * (-p.energia_enlace / p.temperatura.max(1e-9)).exp();
        let mut romper = Vec::new();
        for i in 0..self.cuerpos.len() {
            for &j in &self.enlaces[i] {
                if j > i {
                    let r2 = dist2(&p, &self.cuerpos[i].pos, &self.cuerpos[j].pos);
                    if r2 > p.r_ruptura * p.r_ruptura || self.rng.unif() < prob_termica {
                        romper.push((i, j));
                    }
                }
            }
        }
        for (i, j) in romper {
            self.enlaces[i].retain(|&x| x != j);
            self.enlaces[j].retain(|&x| x != i);
            cambio = true;
        }
        // Formación: dos átomos con valencia libre que se rozan.
        for (i, j, _, _) in self.pares_cercanos(p.r_formacion) {
            if self.libre(i) > 0
                && self.libre(j) > 0
                && !self.enlaces[i].contains(&j)
                && self.rng.unif() < p.prob_reaccion
            {
                self.enlaces[i].push(j);
                self.enlaces[j].push(i);
                cambio = true;
            }
        }
        if cambio {
            self.actualizar_cargas();
        }
    }

    fn libre(&self, i: usize) -> u32 {
        self.cuerpos[i].valencia.saturating_sub(self.enlaces[i].len() as u32)
    }

    /// Termostato de Berendsen: acerca la temperatura a la elegida (baño térmico).
    fn termostato(&mut self) {
        let t = self.temperatura();
        if t > 0.0 {
            let lambda = (1.0 + 0.01 * (self.p.temperatura / t - 1.0)).sqrt();
            for b in &mut self.cuerpos {
                for k in 0..3 {
                    b.vel[k] *= lambda;
                }
            }
        }
    }

    pub fn temperatura(&self) -> f64 {
        let ec: f64 = self.cuerpos.iter().map(|b| 0.5 * b.masa * punto(&b.vel, &b.vel)).sum();
        2.0 * ec / (3.0 * self.cuerpos.len().max(1) as f64)
    }

    pub fn num_enlaces(&self) -> usize {
        self.enlaces.iter().map(|v| v.len()).sum::<usize>() / 2
    }

    /// Ángulo (grados) entre los enlaces a–j–c.
    pub fn angulo(&self, a: usize, j: usize, c: usize) -> f64 {
        let ra = delta(&self.p, &self.cuerpos[a].pos, &self.cuerpos[j].pos);
        let rc = delta(&self.p, &self.cuerpos[c].pos, &self.cuerpos[j].pos);
        (punto(&ra, &rc) / (norma(&ra) * norma(&rc))).clamp(-1.0, 1.0).acos().to_degrees()
    }

    /// Ángulo medio de todos los H–X–H donde X es el elemento `z`.
    pub fn angulo_medio(&self, z: u32) -> Option<f64> {
        let mut suma = 0.0;
        let mut n = 0;
        for j in 0..self.cuerpos.len() {
            if self.cuerpos[j].z != z {
                continue;
            }
            let v = &self.enlaces[j];
            for x in 0..v.len() {
                for y in x + 1..v.len() {
                    suma += self.angulo(v[x], j, v[y]);
                    n += 1;
                }
            }
        }
        (n > 0).then(|| suma / n as f64)
    }

    /// Censo de moléculas: agrupa átomos conectados y escribe su fórmula.
    pub fn censo(&self) -> BTreeMap<String, usize> {
        let n = self.cuerpos.len();
        let mut visto = vec![false; n];
        let mut censo = BTreeMap::new();
        for s in 0..n {
            if visto[s] {
                continue;
            }
            let mut pila = vec![s];
            visto[s] = true;
            let mut cuenta: BTreeMap<u32, usize> = BTreeMap::new();
            let mut carga = 0;
            while let Some(i) = pila.pop() {
                *cuenta.entry(self.cuerpos[i].z).or_default() += 1;
                carga += self.cuerpos[i].carga;
                for &j in &self.enlaces[i] {
                    if !visto[j] {
                        visto[j] = true;
                        pila.push(j);
                    }
                }
            }
            *censo.entry(formula(&cuenta, carga)).or_default() += 1;
        }
        censo
    }
}

/// Notación de Hill: C primero, luego H, luego alfabético.
pub fn formula(cuenta: &BTreeMap<u32, usize>, carga: i32) -> String {
    let mut partes: Vec<(&str, usize)> = cuenta
        .iter()
        .map(|(&z, &n)| (crate::elementos::SIMBOLOS[(z - 1) as usize], n))
        .collect();
    let tiene_c = cuenta.contains_key(&6);
    partes.sort_by_key(|&(s, _)| match s {
        "C" if tiene_c => (0, s),
        "H" if tiene_c => (1, s),
        _ => (2, s),
    });
    let mut f: String = partes
        .iter()
        .map(|&(s, n)| if n == 1 { s.to_string() } else { format!("{s}{n}") })
        .collect();
    match carga {
        0 => {}
        1 => f.push('+'),
        -1 => f.push('-'),
        c if c > 0 => f.push_str(&format!("{c}+")),
        c => f.push_str(&format!("{}-", -c)),
    }
    f
}

fn aplicar(c: &mut [Cuerpo], i: usize, j: usize, d: [f64; 3], f_r: f64) {
    for k in 0..3 {
        c[i].fuerza[k] += f_r * d[k];
        c[j].fuerza[k] -= f_r * d[k];
    }
}

fn punto(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn norma(a: &[f64; 3]) -> f64 {
    punto(a, a).sqrt()
}

/// Vector a−b con imagen mínima (caja periódica).
pub fn delta(p: &Parametros, a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    let l = p.caja;
    let mut d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    for x in &mut d {
        *x -= l * (*x / l).round();
    }
    d
}

fn dist2(p: &Parametros, a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let d = delta(p, a, b);
    punto(&d, &d)
}

fn wrap(c: usize, d: isize, n: usize) -> usize {
    (c as isize + d).rem_euclid(n as isize) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hidrogeno_y_oxigeno_forman_agua() {
        let mut m = Mundo::nuevo(Parametros { caja: 8.0, ..Default::default() });
        m.sembrar(1, 40, 0);
        m.sembrar(8, 20, 0);
        m.avanzar(5000);
        let c = m.censo();
        assert!(c.keys().any(|f| f == "H2O"), "censo: {c:?}");
    }

    #[test]
    fn agua_es_polar_y_doblada() {
        let mut m = Mundo::nuevo(Parametros { caja: 10.0, temperatura: 0.05, energia_enlace: 50.0, ..Default::default() });
        m.sembrar(8, 1, 0);
        m.sembrar(1, 2, 0);
        // Coloca la molécula a mano, abierta a 150°, y deja que la física la doble.
        let o = m.cuerpos[0].pos;
        let a = 75f64.to_radians();
        m.cuerpos[1].pos = [o[0] + a.sin(), o[1] + a.cos(), o[2]];
        m.cuerpos[2].pos = [o[0] - a.sin(), o[1] + a.cos(), o[2]];
        m.enlazar(0, 1);
        m.enlazar(0, 2);
        assert!(m.cuerpos[0].q < -0.9 && m.cuerpos[1].q > 0.4, "O δ− y H δ+");
        m.avanzar(4000);
        let ang = m.angulo_medio(8).unwrap();
        assert!((ang - 104.5).abs() < 3.0, "ángulo H–O–H = {ang}");
    }

    #[test]
    fn metano_tetraedrico() {
        let mut m = Mundo::nuevo(Parametros { caja: 10.0, temperatura: 0.05, energia_enlace: 50.0, ..Default::default() });
        m.sembrar(6, 1, 0);
        m.sembrar(1, 4, 0);
        let c = m.cuerpos[0].pos;
        let dirs = [[1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        for (k, d) in dirs.iter().enumerate() {
            m.cuerpos[k + 1].pos = [c[0] + d[0], c[1] + d[1], c[2] + d[2]];
            m.enlazar(0, k + 1);
        }
        m.avanzar(6000);
        let ang = m.angulo_medio(6).unwrap();
        assert!((ang - 109.5).abs() < 3.0, "ángulo H–C–H = {ang}");
    }

    #[test]
    fn gases_nobles_no_reaccionan() {
        let mut m = Mundo::nuevo(Parametros { caja: 6.0, ..Default::default() });
        m.sembrar(10, 40, 0);
        m.avanzar(1000);
        assert_eq!(m.num_enlaces(), 0);
    }

    #[test]
    fn valencia_nunca_se_excede() {
        let mut m = Mundo::nuevo(Parametros { caja: 7.0, ..Default::default() });
        m.sembrar(6, 20, 0);
        m.sembrar(1, 40, 0);
        m.avanzar(2000);
        for i in 0..m.cuerpos.len() {
            assert!(m.enlaces[i].len() as u32 <= m.cuerpos[i].valencia);
        }
    }

    #[test]
    fn formula_hill() {
        let mut c = BTreeMap::new();
        c.insert(8, 1);
        c.insert(6, 1);
        c.insert(1, 4);
        assert_eq!(formula(&c, 0), "CH4O");
    }
}
