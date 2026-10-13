//! Escala 0: partículas subatómicas.
//! Masas en unidades de masa atómica (u), cargas en unidades de carga elemental (e).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Particula {
    Proton,
    Neutron,
    Electron,
}

impl Particula {
    pub fn carga(self) -> i32 {
        match self {
            Particula::Proton => 1,
            Particula::Neutron => 0,
            Particula::Electron => -1,
        }
    }

    pub fn masa_u(self) -> f64 {
        match self {
            Particula::Proton => 1.007_276,
            Particula::Neutron => 1.008_665,
            Particula::Electron => 0.000_548_6,
        }
    }

    /// Los tres son fermiones de spin 1/2.
    pub fn spin(self) -> f64 {
        0.5
    }

    pub fn nombre(self) -> &'static str {
        match self {
            Particula::Proton => "protón",
            Particula::Neutron => "neutrón",
            Particula::Electron => "electrón",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomo_neutro_suma_cero() {
        let z = 8;
        let carga = z * Particula::Proton.carga() + z * Particula::Electron.carga();
        assert_eq!(carga, 0);
    }
}
