//! Física del átomo con fórmulas reales: constantes CODATA 2018, funciones de onda
//! hidrogenoides, carga efectiva de Slater, niveles de energía, fotones, núcleo
//! (radio y energía de enlace de Weizsäcker), E = mc² y moles.

use std::f64::consts::PI;

// ───────────── Constantes (CODATA 2018) ─────────────
pub const C: f64 = 299_792_458.0; // m/s
pub const H: f64 = 6.626_070_15e-34; // J·s
pub const E_CARGA: f64 = 1.602_176_634e-19; // C
pub const NA: f64 = 6.022_140_76e23; // 1/mol
pub const A0: f64 = 5.291_772_109e-11; // radio de Bohr, m
pub const RYDBERG_EV: f64 = 13.605_693_122; // eV
pub const U_MEV: f64 = 931.494_102; // 1 u = 931.494 MeV/c²
pub const M_PROTON_U: f64 = 1.007_276_466;
pub const M_NEUTRON_U: f64 = 1.008_664_916;
pub const M_ELECTRON_U: f64 = 0.000_548_580;
pub const HC_EV_NM: f64 = 1_239.841_984; // h·c en eV·nm

/// Tamaños típicos (m) de cada escala.
pub const ESCALAS: [(&str, f64); 5] = [
    ("átomo (nube electrónica)", 1e-10),
    ("núcleo", 1e-14),
    ("protón / neutrón", 1.7e-15),
    ("quark", 1e-18),
    ("electrón (cota superior)", 1e-18),
];

// ───────────── Núcleo ─────────────

/// Radio nuclear R = r₀·A^(1/3) con r₀ = 1.2 fm.
pub fn radio_nuclear_fm(a: u32) -> f64 {
    1.2 * (a as f64).cbrt()
}

/// Radio de carga del protón (CODATA 2018), fm.
pub const RADIO_PROTON_FM: f64 = 0.8414;

/// Energía de enlace (MeV) por la fórmula semiempírica de Weizsäcker:
/// B = aV·A − aS·A^(2/3) − aC·Z(Z−1)/A^(1/3) − aA·(A−2Z)²/A ± δ.
pub fn energia_enlace_mev(z: u32, a: u32) -> f64 {
    if a < 2 || z == 0 || z > a {
        return 0.0;
    }
    let (zf, af) = (z as f64, a as f64);
    let n = a - z;
    let volumen = 15.75 * af;
    let superficie = 17.8 * af.powf(2.0 / 3.0);
    let coulomb = 0.711 * zf * (zf - 1.0) / af.cbrt();
    let asimetria = 23.7 * (af - 2.0 * zf).powi(2) / af;
    let pareo = 11.18 / af.sqrt();
    let delta = match (z % 2, n % 2) {
        (0, 0) => pareo,
        (1, 1) => -pareo,
        _ => 0.0,
    };
    (volumen - superficie - coulomb - asimetria + delta).max(0.0)
}

/// Defecto de masa (u): la masa que "falta" porque se convirtió en energía de enlace (E = mc²).
pub fn defecto_masa_u(z: u32, a: u32) -> f64 {
    energia_enlace_mev(z, a) / U_MEV
}

/// Masa del átomo neutro (u) = partículas − defecto de masa.
pub fn masa_atomica_u(z: u32, a: u32) -> f64 {
    let n = a - z;
    z as f64 * (M_PROTON_U + M_ELECTRON_U) + n as f64 * M_NEUTRON_U - defecto_masa_u(z, a)
}

// ───────────── Moles ─────────────

/// Masa molar (g/mol): numéricamente igual a la masa atómica en u.
pub fn masa_molar(z: u32, a: u32) -> f64 {
    masa_atomica_u(z, a)
}

/// Número de átomos en `gramos` de un isótopo puro: N = (m / M)·N_A.
pub fn atomos_en_gramos(z: u32, a: u32, gramos: f64) -> f64 {
    gramos / masa_molar(z, a) * NA
}

// ───────────── Electrones ─────────────

/// Grupos de Slater en orden: [1s] [2s,2p] [3s,3p] [3d] [4s,4p] [4d] [4f] [5s,5p] ...
fn grupo_slater(n: u32, l: u32) -> (u32, u32) {
    // (n, 0) agrupa s y p; d y f van solos. Devuelve una clave ordenable.
    match l {
        0 | 1 => (n, 0),
        _ => (n, l),
    }
}

fn orden_grupo(g: (u32, u32)) -> (u32, u32) {
    // ns,np < nd < nf dentro de la misma n, y nd queda por debajo de (n+1)s,p.
    (g.0, g.1)
}

