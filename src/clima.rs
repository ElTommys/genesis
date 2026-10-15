//! Terreno, clima y biomas del mundo.
//!
//! - Relieve: ruido fractal (fBm) → montañas, llanuras y océanos.
//! - Temperatura: depende de la latitud y baja 6.5 °C por km de altitud (gradiente
//!   térmico de la troposfera estándar).
//! - Lluvia: ruido + humedad que llega del océano + sombra orográfica.
//! - Bioma: clasificación de Whittaker por temperatura media anual y precipitación anual.
//!
//! El mundo es una versión comprimida: unos km de lado contienen gradientes que en
//! la Tierra ocupan miles de km, para que todos los biomas quepan en la vista.

use crate::azar::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoBioma {
    Oceano,
    HieloPolar,
    Tundra,
    Taiga,
    BosqueTemplado,
    Pradera,
    DesiertoFrio,
    Desierto,
    Matorral,
    Sabana,
    SelvaTropical,
}

impl TipoBioma {
    pub const TODOS: [TipoBioma; 11] = [
        TipoBioma::Oceano, TipoBioma::HieloPolar, TipoBioma::Tundra, TipoBioma::Taiga, TipoBioma::BosqueTemplado,
        TipoBioma::Pradera, TipoBioma::DesiertoFrio, TipoBioma::Desierto, TipoBioma::Matorral, TipoBioma::Sabana,
        TipoBioma::SelvaTropical,
    ];

    pub fn nombre(self) -> &'static str {
        match self {
            TipoBioma::Oceano => "Océano",
            TipoBioma::HieloPolar => "Hielo polar",
            TipoBioma::Tundra => "Tundra",
            TipoBioma::Taiga => "Taiga (bosque boreal)",
            TipoBioma::BosqueTemplado => "Bosque templado",
            TipoBioma::Pradera => "Pradera / estepa",
            TipoBioma::DesiertoFrio => "Desierto frío",
            TipoBioma::Desierto => "Desierto cálido",
            TipoBioma::Matorral => "Matorral xerófilo",
            TipoBioma::Sabana => "Sabana",
            TipoBioma::SelvaTropical => "Selva tropical",
        }
    }

    pub fn descripcion(self) -> &'static str {
        match self {
            TipoBioma::Oceano => "Cubre ~71% de la Tierra; el fitoplancton produce cerca de la mitad del oxígeno.",
            TipoBioma::HieloPolar => "Temperatura media < −10 °C; casi sin vida terrestre.",
            TipoBioma::Tundra => "Suelo congelado (permafrost), líquenes y musgos; verano corto.",
            TipoBioma::Taiga => "El bioma terrestre más extenso: coníferas, inviernos largos.",
            TipoBioma::BosqueTemplado => "Árboles caducifolios (robles, hayas), cuatro estaciones.",
            TipoBioma::Pradera => "Gramíneas dominantes; demasiado seco para bosques.",
            TipoBioma::DesiertoFrio => "Menos de 250 mm/año con inviernos fríos (Gobi, Patagonia).",
            TipoBioma::Desierto => "Menos de 250 mm/año, calor extremo de día (Sahara).",
            TipoBioma::Matorral => "Arbustos resistentes a la sequía; lluvias escasas e irregulares.",
            TipoBioma::Sabana => "Pastizal tropical con árboles dispersos; estación seca marcada.",
            TipoBioma::SelvaTropical => "Más de la mitad de las especies terrestres en ~6% de la superficie.",
        }
    }

    pub fn color(self) -> (f32, f32, f32) {
        match self {
            TipoBioma::Oceano => (0.10, 0.28, 0.55),
            TipoBioma::HieloPolar => (0.92, 0.95, 0.98),
            TipoBioma::Tundra => (0.62, 0.66, 0.58),
            TipoBioma::Taiga => (0.18, 0.38, 0.30),
            TipoBioma::BosqueTemplado => (0.25, 0.50, 0.22),
            TipoBioma::Pradera => (0.58, 0.66, 0.30),
            TipoBioma::DesiertoFrio => (0.66, 0.62, 0.50),
            TipoBioma::Desierto => (0.88, 0.76, 0.48),
            TipoBioma::Matorral => (0.62, 0.58, 0.32),
            TipoBioma::Sabana => (0.72, 0.68, 0.30),
            TipoBioma::SelvaTropical => (0.08, 0.42, 0.14),
        }
    }
}

/// Clasificación de Whittaker: temperatura media anual (°C) y precipitación (mm/año).
pub fn whittaker(t: f32, p: f32) -> TipoBioma {
    if t < -10.0 {
        TipoBioma::HieloPolar
    } else if t < -3.0 {
        TipoBioma::Tundra
    } else if t < 4.0 {
        if p < 250.0 { TipoBioma::Tundra } else { TipoBioma::Taiga }
    } else if t < 19.0 {
        if p < 250.0 {
            TipoBioma::DesiertoFrio
        } else if p < 600.0 {
            TipoBioma::Pradera
        } else {
            TipoBioma::BosqueTemplado
        }
    } else if p < 250.0 {
        TipoBioma::Desierto
    } else if p < 600.0 {
        TipoBioma::Matorral
    } else if p < 1500.0 {
        TipoBioma::Sabana
    } else {
        TipoBioma::SelvaTropical
    }
}

