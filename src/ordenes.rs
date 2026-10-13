//! Órdenes de la Fase 3: conocimiento, ADN, bioma y visores.

use crate::bioma::{Bioma, Dieta, ParamBioma};
use crate::conocimiento;
use crate::mundo::{Mundo, Parametros};
use crate::vida;

pub const AYUDA: &str = "
  genesis saber [tema]             Base de conocimiento (células, ADN, gemelos, cadena trófica...)
  genesis adn <secuencia>          Traduce ADN/ARN a proteína con el código genético
  genesis bioma [opciones]         Ecosistema vivo en la terminal
  genesis vivir [opciones]         Ecosistema vivo en ventana (visual)
  genesis ver   [opciones]         Química 3D en ventana (visual; acepta las opciones de simular)

OPCIONES DE bioma / vivir:
  --especies Pasto:500,Conejo:60,Zorro:8,Champiñón:30
  --luz 1.0          Luz solar
  --temp 20          Temperatura ambiente (°C)
  --mutacion 0.002   Probabilidad de mutación por base
  --gemelos 0.01     Probabilidad de gemelos idénticos por cigoto
  --nutrientes 4     Nutrientes iniciales del suelo
  --especiacion 0.12 Distancia genética para declarar especie nueva
  --ticks 5000       Duración (solo bioma)
  --semilla 7
";

pub fn ejecutar(orden: &str, args: &[String]) {
    match orden {
        "saber" => conocimiento::imprimir(args.first().map(String::as_str)),
        "adn" => adn(args),
        "bioma" => {
            let (b, ticks) = crear_bioma(args);
            correr_bioma(b, ticks);
        }
        "vivir" => crate::vista::vivir(crear_bioma(args).0),
        "ver" => crate::vista::ver(crear_mundo(args)),
        _ => unreachable!(),
    }
}

fn adn(args: &[String]) {
    let sec: String = args.concat();
    let prot = vida::traducir(&sec);
    if prot.is_empty() {
        println!("No encontré un codón de inicio (ATG / AUG).");
        return;
    }
    println!("Proteína: {prot}  ({} aminoácidos)", prot.len());
    for c in prot.chars() {
        let nombre = vida::AMINOACIDOS.iter().find(|(l, _)| *l == c).map_or("?", |(_, n)| n);
        print!("{nombre} · ");
    }
    println!();
}

fn pares(args: &[String]) -> Vec<(String, String)> {
    args.chunks(2).filter(|c| c.len() == 2).map(|c| (c[0].clone(), c[1].clone())).collect()
}

fn num(flag: &str, v: &str) -> f64 {
    v.parse().unwrap_or_else(|_| panic!("valor inválido para {flag}: {v}"))
}

pub fn crear_bioma(args: &[String]) -> (Bioma, u64) {
    let mut p = ParamBioma::default();
    let mut especies = "Pasto:500,Trébol:150,Conejo:60,Venado:15,Jabalí:10,Zorro:8,Lobo:4,Champiñón:30".to_string();
    let mut ticks = 5000;
    for (f, v) in pares(args) {
        match f.as_str() {
            "--especies" => especies = v,
            "--luz" => p.luz = num(&f, &v) as f32,
            "--temp" => p.temperatura = num(&f, &v) as f32,
            "--mutacion" => p.tasa_mutacion = num(&f, &v),
            "--gemelos" => p.prob_monocigoticos = num(&f, &v),
            "--nutrientes" => p.nutrientes_iniciales = num(&f, &v) as f32,
            "--especiacion" => p.umbral_especiacion = num(&f, &v),
            "--ticks" => ticks = num(&f, &v) as u64,
            "--semilla" => p.semilla = num(&f, &v) as u64,
            otro => panic!("opción desconocida: {otro}"),
        }
    }
    let mut b = Bioma::nuevo(p);
    for item in especies.split(',') {
        let (n, c) = item.split_once(':').expect("formato: Pasto:500,Conejo:60");
        let pl = conocimiento::plantilla(n).unwrap_or_else(|| panic!("especie desconocida: {n} (mira `genesis saber`)"));
        b.sembrar(pl, c.parse().expect("cantidad inválida"));
    }
    (b, ticks)
}

fn correr_bioma(mut b: Bioma, ticks: u64) {
    let cada = (ticks / 10).max(1);
    println!("{:>6}  {:>6} {:>6} {:>6} {:>6} {:>6}  {:>7} {:>5}", "tick", "prod", "herb", "omni", "carn", "desc", "especies", "gen");
    let mut hecho = 0;
    while hecho < ticks {
        let n = cada.min(ticks - hecho);
        b.avanzar(n);
        hecho += n;
        let v = b.por_nivel();
        println!("{:>6}  {:>6} {:>6} {:>6} {:>6} {:>6}  {:>7} {:>5}", b.tick, v[0], v[1], v[2], v[3], v[4], b.especies_vivas(), b.est.generacion_max);
    }
    let e = &b.est;
    println!("\nNacimientos {} · clones {} · gemelos idénticos {} · partos múltiples (mellizos) {}", e.nacimientos, e.clones, e.gemelos, e.mellizos);
    println!("Muertes: vejez {} · hambre {} · depredación {}", e.muertes_vejez, e.muertes_hambre, e.depredaciones);
    println!("Mutaciones: {} sustituciones · {} inserciones · {} deleciones · {} duplicaciones",
        e.mutaciones.sustituciones, e.mutaciones.inserciones, e.mutaciones.deleciones, e.mutaciones.duplicaciones);
    println!("Especies nuevas {} · extinciones {}", e.especiaciones, e.extinciones);
    let m = b.biomasa_por_nivel();
    println!("\nPirámide de biomasa:");
    let max = m.iter().cloned().fold(1.0, f32::max);
    for (i, d) in Dieta::TODAS.iter().enumerate() {
        println!("  {:<16} {:<40} {:.0}", d.nombre(), "█".repeat((m[i] / max * 40.0).ceil() as usize), m[i]);
    }
    println!("\nEspecies vivas:");
    for s in b.especies.iter().filter(|s| s.poblacion > 0) {
        println!("  {:<26} {:<34} {:>5}", s.nombre, s.cientifico, s.poblacion);
    }
}

pub fn crear_mundo(args: &[String]) -> Mundo {
    let mut p = Parametros::default();
    let mut siembra = "H:300,O:150,C:30,N:20".to_string();
    p.caja = 16.0;
    for (f, v) in pares(args) {
        match f.as_str() {
            "--atomos" => siembra = v,
            "--caja" => p.caja = num(&f, &v),
            "--temp" => p.temperatura = num(&f, &v),
            "--energia" => p.energia_enlace = num(&f, &v),
            "--reaccion" => p.prob_reaccion = num(&f, &v),
            "--semilla" => p.semilla = num(&f, &v) as u64,
            otro => panic!("opción desconocida: {otro}"),
        }
    }
    let mut m = Mundo::nuevo(p);
    for item in siembra.split(',') {
        let (sim, n) = item.split_once(':').expect("formato: H:200,O:100");
        let (sim, extra) = match (sim.strip_suffix('+'), sim.strip_suffix('-')) {
            (Some(s), _) => (s, -1),
            (_, Some(s)) => (s, 1),
            _ => (sim, 0),
        };
        let z = crate::elementos::z_de_simbolo(sim).unwrap_or_else(|| panic!("elemento desconocido: {sim}"));
        m.sembrar(z, n.parse().expect("cantidad inválida"), extra);
    }
    m
}
