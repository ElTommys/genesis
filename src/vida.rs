//! Escala 4: la vida. ADN real (A, C, G, T), el código genético universal,
//! mutaciones y reproducción (clonación y recombinación sexual).
//!
//! Cada organismo lleva un genoma de bases. Sus rasgos (tamaño, velocidad, dieta...)
//! se leen del contenido de G+C de cada gen, así que una mutación puntual cambia un
//! rasgo un poco, y una inserción o deleción desplaza todo lo que viene detrás
//! (corrimiento del marco de lectura).

use crate::azar::Rng;

pub const BASES: [u8; 4] = *b"ACGT";

/// Código genético estándar (orden de bases T, C, A, G). '*' = codón de parada.
const CODIGO: &[u8; 64] = b"FFLLSSSSYY**CC*WLLLLPPPPHHQQRRRRIIIMTTTTNNKKSSRRVVVVAAAADDEEGGGG";

pub const AMINOACIDOS: [(char, &str); 20] = [
    ('A', "Alanina"), ('R', "Arginina"), ('N', "Asparagina"), ('D', "Ácido aspártico"),
    ('C', "Cisteína"), ('Q', "Glutamina"), ('E', "Ácido glutámico"), ('G', "Glicina"),
    ('H', "Histidina"), ('I', "Isoleucina"), ('L', "Leucina"), ('K', "Lisina"),
    ('M', "Metionina"), ('F', "Fenilalanina"), ('P', "Prolina"), ('S', "Serina"),
    ('T', "Treonina"), ('W', "Triptófano"), ('Y', "Tirosina"), ('V', "Valina"),
];

fn indice_base(b: u8) -> Option<usize> {
    match b.to_ascii_uppercase() {
        b'T' | b'U' => Some(0),
        b'C' => Some(1),
        b'A' => Some(2),
        b'G' => Some(3),
        _ => None,
    }
}

pub fn codon_a_aminoacido(c: &[u8]) -> Option<char> {
    let i = indice_base(c[0])? * 16 + indice_base(c[1])? * 4 + indice_base(c[2])?;
    Some(CODIGO[i] as char)
}

/// Traduce ADN/ARN a proteína: busca el primer ATG (inicio) y lee hasta un codón de parada.
pub fn traducir(adn: &str) -> String {
    let b = adn.as_bytes();
    let Some(inicio) = (0..b.len().saturating_sub(2)).find(|&i| b[i..i + 3].eq_ignore_ascii_case(b"ATG") || b[i..i + 3].eq_ignore_ascii_case(b"AUG")) else {
        return String::new();
    };
    let mut prot = String::new();
    for c in b[inicio..].chunks_exact(3) {
        match codon_a_aminoacido(c) {
            Some('*') | None => break,
            Some(a) => prot.push(a),
        }
    }
    prot
}

pub const RASGOS: [&str; 6] = ["tamaño", "velocidad", "visión", "dieta", "fertilidad", "longevidad"];
pub const LARGO_GEN: usize = 30;

#[derive(Debug, Clone, Copy, Default)]
pub struct Mutaciones {
    pub sustituciones: u64,
    pub inserciones: u64,
    pub deleciones: u64,
    pub duplicaciones: u64,
}