/// Ruido de valor suave con permutación aleatoria (reproducible por semilla).
struct Ruido {
    perm: Vec<usize>,
    val: Vec<f32>,
}

impl Ruido {
    fn nuevo(rng: &mut Rng) -> Self {
        let mut perm: Vec<usize> = (0..256).collect();
        for i in (1..256).rev() {
            perm.swap(i, rng.rango(i + 1));
        }
        let val = (0..256).map(|_| rng.unif() as f32 * 2.0 - 1.0).collect();
        Ruido { perm, val }
    }
    fn h(&self, x: i32, y: i32) -> f32 {
        self.val[self.perm[(self.perm[(x & 255) as usize] + (y & 255) as usize) & 255]]
    }
    fn en(&self, x: f32, y: f32) -> f32 {
        let (x0, y0) = (x.floor(), y.floor());
        let (fx, fy) = (x - x0, y - y0);
        let s = |t: f32| t * t * (3.0 - 2.0 * t);
        let (sx, sy) = (s(fx), s(fy));
        let (i, j) = (x0 as i32, y0 as i32);
        let a = self.h(i, j) + sx * (self.h(i + 1, j) - self.h(i, j));
        let b = self.h(i, j + 1) + sx * (self.h(i + 1, j + 1) - self.h(i, j + 1));
        a + sy * (b - a)
    }
    /// Movimiento browniano fraccionario: varias octavas de ruido.
    fn fbm(&self, x: f32, y: f32, octavas: u32) -> f32 {
        let (mut suma, mut amp, mut f, mut norma) = (0.0, 1.0, 1.0, 0.0);
        for _ in 0..octavas {
            suma += amp * self.en(x * f, y * f);
            norma += amp;
            amp *= 0.5;
            f *= 2.0;
        }
        suma / norma
    }
}

#[derive(Clone)]
pub struct Celda {
    /// Altitud en metros (negativa = bajo el mar).
    pub altitud: f32,
    pub temperatura: f32,
    pub lluvia: f32,
    pub bioma: TipoBioma,
}

pub struct Clima {
    pub ancho: f32,
    pub alto: f32,
    pub cols: usize,
    pub filas: usize,
    pub celda: f32,
    pub celdas: Vec<Celda>,
    /// Cambio global de temperatura (°C), editable: calentamiento o glaciación.
    pub delta_t: f32,
    /// Factor global de lluvia, editable.
    pub factor_lluvia: f32,
}

impl Clima {
    pub fn generar(ancho: f32, alto: f32, semilla: u64) -> Clima {
        let mut rng = Rng::nuevo(semilla);
        let relieve = Ruido::nuevo(&mut rng);
        let humedad = Ruido::nuevo(&mut rng);
        let celda = 25.0;
        let cols = (ancho / celda).ceil() as usize;
        let filas = (alto / celda).ceil() as usize;
        let mut alt = vec![0.0f32; cols * filas];
        for f in 0..filas {
            for c in 0..cols {
                let (x, y) = (c as f32 / cols as f32, f as f32 / filas as f32);
                // Continente central con bordes que bajan al mar.
                let borde = ((x - 0.5).abs().max((y - 0.5).abs() * 0.8) * 2.0).powf(3.0);
                let h = relieve.fbm(x * 5.0, y * 4.0, 5) * 1.6 + 0.35 - borde * 1.4;
                alt[f * cols + c] = h * 2600.0;
            }
        }
        let mut celdas = Vec::with_capacity(cols * filas);
        for f in 0..filas {
            for c in 0..cols {
                let a = alt[f * cols + c];
                let lat = f as f32 / (filas - 1) as f32; // 0 = polo norte, 1 = trópico
                let t = -24.0 + 52.0 * lat.powf(0.85) - 6.5 * a.max(0.0) / 1000.0;
                // Lluvia: humedad oceánica (más cerca del mar y de los trópicos) + ruido,
                // y sombra orográfica: detrás de las montañas (al este) llueve menos.
                let (x, y) = (c as f32 / cols as f32, f as f32 / filas as f32);
                let ruido = humedad.fbm(x * 3.0 + 7.0, y * 3.0 + 3.0, 4);
                let oeste = (0..c.min(12)).map(|k| alt[f * cols + c - k - 1]).fold(0.0, f32::max);
                let sombra = (oeste - a).max(0.0) / 2000.0;
                let p = (900.0 + 1700.0 * ruido + 900.0 * (lat - 0.5) - 1400.0 * sombra).max(30.0);
                let bioma = if a < 0.0 { TipoBioma::Oceano } else { whittaker(t, p) };
                celdas.push(Celda { altitud: a, temperatura: t, lluvia: p, bioma });
            }
        }
        Clima { ancho, alto, cols, filas, celda, celdas, delta_t: 0.0, factor_lluvia: 1.0 }
    }