/// Carga nuclear efectiva que siente un electrón de la subcapa (n, l), por las reglas de Slater.
/// `config` son las ocupaciones (n, l, electrones) del átomo.
pub fn z_efectiva(z: u32, config: &[(u32, u32, u32)], n: u32, l: u32) -> f64 {
    let g = grupo_slater(n, l);
    let mut s = 0.0;
    for &(cn, cl, e) in config {
        let cg = grupo_slater(cn, cl);
        let e = e as f64;
        if cg == g {
            // Los demás electrones del mismo grupo.
            let otros = if cn == n && cl == l { e - 1.0 } else { e };
            s += otros * if n == 1 { 0.30 } else { 0.35 };
        } else if orden_grupo(cg) < orden_grupo(g) || cn < n {
            if l >= 2 {
                s += e; // para d y f, todo lo interior apantalla 1.0
            } else if cn == n - 1 {
                s += 0.85 * e;
            } else if cn < n - 1 {
                s += e;
            }
        }
    }
    (z as f64 - s).max(1.0)
}

/// Energía del nivel n en un átomo hidrogenoide de carga Z_ef: Eₙ = −13.6 eV·Z²/n².
pub fn energia_nivel_ev(z_ef: f64, n: u32) -> f64 {
    -RYDBERG_EV * z_ef * z_ef / (n * n) as f64
}

/// Longitud de onda (nm) del fotón emitido o absorbido al saltar entre niveles: λ = hc/ΔE.
pub fn longitud_onda_nm(delta_e_ev: f64) -> f64 {
    HC_EV_NM / delta_e_ev.abs()
}

/// Color aproximado de una longitud de onda visible (380–750 nm); None si es invisible.
pub fn color_visible(nm: f64) -> Option<(f32, f32, f32)> {
    let (r, g, b) = match nm {
        x if (380.0..440.0).contains(&x) => (-(x - 440.0) / 60.0, 0.0, 1.0),
        x if (440.0..490.0).contains(&x) => (0.0, (x - 440.0) / 50.0, 1.0),
        x if (490.0..510.0).contains(&x) => (0.0, 1.0, -(x - 510.0) / 20.0),
        x if (510.0..580.0).contains(&x) => ((x - 510.0) / 70.0, 1.0, 0.0),
        x if (580.0..645.0).contains(&x) => (1.0, -(x - 645.0) / 65.0, 0.0),
        x if (645.0..=750.0).contains(&x) => (1.0, 0.0, 0.0),
        _ => return None,
    };
    Some((r as f32, g as f32, b as f32))
}

/// Región del espectro electromagnético.
pub fn region_espectro(nm: f64) -> &'static str {
    match nm {
        x if x < 10.0 => "rayos X",
        x if x < 380.0 => "ultravioleta",
        x if x <= 750.0 => "luz visible",
        x if x < 1e6 => "infrarrojo",
        _ => "microondas",
    }
}

// ───────────── Funciones de onda ─────────────

/// Polinomio generalizado de Laguerre L_k^α(x) por recurrencia.
fn laguerre(k: u32, alfa: f64, x: f64) -> f64 {
    if k == 0 {
        return 1.0;
    }
    let mut l0 = 1.0;
    let mut l1 = 1.0 + alfa - x;
    for i in 1..k {
        let i = i as f64;
        let l2 = ((2.0 * i + 1.0 + alfa - x) * l1 - (i + alfa) * l0) / (i + 1.0);
        l0 = l1;
        l1 = l2;
    }
    l1
}

/// Parte radial R_nl(r) sin normalizar; r en radios de Bohr.
pub fn radial(n: u32, l: u32, z_ef: f64, r: f64) -> f64 {
    let rho = 2.0 * z_ef * r / n as f64;
    rho.powi(l as i32) * (-rho / 2.0).exp() * laguerre(n - l - 1, (2 * l + 1) as f64, rho)
}

/// Polinomio asociado de Legendre P_l^m(x), m ≥ 0.
fn legendre(l: u32, m: u32, x: f64) -> f64 {
    let mut pmm = 1.0;
    if m > 0 {
        let s = (1.0 - x * x).max(0.0).sqrt();
        let mut f = 1.0;
        for _ in 0..m {
            pmm *= -f * s;
            f += 2.0;
        }
    }
    if l == m {
        return pmm;
    }
    let mut pmmp1 = x * (2 * m + 1) as f64 * pmm;
    if l == m + 1 {
        return pmmp1;
    }
    let mut pll = 0.0;
    for ll in m + 2..=l {
        pll = (x * (2 * ll - 1) as f64 * pmmp1 - (ll + m - 1) as f64 * pmm) / (ll - m) as f64;
        pmm = pmmp1;
        pmmp1 = pll;
    }
    pll
}

/// Armónico esférico real Y_lm(θ, φ) sin normalizar (da la forma de los orbitales).
pub fn armonico(l: u32, m: i32, theta: f64, phi: f64) -> f64 {
    let p = legendre(l, m.unsigned_abs(), theta.cos());
    match m {
        0 => p,
        m if m > 0 => p * (m as f64 * phi).cos(),
        m => p * ((-m) as f64 * phi).sin(),
    }
}

