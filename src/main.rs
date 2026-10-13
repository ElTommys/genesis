mod azar;
mod bioma;
mod conocimiento;
mod cuantica;
mod elementos;
mod mente;
mod mundo;
mod ordenes;
mod particula;
mod vida;
mod vista;

use cuantica::Atomo;
use mundo::{Mundo, Parametros};
use particula::Particula;
use std::time::Instant;

const AYUDA: &str = "\
universo — simulador por escalas (partícula → átomo → molécula → ...)

USO:
  universo particulas              Protón, neutrón y electrón
  universo tabla                   Tabla periódica deducida de la mecánica cuántica
  universo elemento <Z|símbolo>    Ficha de un átomo (ej: universo elemento Fe)
  universo simular [opciones]      Mundo 3D que evoluciona solo
  universo evolucionar [opciones]  Criaturas con cerebro que aprenden a cazar (neuroevolución)

OPCIONES DE simular (todas editables):
  --atomos H:200,O:100,Na+:10,Cl-:10   Qué sembrar (+ catión, - anión)
  --caja 20          Lado de la caja 3D
  --temp 1.0         Temperatura del baño térmico
  --pasos 20000      Pasos de tiempo
  --cada 2000        Cada cuántos pasos imprimir el censo
  --energia 8.0      Energía de enlace (más alta = enlaces más estables)
  --reaccion 0.5     Probabilidad de reaccionar al chocar
  --semilla 42       Misma semilla = mismo universo

OPCIONES DE evolucionar:
  --poblacion 80     Cuántas criaturas por generación
  --generaciones 60  Cuántas generaciones evolucionar
  --pasos 400        Instantes de vida en que se evalúa a cada criatura
  --semilla 2024     Misma semilla = misma evolución
";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("particulas") => particulas(),
        Some("tabla") => tabla(),
        Some("elemento") => match args.get(1) {
            Some(a) => elemento(a),
            None => eprintln!("Falta el elemento. Ej: universo elemento 8"),
        },
        Some("simular") => simular(&args[1..]),
        Some("evolucionar") => evolucionar(&args[1..]),
        Some(o @ ("saber" | "adn" | "bioma" | "ver" | "vivir")) => ordenes::ejecutar(o, &args[1..]),
        _ => {
            print!("{AYUDA}");
            print!("{}", ordenes::AYUDA);
        }
    }
}

fn particulas() {
    println!("{:<10} {:>6} {:>14} {:>5}", "partícula", "carga", "masa (u)", "spin");
    for p in [Particula::Proton, Particula::Neutron, Particula::Electron] {
        println!("{:<10} {:>+6} {:>14.7} {:>5}", p.nombre(), p.carga(), p.masa_u(), p.spin());
    }
}

fn tabla() {
    // Rejilla de 18 columnas + filas aparte para lantánidos (fila 8) y actínidos (fila 9).
    let mut rejilla = vec![vec!["".to_string(); 18]; 9];
    let mut col_f = [0usize; 2];
    for z in 1..=118 {
        let a = Atomo::nuevo(z);
        match a.grupo() {
            Some(g) => rejilla[(a.periodo() - 1) as usize][(g - 1) as usize] = a.simbolo().into(),
            None => {
                let fila = (a.periodo() - 6) as usize;
                rejilla[7 + fila][2 + col_f[fila]] = a.simbolo().into();
                col_f[fila] += 1;
            }
        }
    }
    for (i, fila) in rejilla.iter().enumerate() {
        if i == 7 {
            println!();
        }
        let linea: String = fila.iter().map(|s| format!("{s:<4}")).collect();
        println!("{}", linea.trim_end());
    }
    println!("\nPeriodos, grupos y bloques calculados con Aufbau + Pauli + Hund.");
}

fn elemento(arg: &str) {
    let z = arg.parse::<u32>().ok().or_else(|| elementos::z_de_simbolo(arg));
    let Some(z) = z.filter(|z| (1..=118).contains(z)) else {
        eprintln!("No reconozco '{arg}'");
        return;
    };
    let a = Atomo::nuevo(z);
    println!("{} ({}), Z = {}", a.nombre(), a.simbolo(), a.z);
    println!("  Núcleo:        {} protones (+{}) y {} neutrones  →  A = {}", a.z, a.z, a.neutrones, a.numero_masico());
    println!("  Electrones:    {} (−{})  →  carga neta 0", a.z, a.z);
    println!("  Masa:          {:.3} u (suma de partículas, sin energía de enlace nuclear)", a.masa_u());
    println!("  Configuración: {}", a.configuracion());
    println!("  Periodo {}  ·  Grupo {}  ·  Bloque {}", a.periodo(),
        a.grupo().map_or("f (lantánido/actínido)".into(), |g| g.to_string()), a.bloque());
    println!("  Valencia:      {} electrones, forma {} enlace(s)", a.electrones_valencia(), a.valencia_enlace());
    println!("  Spin total S = {}", a.spin_total());
    println!("\n  Subcapa  n  l  orbitales(mₗ)  e⁻  desapareados");
    for s in &a.config {
        println!("  {:<7}  {}  {}  {:>13}  {:>2}  {:>12}", s.etiqueta(), s.n, s.l, s.orbitales(), s.electrones, s.desapareados());
    }
}