impl Mutaciones {
    pub fn total(&self) -> u64 {
        self.sustituciones + self.inserciones + self.deleciones + self.duplicaciones
    }
    pub fn sumar(&mut self, o: &Mutaciones) {
        self.sustituciones += o.sustituciones;
        self.inserciones += o.inserciones;
        self.deleciones += o.deleciones;
        self.duplicaciones += o.duplicaciones;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Genoma(pub Vec<u8>);

impl Genoma {
    /// Fabrica un genoma cuyos rasgos valen `valores` (0..1), con bases al azar.
    pub fn sintetizar(valores: &[f64; 6], rng: &mut Rng) -> Genoma {
        let mut g = Vec::with_capacity(LARGO_GEN * 6);
        for &v in valores {
            let gc = (v.clamp(0.0, 1.0) * LARGO_GEN as f64).round() as usize;
            let mut gen_: Vec<u8> = (0..LARGO_GEN).map(|i| if i < gc { [b'C', b'G'][rng.rango(2)] } else { [b'A', b'T'][rng.rango(2)] }).collect();
            for i in (1..gen_.len()).rev() {
                gen_.swap(i, rng.rango(i + 1));
            }
            g.extend(gen_);
        }
        Genoma(g)
    }

    /// Valor del rasgo `i` = fracción de G+C en su gen (0..1).
    pub fn rasgo(&self, i: usize) -> f64 {
        let ini = (i * LARGO_GEN).min(self.0.len());
        let fin = ((i + 1) * LARGO_GEN).min(self.0.len());
        if fin <= ini {
            return 0.0;
        }
        self.0[ini..fin].iter().filter(|&&b| b == b'G' || b == b'C').count() as f64 / (fin - ini) as f64
    }

    /// Copia con errores: cada base tiene probabilidad `tasa` de mutar.
    pub fn mutar(&mut self, tasa: f64, rng: &mut Rng) -> Mutaciones {
        let mut m = Mutaciones::default();
        let mut i = 0;
        while i < self.0.len() {
            if rng.unif() < tasa {
                let tipo = rng.unif();
                if tipo < 0.90 {
                    self.0[i] = BASES[rng.rango(4)];
                    m.sustituciones += 1;
                } else if tipo < 0.95 && self.0.len() < LARGO_GEN * 8 {
                    self.0.insert(i, BASES[rng.rango(4)]);
                    m.inserciones += 1;
                    i += 1;
                } else if tipo < 0.99 && self.0.len() > LARGO_GEN * 5 {
                    self.0.remove(i);
                    m.deleciones += 1;
                    continue;
                } else if self.0.len() < LARGO_GEN * 8 {
                    let fin = (i + 3).min(self.0.len());
                    let trozo: Vec<u8> = self.0[i..fin].to_vec();
                    let n = trozo.len();
                    self.0.splice(i..i, trozo);
                    m.duplicaciones += 1;
                    i += n;
                }
            }
            i += 1;
        }
        m
    }

    /// Reproducción sexual: cada gen viene de la madre o del padre (segregación de Mendel).
    pub fn cruzar(madre: &Genoma, padre: &Genoma, rng: &mut Rng) -> Genoma {
        let n = madre.0.len().max(padre.0.len());
        let mut hijo = Vec::with_capacity(n);
        let mut ini = 0;
        while ini < n {
            let fin = (ini + LARGO_GEN).min(n);
            let fuente = if rng.unif() < 0.5 { madre } else { padre };
            let otra = if std::ptr::eq(fuente, madre) { padre } else { madre };
            let f = if fin <= fuente.0.len() { fuente } else { otra };
            if ini < f.0.len() {
                hijo.extend_from_slice(&f.0[ini..fin.min(f.0.len())]);
            }
            ini = fin;
        }
        Genoma(hijo)
    }

    /// Distancia genética: diferencia media de los rasgos que expresan los genes.
    /// (Comparar base a base exageraría: una sola inserción desplaza todo el resto.)
    pub fn distancia(&self, o: &Genoma) -> f64 {
        (0..RASGOS.len()).map(|i| (self.rasgo(i) - o.rasgo(i)).abs()).sum::<f64>() / RASGOS.len() as f64
    }

    pub fn texto(&self) -> String {
        String::from_utf8_lossy(&self.0).into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codigo_genetico() {
        assert_eq!(traducir("ATGGCCTGGTAA"), "MAW");
        assert_eq!(traducir("ccATGTTTAAATGA"), "MFK");
        assert_eq!(traducir("AUGGGUUAG"), "MG"); // ARN
    }

    #[test]
    fn rasgos_sintetizados() {
        let mut r = Rng::nuevo(7);
        let g = Genoma::sintetizar(&[0.2, 0.5, 0.9, 0.0, 1.0, 0.3], &mut r);
        assert!((g.rasgo(2) - 0.9).abs() < 0.04);
        assert_eq!(g.rasgo(3), 0.0);
        assert_eq!(g.rasgo(4), 1.0);
    }

    #[test]
    fn clon_sin_mutacion_es_identico() {
        let mut r = Rng::nuevo(1);
        let g = Genoma::sintetizar(&[0.5; 6], &mut r);
        let mut c = g.clone();
        c.mutar(0.0, &mut r);
        assert_eq!(g.distancia(&c), 0.0);
        c.mutar(0.05, &mut r);
        assert!(g.distancia(&c) > 0.0);
    }

    #[test]
    fn hijo_hereda_de_ambos() {
        let mut r = Rng::nuevo(3);
        let a = Genoma::sintetizar(&[0.0; 6], &mut r);
        let b = Genoma::sintetizar(&[1.0; 6], &mut r);
        let h = Genoma::cruzar(&a, &b, &mut r);
        let de_b = (0..6).filter(|&i| h.rasgo(i) == 1.0).count();
        assert!(de_b > 0 && de_b < 6);
    }
}
