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