fn simular(args: &[String]) {
    let mut p = Parametros::default();
    let mut siembra = "H:200,O:100".to_string();
    let mut pasos = 20_000u64;
    let mut cada = 2_000u64;
    let mut it = args.iter();
    while let Some(flag) = it.next() {
        let Some(v) = it.next() else { break };
        let num = || v.parse::<f64>().unwrap_or_else(|_| panic!("valor inválido para {flag}: {v}"));
        match flag.as_str() {
            "--atomos" => siembra = v.clone(),
            "--caja" => p.caja = num(),
            "--temp" => p.temperatura = num(),
            "--pasos" => pasos = num() as u64,
            "--cada" => cada = (num() as u64).max(1),
            "--energia" => p.energia_enlace = num(),
            "--reaccion" => p.prob_reaccion = num(),
            "--semilla" => p.semilla = num() as u64,
            otro => panic!("opción desconocida: {otro}"),
        }
    }

    let mut m = Mundo::nuevo(p);
    for item in siembra.split(',') {
        let (sim, n) = item.split_once(':').expect("formato: H:200,O:100");
        let (sim, extra) = match sim.strip_suffix('+') {
            Some(s) => (s, -1),
            None => match sim.strip_suffix('-') {
                Some(s) => (s, 1),
                None => (sim, 0),
            },
        };
        let z = elementos::z_de_simbolo(sim).unwrap_or_else(|| panic!("elemento desconocido: {sim}"));
        m.sembrar(z, n.parse().expect("cantidad inválida"), extra);
    }

    println!("Universo: {} átomos, caja {}³, T = {}\n", m.cuerpos.len(), m.p.caja, m.p.temperatura);
    let t0 = Instant::now();
    let mut hecho = 0;
    while hecho < pasos {
        let n = cada.min(pasos - hecho);
        m.avanzar(n);
        hecho += n;
        let mut censo: Vec<_> = m.censo().into_iter().collect();
        censo.sort_by(|a, b| b.1.cmp(&a.1));
        let top: Vec<String> = censo.iter().take(8).map(|(f, c)| format!("{f}×{c}")).collect();
        println!("paso {:>7}  T={:.2}  enlaces={:<5} {}", m.paso, m.temperatura(), m.num_enlaces(), top.join("  "));
    }
    let s = t0.elapsed().as_secs_f64();
    println!("\n{} pasos en {:.2}s ({:.0} átomo·pasos/s)", pasos, s, pasos as f64 * m.cuerpos.len() as f64 / s);
}

fn evolucionar(args: &[String]) {
    let mut poblacion = 80usize;
    let mut generaciones = 60u32;
    let mut pasos = 400u32;
    let mut semilla = 2024u64;
    let mut it = args.iter();
    while let Some(flag) = it.next() {
        let Some(v) = it.next() else { break };
        let num = || v.parse::<f64>().unwrap_or_else(|_| panic!("valor inválido para {flag}: {v}"));
        match flag.as_str() {
            "--poblacion" => poblacion = num() as usize,
            "--generaciones" => generaciones = num() as u32,
            "--pasos" => pasos = num() as u32,
            "--semilla" => semilla = num() as u64,
            otro => panic!("opción desconocida: {otro}"),
        }
    }

    println!(
        "Neuroevolución: {} criaturas · {} generaciones · cerebro de {} pesos\n",
        poblacion, generaciones, mente::PESOS
    );
    println!("{:>4}  {:>8}  {:>8}  {}", "gen", "mejor", "media", "progreso");
    let t0 = Instant::now();
    let mut ev = mente::Evolucion::nueva(poblacion, pasos, semilla);
    let mut primera = None;
    let mut ultima = 0.0;
    let mut mejor_global = f64::MIN;
    for n in 0..generaciones {
        let (g, _) = ev.generacion(n);
        primera.get_or_insert(g.media);
        ultima = g.media;
        mejor_global = mejor_global.max(g.mejor);
        if n % (generaciones / 20).max(1) == 0 || n == generaciones - 1 {
            let barra = "█".repeat((g.media as usize).min(40));
            println!("{:>4}  {:>8.1}  {:>8.2}  {}", g.n, g.mejor, g.media, barra);
        }
    }
    let s = t0.elapsed().as_secs_f64();
    let ini = primera.unwrap_or(0.0).max(0.01);
    println!(
        "\nLa población pasó de cazar {:.1} a {:.1} de media (×{:.1}). Mejor marca: {:.0} presas.",
        ini, ultima, ultima / ini, mejor_global
    );
    println!("{} generaciones en {:.2}s.", generaciones, s);
}