    pub fn idx(&self, pos: [f32; 2]) -> usize {
        let c = ((pos[0] / self.celda) as usize).min(self.cols - 1);
        let f = ((pos[1] / self.celda) as usize).min(self.filas - 1);
        f * self.cols + c
    }

    pub fn en(&self, pos: [f32; 2]) -> &Celda {
        &self.celdas[self.idx(pos)]
    }

    pub fn temperatura(&self, pos: [f32; 2]) -> f32 {
        self.en(pos).temperatura + self.delta_t
    }

    pub fn lluvia(&self, pos: [f32; 2]) -> f32 {
        self.en(pos).lluvia * self.factor_lluvia
    }

    /// Bioma actual con el clima modificado (sube la temperatura → los biomas se desplazan).
    pub fn bioma(&self, pos: [f32; 2]) -> TipoBioma {
        let c = self.en(pos);
        if c.altitud < 0.0 { TipoBioma::Oceano } else { whittaker(c.temperatura + self.delta_t, c.lluvia * self.factor_lluvia) }
    }

    pub fn es_agua(&self, pos: [f32; 2]) -> bool {
        self.en(pos).altitud < 0.0
    }

    /// Altitud interpolada (para dibujar el terreno suave).
    pub fn altitud(&self, pos: [f32; 2]) -> f32 {
        let fx = (pos[0] / self.celda - 0.5).clamp(0.0, (self.cols - 1) as f32);
        let fy = (pos[1] / self.celda - 0.5).clamp(0.0, (self.filas - 1) as f32);
        let (c0, f0) = (fx.floor() as usize, fy.floor() as usize);
        let (c1, f1) = ((c0 + 1).min(self.cols - 1), (f0 + 1).min(self.filas - 1));
        let (tx, ty) = (fx - c0 as f32, fy - f0 as f32);
        let a = |c: usize, f: usize| self.celdas[f * self.cols + c].altitud;
        let s0 = a(c0, f0) + tx * (a(c1, f0) - a(c0, f0));
        let s1 = a(c0, f1) + tx * (a(c1, f1) - a(c0, f1));
        s0 + ty * (s1 - s0)
    }

    /// Fracción del área de tierra de cada bioma.
    pub fn areas(&self) -> Vec<(TipoBioma, f32)> {
        TipoBioma::TODOS
            .iter()
            .map(|&b| {
                let n = (0..self.celdas.len())
                    .filter(|&i| {
                        let c = &self.celdas[i];
                        let actual = if c.altitud < 0.0 { TipoBioma::Oceano } else { whittaker(c.temperatura + self.delta_t, c.lluvia * self.factor_lluvia) };
                        actual == b
                    })
                    .count();
                (b, n as f32 / self.celdas.len() as f32)
            })
            .filter(|x| x.1 > 0.0)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whittaker_clasifica_bien() {
        assert_eq!(whittaker(27.0, 2500.0), TipoBioma::SelvaTropical);
        assert_eq!(whittaker(25.0, 100.0), TipoBioma::Desierto);
        assert_eq!(whittaker(26.0, 900.0), TipoBioma::Sabana);
        assert_eq!(whittaker(10.0, 900.0), TipoBioma::BosqueTemplado);
        assert_eq!(whittaker(10.0, 400.0), TipoBioma::Pradera);
        assert_eq!(whittaker(0.0, 500.0), TipoBioma::Taiga);
        assert_eq!(whittaker(-6.0, 300.0), TipoBioma::Tundra);
    }

    #[test]
    fn el_mundo_tiene_variedad_de_biomas() {
        let c = Clima::generar(3200.0, 2200.0, 7);
        let tipos = c.areas().len();
        assert!(tipos >= 6, "biomas: {:?}", c.areas().iter().map(|x| x.0.nombre()).collect::<Vec<_>>());
        assert!(c.areas().iter().any(|x| x.0 == TipoBioma::Oceano));
    }

    #[test]
    fn la_montania_es_mas_fria() {
        let c = Clima::generar(3200.0, 2200.0, 7);
        // A la misma latitud (misma fila), la celda más alta es la más fría.
        for f in 0..c.filas {
            let fila = &c.celdas[f * c.cols..(f + 1) * c.cols];
            let alta = fila.iter().max_by(|a, b| a.altitud.total_cmp(&b.altitud)).unwrap();
            let baja = fila.iter().filter(|x| x.altitud >= 0.0).min_by(|a, b| a.altitud.total_cmp(&b.altitud));
            if let Some(baja) = baja {
                if alta.altitud > baja.altitud + 500.0 {
                    assert!(alta.temperatura < baja.temperatura);
                }
            }
        }
    }

    #[test]
    fn calentar_desplaza_los_biomas() {
        let mut c = Clima::generar(3200.0, 2200.0, 7);
        let hielo = |c: &Clima| c.areas().iter().filter(|x| matches!(x.0, TipoBioma::HieloPolar | TipoBioma::Tundra)).map(|x| x.1).sum::<f32>();
        let antes = hielo(&c);
        c.delta_t = 4.0;
        assert!(hielo(&c) < antes, "con +4 °C hay menos tundra y hielo");
    }
}
