//! Escala 1: el átomo. La tabla periódica NO está escrita a mano:
//! se deduce de los números cuánticos (n, l, mₗ, mₛ), el principio de Aufbau
//! (regla de Madelung), el principio de exclusión de Pauli y la regla de Hund.

use crate::particula::Particula;

const LETRAS: [char; 4] = ['s', 'p', 'd', 'f'];

/// Una subcapa (n, l) con su ocupación electrónica.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Subcapa {
    pub n: u32,
    pub l: u32,
    pub electrones: u32,
}

impl Subcapa {
    /// Orbitales = valores posibles de mₗ (−l..=l).
    pub fn orbitales(&self) -> u32 {
        2 * self.l + 1
    }
    /// Pauli: cada orbital admite 2 electrones (mₛ = +½, −½).
    pub fn capacidad(&self) -> u32 {
        2 * self.orbitales()
    }
    /// Hund: se llenan primero los orbitales con un electrón cada uno, spin paralelo.
    pub fn desapareados(&self) -> u32 {
        let o = self.orbitales();
        if self.electrones <= o { self.electrones } else { 2 * o - self.electrones }
    }
    pub fn etiqueta(&self) -> String {
        format!("{}{}{}", self.n, LETRAS[self.l as usize], superindice(self.electrones))
    }
}

/// Orden de llenado de Madelung: por n+l creciente, y a igual n+l, por n creciente.
pub fn orden_aufbau() -> Vec<(u32, u32)> {
    let mut v: Vec<(u32, u32)> = (1..=7)
        .flat_map(|n| (0..n.min(4)).map(move |l| (n, l)))
        .collect();
    v.sort_by_key(|&(n, l)| (n + l, n));
    v
}

#[derive(Debug, Clone)]
pub struct Atomo {
    pub z: u32,
    pub neutrones: u32,
    pub config: Vec<Subcapa>,
}

impl Atomo {
    pub fn nuevo(z: u32) -> Self {
        assert!((1..=118).contains(&z), "Z debe estar entre 1 y 118");
        let mut restantes = z;
        let mut config = Vec::new();
        for (n, l) in orden_aufbau() {
            if restantes == 0 {
                break;
            }
            let cap = Subcapa { n, l, electrones: 0 }.capacidad();
            let e = restantes.min(cap);
            config.push(Subcapa { n, l, electrones: e });
            restantes -= e;
        }
        Atomo { z, neutrones: neutrones_isotopo_comun(z), config }
    }

    pub fn simbolo(&self) -> &'static str {
        crate::elementos::SIMBOLOS[(self.z - 1) as usize]
    }
    pub fn nombre(&self) -> &'static str {
        crate::elementos::NOMBRES[(self.z - 1) as usize]
    }

    pub fn numero_masico(&self) -> u32 {
        self.z + self.neutrones
    }
    pub fn masa_u(&self) -> f64 {
        self.z as f64 * (Particula::Proton.masa_u() + Particula::Electron.masa_u())
            + self.neutrones as f64 * Particula::Neutron.masa_u()
    }

    /// Periodo = la capa n más alta ocupada.
    pub fn periodo(&self) -> u32 {
        self.config.iter().map(|s| s.n).max().unwrap()
    }

    /// Bloque = tipo de la última subcapa en llenarse.
    pub fn bloque(&self) -> char {
        LETRAS[self.config.last().unwrap().l as usize]
    }

    fn electrones_en(&self, n: u32, l: u32) -> u32 {
        self.config.iter().filter(|s| s.n == n && s.l == l).map(|s| s.electrones).sum()
    }

    /// Grupo IUPAC (1..18). Lantánidos y actínidos (bloque f) devuelven None.
    pub fn grupo(&self) -> Option<u32> {
        let p = self.periodo();
        let s = self.electrones_en(p, 0);
        match self.bloque() {
            's' if self.z == 2 => Some(18),
            's' => Some(s),
            'p' => Some(10 + s + self.electrones_en(p, 1)),
            'd' => Some(s + self.electrones_en(p - 1, 2)),
            _ => None,
        }
    }

    /// Electrones de valencia (capa externa s+p; para el bloque d suma los d).
    pub fn electrones_valencia(&self) -> u32 {
        let p = self.periodo();
        let base = self.electrones_en(p, 0) + self.electrones_en(p, 1);
        if self.bloque() == 'd' { base + self.electrones_en(p - 1, 2) } else { base }
    }

    /// Cuántos enlaces covalentes tiende a formar (regla del octeto / dueto).
    pub fn valencia_enlace(&self) -> u32 {
        if self.grupo() == Some(18) {
            return 0;
        }
        if self.z == 1 {
            return 1;
        }
        match self.bloque() {
            's' | 'p' => {
                let v = self.electrones_valencia();
                if v <= 4 { v } else { 8 - v }
            }
            _ => 2,
        }
    }

    /// Spin total del estado fundamental: S = desapareados × ½.
    pub fn spin_total(&self) -> f64 {
        self.config.iter().map(|s| s.desapareados()).sum::<u32>() as f64 * 0.5
    }

    pub fn configuracion(&self) -> String {
        self.config.iter().map(|s| s.etiqueta()).collect::<Vec<_>>().join(" ")
    }
}

/// Aproximación del isótopo más común. Exacto para H y He; para el resto
/// sigue la línea de estabilidad nuclear (N/Z crece con Z).
fn neutrones_isotopo_comun(z: u32) -> u32 {
    match z {
        1 => 0,
        2 => 2,
        _ => {
            let zf = z as f64;
            (zf + 0.0060 * zf * zf).round() as u32
        }
    }
}

fn superindice(n: u32) -> String {
    const S: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];
    n.to_string().chars().map(|c| S[c.to_digit(10).unwrap() as usize]).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oxigeno() {
        let o = Atomo::nuevo(8);
        assert_eq!(o.configuracion(), "1s² 2s² 2p⁴");
        assert_eq!(o.periodo(), 2);
        assert_eq!(o.grupo(), Some(16));
        assert_eq!(o.valencia_enlace(), 2);
        assert_eq!(o.spin_total(), 1.0); // dos electrones desapareados
    }

    #[test]
    fn carbono_y_nitrogeno() {
        assert_eq!(Atomo::nuevo(6).valencia_enlace(), 4);
        assert_eq!(Atomo::nuevo(7).valencia_enlace(), 3);
        assert_eq!(Atomo::nuevo(7).spin_total(), 1.5);
    }

    #[test]
    fn grupos_emergen_de_la_cuantica() {
        assert_eq!(Atomo::nuevo(1).grupo(), Some(1));
        assert_eq!(Atomo::nuevo(2).grupo(), Some(18));
        assert_eq!(Atomo::nuevo(11).grupo(), Some(1)); // Na
        assert_eq!(Atomo::nuevo(26).grupo(), Some(8)); // Fe
        assert_eq!(Atomo::nuevo(35).grupo(), Some(17)); // Br
        assert_eq!(Atomo::nuevo(54).grupo(), Some(18)); // Xe
        assert_eq!(Atomo::nuevo(60).grupo(), None); // Nd, lantánido
        assert_eq!(Atomo::nuevo(118).periodo(), 7);
    }

    #[test]
    fn hierro_4s_antes_que_3d() {
        assert_eq!(Atomo::nuevo(26).configuracion(), "1s² 2s² 2p⁶ 3s² 3p⁶ 4s² 3d⁶");
    }
}
