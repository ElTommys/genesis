//! Escala 5-7: organismos, poblaciones y ecosistema.
//!
//! Cada organismo tiene un genoma de ADN del que se leen sus rasgos. Nadie le dice
//! qué hacer: busca alimento según su dieta, gasta energía (ley de Kleiber),
//! se reproduce, muta, muere y los hongos devuelven sus nutrientes al suelo.
//! Las especies nuevas aparecen solas cuando la distancia genética crece demasiado.

use crate::azar::Rng;
use crate::conocimiento::{Plantilla, Reino};
use crate::vida::{Genoma, Mutaciones};

#[derive(Debug, Clone)]
pub struct ParamBioma {
    pub ancho: f32,
    pub alto: f32,
    /// Intensidad de luz solar (1.0 ≈ día soleado templado).
    pub luz: f32,
    /// Temperatura ambiente en °C.
    pub temperatura: f32,
    /// Probabilidad de mutación por base copiada.
    pub tasa_mutacion: f64,
    /// Probabilidad de que un cigoto se divida en gemelos idénticos (humanos ≈ 0.004).
    pub prob_monocigoticos: f64,
    pub nutrientes_iniciales: f32,
    pub umbral_especiacion: f64,
    pub max_organismos: usize,
    pub semilla: u64,
}

