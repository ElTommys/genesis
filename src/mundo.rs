//! Escala 2: el mundo. Átomos en una caja 3D periódica que se mueven y
//! reaccionan solos, como en el Juego de la Vida: no hay guion, solo reglas locales.
//!
//! Unidades reducidas: longitud σ ≈ 1 Å, energía ε, masa en u.
//! - Átomos no enlazados: Lennard-Jones (repulsión de Pauli + atracción de van der Waals).
//! - Iones: Coulomb con apantallamiento (carga = protones − electrones).
//! - Átomos enlazados: resorte armónico.
//! - Un enlace se forma si dos átomos con valencia libre se acercan y se rompe
//!   por agitación térmica (probabilidad de Boltzmann) o estiramiento.

use crate::cuantica::Atomo;
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
            semilla: 42,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cuerpo {
    pub z: u32,
    pub carga: i32,
    pub masa: f64,
    pub valencia: u32,
    pub pos: [f64; 3],
    pub vel: [f64; 3],
    fuerza: [f64; 3],
}

pub struct Mundo {
    pub p: Parametros,
    pub cuerpos: Vec<Cuerpo>,
    vecinos_enlace: Vec<Vec<usize>>,
    rng: Rng,
    pub paso: u64,
}

impl Mundo {
    pub fn nuevo(p: Parametros) -> Self {
        let rng = Rng(p.semilla.max(1));
        Mundo { p, cuerpos: Vec::new(), vecinos_enlace: Vec::new(), rng, paso: 0 }
    }

    /// Siembra `cantidad` átomos del elemento `z` con `electrones_extra`
    /// (−1 = catión, +1 = anión) en posiciones aleatorias sin solaparse.
    pub fn sembrar(&mut self, z: u32, cantidad: usize, electrones_extra: i32) {
        let atomo = Atomo::nuevo(z);
        let electrones = z as i32 + electrones_extra;
        let carga = z as i32 - electrones;
        let valencia = if carga != 0 { 0 } else { atomo.valencia_enlace() };
        let sigma_v = self.p.temperatura.sqrt();
        for _ in 0..cantidad {
            let pos = self.posicion_libre();
            let masa = atomo.masa_u();
            let s = sigma_v / masa.sqrt();
            let vel = [self.rng.normal() * s, self.rng.normal() * s, self.rng.normal() * s];
            self.cuerpos.push(Cuerpo { z, carga, masa, valencia, pos, vel, fuerza: [0.0; 3] });
            self.vecinos_enlace.push(Vec::new());
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

    pub fn avanzar(&mut self, pasos: u64) {
        if self.paso == 0 {
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
            let mut f_r = 0.0; // fuerza escalar / r
            if !self.vecinos_enlace[i].contains(&j) && r2 < lj_c2 {
                let inv2 = 1.0 / r2.max(0.36);
                let inv6 = inv2 * inv2 * inv2;
                f_r += 24.0 * p.epsilon_lj * inv2 * (2.0 * inv6 * inv6 - inv6);
            }
            let qq = (self.cuerpos[i].carga * self.cuerpos[j].carga) as f64;
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
            for &j in &self.vecinos_enlace[i].clone() {
                if j > i {
                    let d = delta(&p, &self.cuerpos[i].pos, &self.cuerpos[j].pos);
                    let r = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                    aplicar(&mut self.cuerpos, i, j, d, -p.k_enlace * (r - p.r_enlace) / r);
                }
            }
        }
    }

    fn reacciones(&mut self) {
        let p = self.p.clone();
        // Ruptura: estiramiento o agitación térmica (factor de Boltzmann).
        let prob_termica = p.tasa_ruptura * p.dt * (-p.energia_enlace / p.temperatura.max(1e-9)).exp();
        let mut romper = Vec::new();
        for i in 0..self.cuerpos.len() {
            for &j in &self.vecinos_enlace[i] {
                if j > i {
                    let r2 = dist2(&p, &self.cuerpos[i].pos, &self.cuerpos[j].pos);
                    if r2 > p.r_ruptura * p.r_ruptura || self.rng.unif() < prob_termica {
                        romper.push((i, j));
                    }
                }
            }
        }
        for (i, j) in romper {
            self.vecinos_enlace[i].retain(|&x| x != j);
            self.vecinos_enlace[j].retain(|&x| x != i);
        }
        // Formación: dos átomos con valencia libre que se rozan.
        for (i, j, _, _) in self.pares_cercanos(p.r_formacion) {
            if self.libre(i) > 0
                && self.libre(j) > 0
                && !self.vecinos_enlace[i].contains(&j)
                && self.rng.unif() < p.prob_reaccion
            {
                self.vecinos_enlace[i].push(j);
                self.vecinos_enlace[j].push(i);
            }
        }
    }

    fn libre(&self, i: usize) -> u32 {
        self.cuerpos[i].valencia.saturating_sub(self.vecinos_enlace[i].len() as u32)
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
        let ec: f64 = self
            .cuerpos
            .iter()
            .map(|b| 0.5 * b.masa * (b.vel[0] * b.vel[0] + b.vel[1] * b.vel[1] + b.vel[2] * b.vel[2]))
            .sum();
        2.0 * ec / (3.0 * self.cuerpos.len().max(1) as f64)
    }

    pub fn num_enlaces(&self) -> usize {
        self.vecinos_enlace.iter().map(|v| v.len()).sum::<usize>() / 2
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
                for &j in &self.vecinos_enlace[i] {
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

/// Vector i−j con imagen mínima (caja periódica).
fn delta(p: &Parametros, a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    let l = p.caja;
    let mut d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    for x in &mut d {
        *x -= l * (*x / l).round();
    }
    d
}

fn dist2(p: &Parametros, a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let d = delta(p, a, b);
    d[0] * d[0] + d[1] * d[1] + d[2] * d[2]
}

fn wrap(c: usize, d: isize, n: usize) -> usize {
    (c as isize + d).rem_euclid(n as isize) as usize
}

/// Generador xorshift: reproducible con la misma semilla.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn unif(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
    fn normal(&mut self) -> f64 {
        let u = self.unif().max(1e-12);
        (-2.0 * u.ln()).sqrt() * (std::f64::consts::TAU * self.unif()).cos()
    }
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
            assert!(m.vecinos_enlace[i].len() as u32 <= m.cuerpos[i].valencia);
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
