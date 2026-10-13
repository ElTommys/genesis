//! Escala 5: la mente. Una red neuronal diminuta —escrita a mano, sin dependencias—
//! le da a cada criatura un cerebro: siente hacia dónde está la comida y decide hacia
//! dónde moverse. Nadie programa la conducta de buscar comida: emerge por selección
//! natural sobre los pesos de la red (neuroevolución).
//!
//! El genoma de la mente ya no son bases (eso es la escala 4), son los pesos sinápticos.
//! Un mundo evalúa a cada individuo, los mejores se reproducen con cruce y mutación, y
//! generación tras generación la población caza mejor. El aprendizaje es medible: la
//! aptitud media sube con las generaciones (ver los tests).

use crate::azar::Rng;

/// Topología fija: 3 sentidos → capa oculta (tanh) → 2 músculos.
pub const ENTRADAS: usize = 3; // dx, dy hacia la comida, distancia
pub const OCULTAS: usize = 6;
pub const SALIDAS: usize = 2; // velocidad en x, y
pub const PESOS: usize = (ENTRADAS + 1) * OCULTAS + (OCULTAS + 1) * SALIDAS;

/// Red feedforward. El cerebro es exactamente `PESOS` números reales.
#[derive(Debug, Clone)]
pub struct Mente {
    pub pesos: Vec<f64>,
}

impl Mente {
    /// Cerebro al azar: pesos ~ N(0, 1).
    pub fn azar(rng: &mut Rng) -> Mente {
        Mente { pesos: (0..PESOS).map(|_| rng.normal()).collect() }
    }

    /// Siente el entorno y decide. Devuelve (mov_x, mov_y) en [-1, 1].
    pub fn pensar(&self, entradas: &[f64; ENTRADAS]) -> [f64; SALIDAS] {
        let w = &self.pesos;
        let mut k = 0;
        let mut oculta = [0.0f64; OCULTAS];
        for h in oculta.iter_mut() {
            let mut suma = w[k]; // sesgo
            k += 1;
            for &e in entradas {
                suma += w[k] * e;
                k += 1;
            }
            *h = suma.tanh();
        }
        let mut salida = [0.0f64; SALIDAS];
        for s in salida.iter_mut() {
            let mut suma = w[k]; // sesgo
            k += 1;
            for &h in &oculta {
                suma += w[k] * h;
                k += 1;
            }
            *s = suma.tanh();
        }
        salida
    }

    /// Reproducción: cada peso viene de un padre o del otro, con mutación gaussiana.
    pub fn cruzar(a: &Mente, b: &Mente, tasa: f64, fuerza: f64, rng: &mut Rng) -> Mente {
        let pesos = (0..PESOS)
            .map(|i| {
                let base = if rng.unif() < 0.5 { a.pesos[i] } else { b.pesos[i] };
                if rng.unif() < tasa { base + fuerza * rng.normal() } else { base }
            })
            .collect();
        Mente { pesos }
    }
}

/// El mundo donde se prueba una mente: una criatura busca comida en un cuadrado [0,1]².
/// Cada vez que la alcanza, come (+1 de aptitud) y la comida reaparece en otro sitio.
/// La aptitud es cuánta comida reúne en `pasos` instantes. Determinista con la semilla.
pub fn aptitud(mente: &Mente, pasos: u32, semilla: u64) -> f64 {
    let mut rng = Rng::nuevo(semilla);
    let (mut x, mut y) = (0.5, 0.5);
    let (mut cx, mut cy) = (rng.unif(), rng.unif());
    let paso = 0.02;
    let radio = 0.05;
    let mut comida = 0.0;
    for _ in 0..pasos {
        let dx = cx - x;
        let dy = cy - y;
        let dist = (dx * dx + dy * dy).sqrt();
        let eps = 1e-6;
        let entradas = [dx / (dist + eps), dy / (dist + eps), dist];
        let m = mente.pensar(&entradas);
        x = (x + m[0] * paso).clamp(0.0, 1.0);
        y = (y + m[1] * paso).clamp(0.0, 1.0);
        if ((cx - x).powi(2) + (cy - y).powi(2)).sqrt() < radio {
            comida += 1.0;
            cx = rng.unif();
            cy = rng.unif();
        }
    }
    comida
}

#[derive(Debug, Clone, Copy)]
pub struct Generacion {
    pub n: u32,
    pub mejor: f64,
    pub media: f64,
}

/// Neuroevolución: una población de cerebros evoluciona para cazar mejor.
/// Devuelve el historial por generación y la mejor mente encontrada.
pub struct Evolucion {
    poblacion: Vec<Mente>,
    pasos: u32,
    tasa_mut: f64,
    fuerza_mut: f64,
    rng: Rng,
}

