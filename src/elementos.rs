//! Nombres y símbolos (lo único que es convención humana y no física).

pub const SIMBOLOS: [&str; 118] = [
    "H", "He", "Li", "Be", "B", "C", "N", "O", "F", "Ne", "Na", "Mg", "Al", "Si", "P", "S", "Cl",
    "Ar", "K", "Ca", "Sc", "Ti", "V", "Cr", "Mn", "Fe", "Co", "Ni", "Cu", "Zn", "Ga", "Ge", "As",
    "Se", "Br", "Kr", "Rb", "Sr", "Y", "Zr", "Nb", "Mo", "Tc", "Ru", "Rh", "Pd", "Ag", "Cd", "In",
    "Sn", "Sb", "Te", "I", "Xe", "Cs", "Ba", "La", "Ce", "Pr", "Nd", "Pm", "Sm", "Eu", "Gd", "Tb",
    "Dy", "Ho", "Er", "Tm", "Yb", "Lu", "Hf", "Ta", "W", "Re", "Os", "Ir", "Pt", "Au", "Hg", "Tl",
    "Pb", "Bi", "Po", "At", "Rn", "Fr", "Ra", "Ac", "Th", "Pa", "U", "Np", "Pu", "Am", "Cm", "Bk",
    "Cf", "Es", "Fm", "Md", "No", "Lr", "Rf", "Db", "Sg", "Bh", "Hs", "Mt", "Ds", "Rg", "Cn", "Nh",
    "Fl", "Mc", "Lv", "Ts", "Og",
];

pub const NOMBRES: [&str; 118] = [
    "Hidrógeno", "Helio", "Litio", "Berilio", "Boro", "Carbono", "Nitrógeno", "Oxígeno", "Flúor",
    "Neón", "Sodio", "Magnesio", "Aluminio", "Silicio", "Fósforo", "Azufre", "Cloro", "Argón",
    "Potasio", "Calcio", "Escandio", "Titanio", "Vanadio", "Cromo", "Manganeso", "Hierro",
    "Cobalto", "Níquel", "Cobre", "Zinc", "Galio", "Germanio", "Arsénico", "Selenio", "Bromo",
    "Kriptón", "Rubidio", "Estroncio", "Itrio", "Circonio", "Niobio", "Molibdeno", "Tecnecio",
    "Rutenio", "Rodio", "Paladio", "Plata", "Cadmio", "Indio", "Estaño", "Antimonio", "Telurio",
    "Yodo", "Xenón", "Cesio", "Bario", "Lantano", "Cerio", "Praseodimio", "Neodimio", "Prometio",
    "Samario", "Europio", "Gadolinio", "Terbio", "Disprosio", "Holmio", "Erbio", "Tulio", "Iterbio",
    "Lutecio", "Hafnio", "Tantalio", "Wolframio", "Renio", "Osmio", "Iridio", "Platino", "Oro",
    "Mercurio", "Talio", "Plomo", "Bismuto", "Polonio", "Astato", "Radón", "Francio", "Radio",
    "Actinio", "Torio", "Protactinio", "Uranio", "Neptunio", "Plutonio", "Americio", "Curio",
    "Berkelio", "Californio", "Einstenio", "Fermio", "Mendelevio", "Nobelio", "Lawrencio",
    "Rutherfordio", "Dubnio", "Seaborgio", "Bohrio", "Hasio", "Meitnerio", "Darmstatio",
    "Roentgenio", "Copernicio", "Nihonio", "Flerovio", "Moscovio", "Livermorio", "Teneso",
    "Oganesón",
];

pub fn z_de_simbolo(s: &str) -> Option<u32> {
    SIMBOLOS.iter().position(|x| x.eq_ignore_ascii_case(s)).map(|i| i as u32 + 1)
}

/// Electronegatividad de Pauling (0.0 = sin dato medido).
pub const ELECTRONEGATIVIDAD: [f64; 118] = [
    2.20, 0.0, 0.98, 1.57, 2.04, 2.55, 3.04, 3.44, 3.98, 0.0, // 1-10
    0.93, 1.31, 1.61, 1.90, 2.19, 2.58, 3.16, 0.0, // 11-18
    0.82, 1.00, 1.36, 1.54, 1.63, 1.66, 1.55, 1.83, 1.88, 1.91, 1.90, 1.65, 1.81, 2.01, 2.18, 2.55, 2.96, 3.00, // 19-36
    0.82, 0.95, 1.22, 1.33, 1.60, 2.16, 1.90, 2.20, 2.28, 2.20, 1.93, 1.69, 1.78, 1.96, 2.05, 2.10, 2.66, 2.60, // 37-54
    0.79, 0.89, 1.10, 1.12, 1.13, 1.14, 1.13, 1.17, 1.20, 1.20, 1.10, 1.22, 1.23, 1.24, 1.25, 1.10, // 55-70
    1.27, 1.30, 1.50, 2.36, 1.90, 2.20, 2.20, 2.28, 2.54, 2.00, 1.62, 2.33, 2.02, 2.00, 2.20, 2.20, // 71-86
    0.70, 0.90, 1.10, 1.30, 1.50, 1.38, 1.36, 1.28, 1.13, 1.28, 1.30, 1.30, 1.30, 1.30, 1.30, 1.30, 1.30, // 87-103
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, // 104-118
];

/// Color CPK convencional (r, g, b) para dibujar.
pub fn color_cpk(z: u32) -> (u8, u8, u8) {
    match z {
        1 => (240, 240, 240),
        6 => (80, 80, 80),
        7 => (48, 80, 248),
        8 => (255, 13, 13),
        9 | 17 => (31, 240, 31),
        15 => (255, 128, 0),
        16 => (255, 220, 48),
        11 | 19 => (171, 92, 242),
        12 | 20 => (61, 255, 0),
        26 => (224, 102, 51),
        2 | 10 | 18 | 36 | 54 | 86 => (217, 255, 255),
        _ => (255, 20, 147),
    }
}