impl Default for ParamBioma {
    fn default() -> Self {
        ParamBioma {
            ancho: 900.0,
            alto: 600.0,
            luz: 1.0,
            temperatura: 20.0,
            tasa_mutacion: 0.002,
            prob_monocigoticos: 0.01,
            nutrientes_iniciales: 4.0,
            umbral_especiacion: 0.12,
            max_organismos: 12000,
            semilla: 7,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dieta {
    Productor,
    Herbivoro,
    Omnivoro,
    Carnivoro,
    Descomponedor,
}

impl Dieta {
    pub fn nombre(self) -> &'static str {
        match self {
            Dieta::Productor => "productores",
            Dieta::Herbivoro => "herbívoros",
            Dieta::Omnivoro => "omnívoros",
            Dieta::Carnivoro => "carnívoros",
            Dieta::Descomponedor => "descomponedores",
        }
    }
    pub const TODAS: [Dieta; 5] = [Dieta::Productor, Dieta::Herbivoro, Dieta::Omnivoro, Dieta::Carnivoro, Dieta::Descomponedor];
}

#[derive(Debug, Clone)]
pub struct Organismo {
    pub id: u64,
    pub reino: Reino,
    pub especie: usize,
    pub genoma: Genoma,
    pub pos: [f32; 2],
    pub dir: f32,
    pub energia: f32,
    pub edad: u32,
    pub generacion: u32,
    pub vivo: bool,
    pub gemelo_de: Option<u64>,
    /// Ticks que faltan para poder volver a reproducirse (gestación + crianza).
    pub descanso: u32,
    pub tam: f32,
    pub vel: f32,
    pub vision: f32,
    pub dieta_v: f32,
    pub fert: f32,
    pub longevidad: u32,
}

impl Organismo {
    fn nuevo(id: u64, reino: Reino, especie: usize, genoma: Genoma, pos: [f32; 2], energia: f32, generacion: u32) -> Self {
        let r = |i| genoma.rasgo(i) as f32;
        let movil = reino == Reino::Animalia;
        Organismo {
            id,
            reino,
            especie,
            pos,
            dir: 0.0,
            energia,
            edad: 0,
            generacion,
            vivo: true,
            gemelo_de: None,
            descanso: 0,
            tam: 1.5 + 6.0 * r(0),
            vel: if movil { 0.3 + 2.5 * r(1) } else { 0.0 },
            vision: 15.0 + 120.0 * r(2),
            dieta_v: r(3),
            fert: r(4),
            longevidad: 400 + (4000.0 * r(5)) as u32,
            genoma,
        }
    }

    pub fn dieta(&self) -> Dieta {
        match self.reino {
            Reino::Plantae => Dieta::Productor,
            Reino::Fungi => Dieta::Descomponedor,
            Reino::Animalia if self.dieta_v < 0.33 => Dieta::Herbivoro,
            Reino::Animalia if self.dieta_v < 0.66 => Dieta::Omnivoro,
            Reino::Animalia => Dieta::Carnivoro,
        }
    }

    /// Energía almacenada en el cuerpo (∝ masa ∝ tamaño²).
    pub fn cuerpo(&self) -> f32 {
        0.6 * self.tam * self.tam
    }
    fn umbral_reproduccion(&self) -> f32 {
        match self.reino {
            // Las plantas se propagan aunque las pastoreen (estolones, rizomas).
            Reino::Plantae => self.cuerpo() * (1.6 - 0.6 * self.fert),
            _ => self.cuerpo() * (3.0 - 1.5 * self.fert),
        }
    }
    fn maduro(&self) -> bool {
        self.edad as f32 > self.longevidad as f32 * 0.12
    }
}

#[derive(Debug, Clone)]
pub struct Especie {
    pub nombre: String,
    pub cientifico: String,
    pub reino: Reino,
    pub referencia: Genoma,
    pub poblacion: usize,
    pub maxima: usize,
    pub aparecio: u64,
    pub extinta: Option<u64>,
    pub ancestro: Option<usize>,
    /// Nicho climático (heredado de la plantilla; no evoluciona todavía).
    pub t_opt: f32,
    pub t_tol: f32,
    pub lluvia_min: f32,
    /// Plantilla original (para la inmigración desde biomas vecinos).
    pub plantilla: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Estadisticas {
    pub nacimientos: u64,
    pub muertes_vejez: u64,
    pub muertes_hambre: u64,
    pub depredaciones: u64,
    pub clones: u64,
    pub gemelos: u64,
    pub mellizos: u64,
    pub mutaciones: Mutaciones,
    pub especiaciones: u64,
    pub extinciones: u64,
    pub migraciones: u64,
    pub generacion_max: u32,
}

#[derive(Debug, Clone)]
pub struct Cadaver {
    pub pos: [f32; 2],
    pub energia: f32,
}

pub struct Bioma {
    pub p: ParamBioma,
    pub orgs: Vec<Organismo>,
    pub cadaveres: Vec<Cadaver>,
    pub especies: Vec<Especie>,
    pub suelo: Vec<f32>,
    pub cols: usize,
    pub filas: usize,
    pub est: Estadisticas,
    pub tick: u64,
    /// Historial de población por nivel trófico (cada 10 ticks).
    pub historial: Vec<[usize; 5]>,
    /// Mundo con relieve, clima y biomas (None = un solo clima uniforme).
    pub clima: Option<crate::clima::Clima>,
    rng: Rng,
    sig_id: u64,
    rejilla: Vec<Vec<usize>>,
    r_cols: usize,
    r_filas: usize,
}

pub const CELDA_SUELO: f32 = 20.0;
const CELDA_REJILLA: f32 = 40.0;

impl Bioma {
    pub fn nuevo(p: ParamBioma) -> Self {
        let cols = (p.ancho / CELDA_SUELO).ceil() as usize;
        let filas = (p.alto / CELDA_SUELO).ceil() as usize;
        let r_cols = (p.ancho / CELDA_REJILLA).ceil() as usize;
        let r_filas = (p.alto / CELDA_REJILLA).ceil() as usize;
        Bioma {
            suelo: vec![p.nutrientes_iniciales; cols * filas],
            rng: Rng::nuevo(p.semilla),
            p,
            orgs: Vec::new(),
            cadaveres: Vec::new(),
            especies: Vec::new(),
            cols,
            filas,
            est: Estadisticas::default(),
            tick: 0,
            historial: Vec::new(),
            clima: None,
            sig_id: 1,
            rejilla: vec![Vec::new(); r_cols * r_filas],
            r_cols,
            r_filas,
        }
    }

    /// La biosfera completa: continente con relieve, clima y biomas, y cada especie
    /// de la base de conocimiento sembrada donde su clima le sirve.
    pub fn mundo(semilla: u64) -> Bioma {
        let p = ParamBioma { ancho: 3200.0, alto: 2200.0, max_organismos: 16000, semilla, ..Default::default() };
        let mut b = Bioma::nuevo(p);
        b.clima = Some(crate::clima::Clima::generar(3200.0, 2200.0, semilla));
        for pl in crate::conocimiento::PLANTILLAS {
            let n = match pl.reino {
                Reino::Plantae if pl.rasgos[0] > 0.5 => 260,
                Reino::Plantae => 700,
                Reino::Animalia if pl.rasgos[3] > 0.66 => 14,
                Reino::Animalia => 60,
                Reino::Fungi => 250,
            };
            b.sembrar(pl, n);
        }
        b
    }

    pub fn sembrar(&mut self, pl: &Plantilla, cantidad: usize) {
        let mut ref_gen = Genoma::sintetizar(&pl.rasgos, &mut self.rng);
        ref_gen.0.shrink_to_fit();
        let especie = self.especies.len();
        self.especies.push(Especie {
            nombre: pl.nombre.to_string(),
            cientifico: pl.cientifico.to_string(),
            reino: pl.reino,
            referencia: ref_gen.clone(),
            poblacion: 0,
            maxima: 0,
            aparecio: self.tick,
            extinta: None,
            ancestro: None,
            t_opt: pl.t_opt,
            t_tol: pl.t_tol,
            lluvia_min: pl.lluvia_min,
            plantilla: Some(pl.nombre),
        });
        self.repoblar(especie, cantidad);
        self.contar_especies();
    }

    /// Agrega `cantidad` individuos de la especie en lugares donde su clima le sirve.
    pub fn repoblar(&mut self, especie: usize, cantidad: usize) {
        let ref_gen = self.especies[especie].referencia.clone();
        let reino = self.especies[especie].reino;
        for _ in 0..cantidad {
            let mut g = ref_gen.clone();
            let m = g.mutar(self.p.tasa_mutacion * 5.0, &mut self.rng);
            self.est.mutaciones.sumar(&m);
            let mut pos = [self.rng.unif() as f32 * self.p.ancho, self.rng.unif() as f32 * self.p.alto];
            if self.clima.is_some() {
                for _ in 0..300 {
                    if self.idoneidad(especie, pos) > 0.55 {
                        break;
                    }
                    pos = [self.rng.unif() as f32 * self.p.ancho, self.rng.unif() as f32 * self.p.alto];
                }
                if self.idoneidad(especie, pos) <= 0.3 {
                    continue;
                }
            }
            let id = self.id();
            let mut o = Organismo::nuevo(id, reino, especie, g, pos, 0.0, 0);
            o.energia = o.cuerpo() * 1.2;
            o.edad = self.rng.rango(o.longevidad as usize / 3) as u32;
            o.dir = (self.rng.unif() * std::f64::consts::TAU) as f32;
            self.orgs.push(o);
        }
        self.contar_especies();
    }

    pub fn temperatura_en(&self, pos: [f32; 2]) -> f32 {
        self.clima.as_ref().map_or(self.p.temperatura, |c| c.temperatura(pos))
    }

    pub fn lluvia_en(&self, pos: [f32; 2]) -> f32 {
        self.clima.as_ref().map_or(1000.0, |c| c.lluvia(pos))
    }

    /// Qué tan bien le sirve el clima de `pos` a la especie (0..1): campana de Gauss
    /// alrededor de su temperatura óptima × suficiencia de lluvia. Agua = 0.
    pub fn idoneidad(&self, especie: usize, pos: [f32; 2]) -> f32 {
        let Some(c) = &self.clima else { return 1.0 };
        if c.es_agua(pos) {
            return 0.0;
        }
        let e = &self.especies[especie];
        let t = c.temperatura(pos);
        let g = (-((t - e.t_opt) / e.t_tol).powi(2)).exp();
        let l = if e.lluvia_min > 0.0 { (c.lluvia(pos) / e.lluvia_min).min(1.0) } else { 1.0 };
        g * l
    }

    fn id(&mut self) -> u64 {
        self.sig_id += 1;
        self.sig_id
    }

    fn envolver(&self, mut p: [f32; 2]) -> [f32; 2] {
        p[0] = p[0].rem_euclid(self.p.ancho);
        p[1] = p[1].rem_euclid(self.p.alto);
        p
    }

    fn delta(&self, a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
        let mut dx = b[0] - a[0];
        let mut dy = b[1] - a[1];
        dx -= self.p.ancho * (dx / self.p.ancho).round();
        dy -= self.p.alto * (dy / self.p.alto).round();
        [dx, dy]
    }

    fn celda_suelo(&self, p: [f32; 2]) -> usize {
        let c = ((p[0] / CELDA_SUELO) as usize).min(self.cols - 1);
        let f = ((p[1] / CELDA_SUELO) as usize).min(self.filas - 1);
        f * self.cols + c
    }

    fn reconstruir_rejilla(&mut self) {
        for c in &mut self.rejilla {
            c.clear();
        }
        for (i, o) in self.orgs.iter().enumerate() {
            let c = ((o.pos[0] / CELDA_REJILLA) as usize).min(self.r_cols - 1);
            let f = ((o.pos[1] / CELDA_REJILLA) as usize).min(self.r_filas - 1);
            self.rejilla[f * self.r_cols + c].push(i);
        }
    }

    /// Índices de organismos vivos dentro de `radio` de `pos`.
    fn cercanos(&self, pos: [f32; 2], radio: f32, salida: &mut Vec<usize>) {
        salida.clear();
        let alcance = (radio / CELDA_REJILLA).ceil() as isize;
        let c0 = (pos[0] / CELDA_REJILLA) as isize;
        let f0 = (pos[1] / CELDA_REJILLA) as isize;
        let (rc, rf) = (self.r_cols as isize, self.r_filas as isize);
        // Limita el alcance para no visitar la misma celda dos veces en un mundo pequeño.
        let af = alcance.min((rf - 1) / 2);
        let ac = alcance.min((rc - 1) / 2);
        for df in -af..=af {
            for dc in -ac..=ac {
                let f = (f0 + df).rem_euclid(rf) as usize;
                let c = (c0 + dc).rem_euclid(rc) as usize;
                for &j in &self.rejilla[f * self.r_cols + c] {
                    let o = &self.orgs[j];
                    if o.vivo {
                        let d = self.delta(pos, o.pos);
                        if d[0] * d[0] + d[1] * d[1] <= radio * radio {
                            salida.push(j);
                        }
                    }
                }
            }
        }
    }

    pub fn avanzar(&mut self, ticks: u64) {
        for _ in 0..ticks {
            self.paso();
        }
    }

    fn paso(&mut self) {
        self.reconstruir_rejilla();
        let mut vecinos = Vec::new();
        let mut nuevos: Vec<Organismo> = Vec::new();
        // Factor térmico: regla Q10 para plantas y hongos, costo de termorregulación para animales.
        let t = self.p.temperatura;
        let q10 = 2f32.powf((t - 20.0) / 10.0);
        let termo = 1.0 + 0.03 * (t - 20.0).abs();
        let n = self.orgs.len();
        for i in 0..n {
            if !self.orgs[i].vivo {
                continue;
            }
            self.orgs[i].edad += 1;
            self.orgs[i].descanso = self.orgs[i].descanso.saturating_sub(1);
            match self.orgs[i].reino {
                Reino::Plantae => self.vivir_planta(i, q10, &mut vecinos),
                Reino::Fungi => self.vivir_hongo(i, q10),
                Reino::Animalia => self.vivir_animal(i, termo, &mut vecinos),
            }
            let o = &self.orgs[i];
            if o.vivo && (o.energia <= 0.0 || o.edad > o.longevidad) {
                if o.energia <= 0.0 { self.est.muertes_hambre += 1 } else { self.est.muertes_vejez += 1 }
                self.morir(i);
            }
            // El tope global solo frena a plantas y hongos (que llenan el espacio); los animales
            // se regulan por comida.
            let hay_lugar = self.orgs[i].reino == Reino::Animalia || self.orgs.len() + nuevos.len() < self.p.max_organismos;
            if self.orgs[i].vivo && hay_lugar {
                self.reproducir(i, &mut nuevos, &mut vecinos);
            }
        }
        self.orgs.retain(|o| o.vivo);
        self.orgs.extend(nuevos);
        // Los cadáveres se pudren lentamente (bacterias) y el suelo se meteoriza.
        for c in &mut self.cadaveres {
            let d = c.energia * 0.002;
            c.energia -= d;
            let k = ((c.pos[1] / CELDA_SUELO) as usize).min(self.filas - 1) * self.cols + ((c.pos[0] / CELDA_SUELO) as usize).min(self.cols - 1);
            self.suelo[k] += d;
        }
        self.cadaveres.retain(|c| c.energia > 0.05);
        for s in &mut self.suelo {
            *s = (*s + 0.001).min(30.0);
        }
        self.tick += 1;
        // Inmigración: si una especie desaparece del mapa, llegan unos pocos desde
        // fuera (metapoblaciones: los biomas reales no son islas cerradas).
        if self.clima.is_some() && self.tick % 500 == 0 {
            for e in 0..self.especies.len() {
                if self.especies[e].plantilla.is_some() && self.especies[e].poblacion == 0 && self.orgs.len() < self.p.max_organismos {
                    self.est.migraciones += 1;
                    self.repoblar(e, 6);
                }
            }
        }
        if self.tick % 10 == 0 {
            self.contar_especies();
            self.historial.push(self.por_nivel());
            if self.historial.len() > 2000 {
                self.historial.remove(0);
            }
        }
    }

    fn vivir_planta(&mut self, i: usize, q10: f32, vecinos: &mut Vec<usize>) {
        let pos = self.orgs[i].pos;
        let tam = self.orgs[i].tam;
        // Competencia por luz: la sombra de las vecinas reduce la fotosíntesis.
        self.cercanos(pos, 12.0, vecinos);
        let sombra = vecinos.iter().filter(|&&j| self.orgs[j].reino == Reino::Plantae).count().saturating_sub(1) as f32;
        let k = self.celda_suelo(pos);
        let nutr = self.suelo[k];
        let factor_clima = if self.clima.is_some() { self.idoneidad(self.orgs[i].especie, pos) * 1.6 } else { q10.min(2.0) };
        let ganancia = self.p.luz * 0.08 * tam * (nutr / (nutr + 1.0)) * factor_clima / (1.0 + 0.5 * sombra);
        let consumo = (ganancia * 0.25).min(self.suelo[k]);
        self.suelo[k] -= consumo;
        let o = &mut self.orgs[i];
        o.energia += ganancia - 0.004 * tam * q10;
        o.energia = o.energia.min(o.cuerpo() * 4.0);
    }

    fn vivir_hongo(&mut self, i: usize, q10: f32) {
        let pos = self.orgs[i].pos;
        let alcance = self.orgs[i].vision;
        let mut mejor: Option<(usize, f32)> = None;
        for (k, c) in self.cadaveres.iter().enumerate() {
            let d = self.delta(pos, c.pos);
            let d2 = d[0] * d[0] + d[1] * d[1];
            if d2 < alcance * alcance && mejor.is_none_or(|(_, m)| d2 < m) {
                mejor = Some((k, d2));
            }
        }
        let tam = self.orgs[i].tam;
        let mut gana = 0.0;
        if let Some((k, _)) = mejor {
            let bocado = self.cadaveres[k].energia.min(0.15 * tam * q10.min(2.0));
            self.cadaveres[k].energia -= bocado;
            gana = bocado * 0.15;
            let ks = self.celda_suelo(self.cadaveres[k].pos);
            self.suelo[ks] += bocado * 0.85;
        }
        let o = &mut self.orgs[i];
        o.energia += gana - 0.002 * tam * q10;
        o.energia = o.energia.min(o.cuerpo() * 2.0);
    }

    fn come(&self, cazador: &Organismo, presa: &Organismo) -> Option<f32> {
        if presa.id == cazador.id {
            return None;
        }
        // Una planta pelada no es alimento: hay que buscar otra con hojas.
        if presa.reino == Reino::Plantae && presa.energia < 0.8 * presa.cuerpo() {
            return None;
        }
        match (cazador.dieta(), presa.reino) {
            (Dieta::Herbivoro, Reino::Plantae) => Some(0.45),
            (Dieta::Omnivoro, Reino::Plantae) => Some(0.45),
            (Dieta::Omnivoro, Reino::Animalia) if presa.especie != cazador.especie && presa.tam < cazador.tam * 0.8 => Some(0.6),
            (Dieta::Carnivoro, Reino::Animalia) if presa.especie != cazador.especie && presa.tam < cazador.tam * 1.2 => Some(0.6),
            _ => None,
        }
    }

    fn vivir_animal(&mut self, i: usize, termo: f32, vecinos: &mut Vec<usize>) {
        let yo = self.orgs[i].clone();
        // Busca la comida más cercana que su dieta le permite.
        self.cercanos(yo.pos, yo.vision, vecinos);
        let mut objetivo: Option<(usize, f32, f32)> = None;
        // Saciedad: un animal lleno no caza (evita que los depredadores vacíen el mundo).
        let saciado = yo.energia > yo.umbral_reproduccion() * 1.1;
        for &j in vecinos.iter().filter(|_| !saciado) {
            if let Some(ef) = self.come(&yo, &self.orgs[j]) {
                let d = self.delta(yo.pos, self.orgs[j].pos);
                let d2 = d[0] * d[0] + d[1] * d[1];
                if objetivo.is_none_or(|(_, m, _)| d2 < m) {
                    objetivo = Some((j, d2, ef));
                }
            }
        }
        // Huye si hay un carnívoro que puede comérselo.
        let alarma = yo.vision * 0.35;
        let amenaza = vecinos.iter().copied().find(|&j| {
            let d = self.delta(yo.pos, self.orgs[j].pos);
            d[0] * d[0] + d[1] * d[1] < alarma * alarma && self.come(&self.orgs[j], &yo).is_some()
        });
        let mut dir = yo.dir;
        if let Some(a) = amenaza {
            let d = self.delta(yo.pos, self.orgs[a].pos);
            dir = (-d[1]).atan2(-d[0]);
        } else if let Some((j, _, _)) = objetivo {
            let d = self.delta(yo.pos, self.orgs[j].pos);
            dir = d[1].atan2(d[0]);
        } else {
            dir += (self.rng.unif() as f32 - 0.5) * 0.6;
        }
        let mut pos = self.envolver([yo.pos[0] + dir.cos() * yo.vel, yo.pos[1] + dir.sin() * yo.vel]);
        let mut termo = termo;
        if let Some(c) = &self.clima {
            if c.es_agua(pos) {
                // No sabe nadar: se da la vuelta en la orilla.
                pos = yo.pos;
                dir += std::f32::consts::PI * (0.7 + 0.6 * self.rng.unif() as f32);
            }
            // Fuera de su nicho climático gasta más energía en termorregular.
            termo = 1.0 + 1.5 * (1.0 - self.idoneidad(yo.especie, pos));
        }
        let costo = (0.002 * (yo.tam * yo.tam).powf(0.75) + 0.0006 * yo.vel * yo.vel * yo.tam) * termo;
        {
            let o = &mut self.orgs[i];
            o.pos = pos;
            o.dir = dir;
            o.energia -= costo;
        }
        if let Some((j, _, ef)) = objetivo {
            let d = self.delta(pos, self.orgs[j].pos);
            let contacto = yo.tam + self.orgs[j].tam;
            if d[0] * d[0] + d[1] * d[1] <= contacto * contacto {
                let presa = &mut self.orgs[j];
                let gana = if presa.reino == Reino::Plantae {
                    let bocado = (presa.energia - 0.5 * presa.cuerpo()).clamp(0.0, 0.12 * yo.tam);
                    presa.energia -= bocado; // la raíz queda: el pasto rebrota
                    bocado
                } else if self.rng.unif() as f32 > 0.25 * (yo.vel / presa.vel.max(0.1)).min(1.5) {
                    0.0 // la presa escapa: la mayoría de las cacerías fallan
                } else {
                    presa.vivo = false;
                    self.est.depredaciones += 1;
                    let total = presa.energia.max(0.0) + presa.cuerpo();
                    self.cadaveres.push(Cadaver { pos: presa.pos, energia: total * 0.3 });
                    total * 0.7
                };
                let o = &mut self.orgs[i];
                o.energia = (o.energia + gana * ef).min(o.cuerpo() * 4.0);
            }
        }
    }

    fn morir(&mut self, i: usize) {
        let o = &mut self.orgs[i];
        o.vivo = false;
        self.cadaveres.push(Cadaver { pos: o.pos, energia: o.energia.max(0.0) + o.cuerpo() });
    }

    fn reproducir(&mut self, i: usize, nuevos: &mut Vec<Organismo>, vecinos: &mut Vec<usize>) {
        let o = &self.orgs[i];
        if !o.maduro() || o.descanso > 0 || o.energia < o.umbral_reproduccion() {
            return;
        }
        let tasa = self.p.tasa_mutacion;
        match o.reino {
            // Plantas y hongos: clonación (estolones, rizomas, esporas) con errores de copia.
            Reino::Plantae | Reino::Fungi => {
                let radio = if o.reino == Reino::Fungi { 60.0 } else { 30.0 };
                let a = self.rng.unif() as f32 * std::f32::consts::TAU;
                let r = self.rng.unif() as f32 * radio;
                let pos = self.envolver([o.pos[0] + a.cos() * r, o.pos[1] + a.sin() * r]);
                // Sin espacio no hay donde germinar: el límite de las plantas es el terreno.
                let reino = o.reino;
                if self.clima.as_ref().is_some_and(|c| c.es_agua(pos)) {
                    return;
                }
                self.cercanos(pos, 9.0, vecinos);
                if vecinos.iter().filter(|&&j| self.orgs[j].reino == reino).count() >= 2 {
                    return;
                }
                let o = &self.orgs[i];
                let mut g = o.genoma.clone();
                let m = g.mutar(tasa, &mut self.rng);
                self.est.mutaciones.sumar(&m);
                self.est.clones += 1;
                let e = o.cuerpo() * 0.5;
                let (reino, esp, generacion) = (o.reino, o.especie, o.generacion + 1);
                self.orgs[i].energia -= e * 1.3;
                let hijo = self.nacer(reino, esp, g, pos, e, generacion);
                nuevos.push(hijo);
            }
            // Animales: reproducción sexual con pareja de la misma especie.
            Reino::Animalia => {
                let yo = o.clone();
                self.cercanos(yo.pos, yo.vision, vecinos);
                let pareja = vecinos.iter().copied().find(|&j| {
                    let p = &self.orgs[j];
                    j != i && p.especie == yo.especie && p.maduro() && p.energia > p.umbral_reproduccion() * 0.5
                });
                let Some(j) = pareja else { return };
                // Dependencia de la densidad: con demasiados congéneres cerca (territorio,
                // estrés, enfermedades) no hay cría. Es la capacidad de carga local.
                let congeneres = vecinos.iter().filter(|&&k| self.orgs[k].especie == yo.especie).count();
                if congeneres > 8 {
                    return;
                }
                let camada = 1 + (yo.fert * yo.fert * 4.0) as usize;
                if camada >= 2 {
                    self.est.mellizos += 1;
                }
                // La gestación cuesta: la madre queda con 40% del umbral; la mitad de lo invertido llega a las crías.
                let invertido = yo.energia - 0.4 * yo.umbral_reproduccion();
                let e_hijo = (invertido * 0.5 / camada as f32).max(0.25 * yo.cuerpo());
                let mut costo = 0.0;
                for _ in 0..camada {
                    let mut g = Genoma::cruzar(&yo.genoma, &self.orgs[j].genoma, &mut self.rng);
                    let m = g.mutar(tasa, &mut self.rng);
                    self.est.mutaciones.sumar(&m);
                    let copias = if self.rng.unif() < self.p.prob_monocigoticos { 2 } else { 1 };
                    if copias == 2 {
                        self.est.gemelos += 1;
                    }
                    let mut primero = None;
                    for _ in 0..copias {
                        let (dx, dy) = (self.rng.normal() as f32 * 3.0, self.rng.normal() as f32 * 3.0);
                        let pos = self.envolver([yo.pos[0] + dx, yo.pos[1] + dy]);
                        let mut h = self.nacer(Reino::Animalia, yo.especie, g.clone(), pos, e_hijo, yo.generacion.max(self.orgs[j].generacion) + 1);
                        h.dir = (self.rng.unif() * std::f64::consts::TAU) as f32;
                        h.gemelo_de = primero;
                        primero.get_or_insert(h.id);
                        nuevos.push(h);
                        costo += e_hijo;
                    }
                }
                let _ = costo;
                self.orgs[i].energia = 0.4 * yo.umbral_reproduccion();
                self.orgs[j].energia -= 0.1 * invertido;
                self.orgs[i].descanso = self.orgs[i].longevidad / 10;
                self.orgs[j].descanso = self.orgs[j].longevidad / 20;
            }
        }
    }

    fn nacer(&mut self, reino: Reino, especie: usize, g: Genoma, pos: [f32; 2], energia: f32, generacion: u32) -> Organismo {
        self.est.nacimientos += 1;
        self.est.generacion_max = self.est.generacion_max.max(generacion);
        let mut especie = especie;
        if g.distancia(&self.especies[especie].referencia) > self.p.umbral_especiacion {
            especie = self.especiar(especie, &g);
        }
        let id = self.id();
        Organismo::nuevo(id, reino, especie, g, pos, energia, generacion)
    }

    fn especiar(&mut self, madre: usize, g: &Genoma) -> usize {
        self.est.especiaciones += 1;
        let base = self.especies[madre].nombre.split(" (").next().unwrap().to_string();
        let n = self.especies.iter().filter(|e| e.nombre.starts_with(&base)).count();
        let nueva = Especie {
            nombre: format!("{base} (linaje {n})"),
            cientifico: format!("{} var. {n}", self.especies[madre].cientifico),
            reino: self.especies[madre].reino,
            referencia: g.clone(),
            poblacion: 0,
            maxima: 0,
            aparecio: self.tick,
            extinta: None,
            ancestro: Some(madre),
            t_opt: self.especies[madre].t_opt,
            t_tol: self.especies[madre].t_tol,
            lluvia_min: self.especies[madre].lluvia_min,
            plantilla: None,
        };
        self.especies.push(nueva);
        self.especies.len() - 1
    }

    fn contar_especies(&mut self) {
        for e in &mut self.especies {
            e.poblacion = 0;
        }
        for o in &self.orgs {
            self.especies[o.especie].poblacion += 1;
        }
        let tick = self.tick;
        for e in &mut self.especies {
            e.maxima = e.maxima.max(e.poblacion);
            if e.poblacion == 0 && e.maxima > 0 && e.extinta.is_none() {
                e.extinta = Some(tick);
                self.est.extinciones += 1;
            }
        }
    }

    /// Población por nivel trófico, en el orden de `Dieta::TODAS`.
    pub fn por_nivel(&self) -> [usize; 5] {
        let mut v = [0; 5];
        for o in &self.orgs {
            v[Dieta::TODAS.iter().position(|&d| d == o.dieta()).unwrap()] += 1;
        }
        v
    }

    /// Biomasa (energía corporal) por nivel trófico: la pirámide ecológica.
    pub fn biomasa_por_nivel(&self) -> [f32; 5] {
        let mut v = [0.0; 5];
        for o in &self.orgs {
            v[Dieta::TODAS.iter().position(|&d| d == o.dieta()).unwrap()] += o.cuerpo() + o.energia.max(0.0);
        }
        v
    }

    pub fn especies_vivas(&self) -> usize {
        self.especies.iter().filter(|e| e.poblacion > 0).count()
    }

    pub fn nutrientes_totales(&self) -> f32 {
        self.suelo.iter().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conocimiento::plantilla;

    fn bioma_base() -> Bioma {
        let mut b = Bioma::nuevo(ParamBioma::default());
        for (n, c) in [("Pasto", 500), ("Conejo", 60), ("Zorro", 8), ("Champiñón", 30)] {
            b.sembrar(plantilla(n).unwrap(), c);
        }
        b
    }

    #[test]
    fn el_ecosistema_sobrevive() {
        let mut b = bioma_base();
        b.avanzar(3000);
        let v = b.por_nivel();
        assert!(v[0] > 0 && v[1] > 0, "productores y herbívoros vivos: {v:?}");
    }

    #[test]
    fn hay_mutaciones_y_clones() {
        let mut b = bioma_base();
        b.avanzar(800);
        assert!(b.est.clones > 0);
        assert!(b.est.mutaciones.total() > 0);
        assert!(b.est.nacimientos > 0);
    }

    #[test]
    fn piramide_de_biomasa() {
        let mut b = bioma_base();
        b.avanzar(1500);
        let m = b.biomasa_por_nivel();
        assert!(m[0] > m[1], "productores > herbívoros: {m:?}");
    }

    #[test]
    fn sin_luz_mueren_las_plantas() {
        let mut b = Bioma::nuevo(ParamBioma { luz: 0.0, ..Default::default() });
        b.sembrar(plantilla("Pasto").unwrap(), 200);
        b.avanzar(3000);
        assert_eq!(b.por_nivel()[0], 0);
    }
}

#[cfg(test)]
mod diag_mundo {
    use super::*;
    #[test]
    #[ignore]
    fn diag_mundo() {
        let mut b = Bioma::mundo(7);
        for _ in 0..8 {
            let t0 = std::time::Instant::now();
            b.avanzar(500);
            let v = b.por_nivel();
            let vivas: Vec<String> = b.especies.iter().filter(|e| e.poblacion > 0 && e.plantilla.is_some()).map(|e| format!("{}:{}", e.nombre, e.poblacion)).collect();
            eprintln!("t{} {:?} {:.2}s mig {} | {}", b.tick, v, t0.elapsed().as_secs_f32(), b.est.migraciones, vivas.join(" "));
        }
    }
}

#[cfg(test)]
mod diagnostico {
    use super::*;
    use crate::conocimiento::plantilla;
    #[test]
    #[ignore]
    fn diag() {
        let mut b = Bioma::nuevo(ParamBioma::default());
        b.sembrar(plantilla("Pasto").unwrap(), 500);
        b.sembrar(plantilla("Conejo").unwrap(), 60);
        for _ in 0..10 {
            b.avanzar(50);
            let h: Vec<&Organismo> = b.orgs.iter().filter(|o| o.dieta() == Dieta::Herbivoro).collect();
            let p: Vec<&Organismo> = b.orgs.iter().filter(|o| o.dieta() == Dieta::Productor).collect();
            let em = h.iter().map(|o| o.energia).sum::<f32>() / h.len().max(1) as f32;
            let ep = p.iter().map(|o| o.energia / o.cuerpo()).sum::<f32>() / p.len().max(1) as f32;
            eprintln!("t{} herb {} E={:.2} tam={:.2} vel={:.2} | plantas {} E/cuerpo={:.2} | hambre {} vejez {}", b.tick, h.len(), em,
                h.first().map_or(0.0, |o| o.tam), h.first().map_or(0.0, |o| o.vel), p.len(), ep, b.est.muertes_hambre, b.est.muertes_vejez);
        }
    }
}