impl Evolucion {
    pub fn nueva(tam: usize, pasos: u32, semilla: u64) -> Evolucion {
        let mut rng = Rng::nuevo(semilla);
        let poblacion = (0..tam.max(4)).map(|_| Mente::azar(&mut rng)).collect();
        Evolucion { poblacion, pasos, tasa_mut: 0.15, fuerza_mut: 0.4, rng }
    }

    /// Avanza una generación: evalúa, ordena, y la mitad superior engendra a la inferior.
    /// Todos los individuos se evalúan en el MISMO mundo (misma semilla de tarea) para que
    /// la comparación sea justa; la semilla cambia por generación para que no sobreajusten.
    pub fn generacion(&mut self, n: u32) -> (Generacion, Mente) {
        let tarea = self.rng.next();
        let mut puntuados: Vec<(f64, usize)> = self
            .poblacion
            .iter()
            .enumerate()
            .map(|(i, m)| (aptitud(m, self.pasos, tarea), i))
            .collect();
        puntuados.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        let media = puntuados.iter().map(|p| p.0).sum::<f64>() / puntuados.len() as f64;
        let mejor = puntuados[0].0;
        let campeon = self.poblacion[puntuados[0].1].clone();

        let tam = self.poblacion.len();
        let elite = (tam / 2).max(2);
        let padres: Vec<Mente> = puntuados[..elite].iter().map(|p| self.poblacion[p.1].clone()).collect();

        let mut nueva = Vec::with_capacity(tam);
        nueva.push(campeon.clone()); // elitismo: el mejor pasa intacto
        while nueva.len() < tam {
            let a = &padres[self.rng.rango(padres.len())];
            let b = &padres[self.rng.rango(padres.len())];
            nueva.push(Mente::cruzar(a, b, self.tasa_mut, self.fuerza_mut, &mut self.rng));
        }
        self.poblacion = nueva;

        (Generacion { n, mejor, media }, campeon)
    }

    /// Corre `gens` generaciones. Devuelve (historial, mejor mente global).
    pub fn correr(&mut self, gens: u32) -> (Vec<Generacion>, Mente) {
        let mut historial = Vec::with_capacity(gens as usize);
        let mut campeon = self.poblacion[0].clone();
        let mut mejor_global = f64::MIN;
        for n in 0..gens {
            let (g, c) = self.generacion(n);
            if g.mejor > mejor_global {
                mejor_global = g.mejor;
                campeon = c;
            }
            historial.push(g);
        }
        (historial, campeon)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn red_respeta_rango() {
        let mut r = Rng::nuevo(5);
        let m = Mente::azar(&mut r);
        let s = m.pensar(&[0.3, -0.7, 0.5]);
        assert!(s.iter().all(|v| (-1.0..=1.0).contains(v)), "tanh debe acotar la salida");
    }

    #[test]
    fn tamano_del_cerebro() {
        let mut r = Rng::nuevo(1);
        assert_eq!(Mente::azar(&mut r).pesos.len(), PESOS);
    }

    #[test]
    fn la_poblacion_aprende() {
        // La aptitud media de las últimas generaciones debe superar claramente a la del inicio.
        let mut ev = Evolucion::nueva(60, 300, 2024);
        let (hist, _) = ev.correr(40);
        let inicio: f64 = hist[..5].iter().map(|g| g.media).sum::<f64>() / 5.0;
        let fin: f64 = hist[hist.len() - 5..].iter().map(|g| g.media).sum::<f64>() / 5.0;
        assert!(fin > inicio + 1.0, "no aprendió: inicio={inicio:.2} fin={fin:.2}");
    }

    #[test]
    fn evolucion_es_reproducible() {
        let (a, _) = Evolucion::nueva(40, 200, 99).correr(15);
        let (b, _) = Evolucion::nueva(40, 200, 99).correr(15);
        assert_eq!(a.last().unwrap().mejor, b.last().unwrap().mejor);
    }

    #[test]
    fn campeon_le_gana_al_azar() {
        let mut ev = Evolucion::nueva(60, 300, 7);
        let (_, campeon) = ev.correr(40);
        let mut r = Rng::nuevo(123);
        let azar = Mente::azar(&mut r);
        let (mut gc, mut ga) = (0.0, 0.0);
        for s in 0..20 {
            gc += aptitud(&campeon, 300, 1000 + s);
            ga += aptitud(&azar, 300, 1000 + s);
        }
        assert!(gc > ga, "campeón {gc} debería superar al azar {ga}");
    }
}