/// Muestrea puntos (en radios de Bohr) distribuidos según |ψ_nlm|²: así se ve la nube real.
pub fn muestrear_orbital(n: u32, l: u32, m: i32, z_ef: f64, puntos: usize, rng: &mut crate::azar::Rng) -> Vec<[f64; 3]> {
    // Distribución radial P(r) = r²·R², tabulada e invertida.
    let r_max = 4.0 * (n * n) as f64 / z_ef + 6.0 / z_ef;
    let pasos = 600;
    let dr = r_max / pasos as f64;
    let mut cdf = Vec::with_capacity(pasos + 1);
    let mut acum = 0.0;
    cdf.push(0.0);
    for i in 1..=pasos {
        let r = i as f64 * dr;
        let rr = radial(n, l, z_ef, r);
        acum += r * r * rr * rr * dr;
        cdf.push(acum);
    }
    // Máximo de |Y|² para el muestreo angular por rechazo.
    let mut y_max = 1e-12f64;
    for i in 0..60 {
        for j in 0..120 {
            let y = armonico(l, m, PI * (i as f64 + 0.5) / 60.0, 2.0 * PI * j as f64 / 120.0);
            y_max = y_max.max(y * y);
        }
    }
    let mut v = Vec::with_capacity(puntos);
    while v.len() < puntos {
        let u = rng.unif() * acum;
        let k = cdf.partition_point(|&c| c < u).clamp(1, pasos);
        let r = (k as f64 - rng.unif()) * dr;
        loop {
            let cos_t = 2.0 * rng.unif() - 1.0;
            let theta = cos_t.acos();
            let phi = 2.0 * PI * rng.unif();
            let y = armonico(l, m, theta, phi);
            if rng.unif() * y_max <= y * y {
                let s = theta.sin();
                v.push([r * s * phi.cos(), r * cos_t, r * s * phi.sin()]);
                break;
            }
        }
    }
    v
}

/// Radio más probable de la subcapa (n, l), en radios de Bohr.
pub fn radio_mas_probable(n: u32, l: u32, z_ef: f64) -> f64 {
    let r_max = 4.0 * (n * n) as f64 / z_ef + 6.0 / z_ef;
    (1..2000)
        .map(|i| i as f64 * r_max / 2000.0)
        .max_by(|a, b| {
            let pa = a * a * radial(n, l, z_ef, *a).powi(2);
            let pb = b * b * radial(n, l, z_ef, *b).powi(2);
            pa.total_cmp(&pb)
        })
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hierro_56_energia_por_nucleon() {
        let b = energia_enlace_mev(26, 56) / 56.0;
        assert!((b - 8.79).abs() < 0.15, "B/A Fe-56 = {b}");
    }

    #[test]
    fn uranio_mas_ligado_que_sus_partes_pero_menos_por_nucleon() {
        let u = energia_enlace_mev(92, 235) / 235.0;
        assert!((u - 7.59).abs() < 0.2, "B/A U-235 = {u}");
        assert!(u < energia_enlace_mev(26, 56) / 56.0); // por eso la fisión libera energía
    }

    #[test]
    fn linea_h_alfa_de_balmer() {
        let de = energia_nivel_ev(1.0, 3) - energia_nivel_ev(1.0, 2);
        let nm = longitud_onda_nm(de);
        assert!((nm - 656.1).abs() < 1.0, "Hα = {nm} nm");
        assert_eq!(region_espectro(nm), "luz visible");
    }

    #[test]
    fn slater_oxigeno_2p() {
        let cfg = [(1, 0, 2), (2, 0, 2), (2, 1, 4)];
        let z = z_efectiva(8, &cfg, 2, 1);
        assert!((z - 4.55).abs() < 0.01, "Z_ef O 2p = {z}");
    }

    #[test]
    fn hidrogeno_1s_mas_probable_en_a0() {
        let r = radio_mas_probable(1, 0, 1.0);
        assert!((r - 1.0).abs() < 0.01, "r = {r}");
    }

    #[test]
    fn un_mol_de_carbono_12() {
        let m = masa_molar(6, 12);
        assert!((m - 12.0).abs() < 0.01, "M(C-12) = {m}");
        let n = atomos_en_gramos(6, 12, 12.0);
        assert!((n / NA - 1.0).abs() < 0.001);
    }

    #[test]
    fn radio_uranio() {
        assert!((radio_nuclear_fm(238) - 7.44).abs() < 0.05);
    }

    #[test]
    fn nube_2p_tiene_nodo_en_el_plano() {
        // 2p con m=0 apunta en el eje y: casi ningún punto en el plano y=0.
        let mut rng = crate::azar::Rng::nuevo(3);
        let pts = muestrear_orbital(2, 1, 0, 1.0, 2000, &mut rng);
        let en_plano = pts.iter().filter(|p| p[1].abs() < 0.1).count();
        assert!(en_plano < 60, "{en_plano}");
    }
}
