//! Base de conocimiento: lo que la ciencia ya ha descubierto y medido.
//! El simulador la usa para sembrar el bioma y la muestra con `genesis saber`.
//! Para agregar conocimiento solo se añaden filas a estas tablas.

pub struct Tema {
    pub clave: &'static str,
    pub titulo: &'static str,
    pub entradas: &'static [(&'static str, &'static str)],
}

pub const TEMAS: &[Tema] = &[
    Tema {
        clave: "dominios",
        titulo: "Árbol de la vida (Woese, 1990) y reinos",
        entradas: &[
            ("Bacteria", "Procariotas sin núcleo. Ej: E. coli, cianobacterias (inventaron la fotosíntesis oxigénica, ~2.700 Ma)."),
            ("Archaea", "Procariotas con membranas de éteres lipídicos; muchas extremófilas (metanógenas, halófilas)."),
            ("Eukarya", "Células con núcleo y orgánulos. Incluye los reinos de abajo."),
            ("  Protista", "Eucariotas mayormente unicelulares: amebas, paramecios, algas."),
            ("  Fungi", "Hongos: heterótrofos por absorción, pared de quitina. Descomponedores clave."),
            ("  Plantae", "Autótrofos fotosintéticos, pared de celulosa, cloroplastos."),
            ("  Animalia", "Heterótrofos multicelulares por ingestión, sin pared celular."),
            ("Virus", "Acelulares: genoma (ADN o ARN) + cápside. Solo se replican dentro de una célula."),
        ],
    },
    Tema {
        clave: "celulas",
        titulo: "Tipos de célula y orgánulos",
        entradas: &[
            ("Procariota", "1-10 µm. ADN circular en el nucleoide, ribosomas 70S, plásmidos, a veces flagelo."),
            ("Eucariota animal", "10-30 µm. Núcleo, mitocondrias, RE, Golgi, lisosomas, centriolos."),
            ("Eucariota vegetal", "10-100 µm. Además: pared de celulosa, cloroplastos, gran vacuola central."),
            ("Eucariota fúngica", "Pared de quitina, hifas; sin cloroplastos."),
            ("Núcleo", "Guarda el ADN. En humanos: 46 cromosomas, ~3.100 millones de pares de bases."),
            ("Mitocondria", "Respiración celular: glucosa + O₂ → ~30-32 ATP. Tiene su propio ADN (herencia materna)."),
            ("Cloroplasto", "Fotosíntesis: 6CO₂ + 6H₂O + luz → C₆H₁₂O₆ + 6O₂."),
            ("Ribosoma", "Traduce ARNm a proteína usando el código genético."),
            ("Membrana", "Bicapa de fosfolípidos: cabeza polar (hidrófila) y colas apolares."),
        ],
    },
    Tema {
        clave: "adn",
        titulo: "ADN, genes y código genético",
        entradas: &[
            ("Doble hélice", "Watson, Crick, Franklin y Wilkins (1953). Pares A–T (2 puentes H) y G–C (3 puentes H)."),
            ("Dogma central", "ADN → (transcripción) → ARN → (traducción) → proteína."),
            ("Codón", "3 bases = 1 aminoácido. 64 codones → 20 aminoácidos + 3 de parada. Inicio: AUG (Met)."),
            ("Genoma humano", "~20.000 genes codificantes de proteínas; Proyecto Genoma Humano completado en 2003 (T2T en 2022)."),
            ("Herencia", "Mendel (1866): dominante/recesivo, segregación y distribución independiente."),
            ("CRISPR-Cas9", "Edición genética dirigida (Charpentier y Doudna, 2012; Nobel 2020)."),
        ],
    },
    Tema {
        clave: "mutaciones",
        titulo: "Mutaciones",
        entradas: &[
            ("Sustitución", "Cambia una base. Silenciosa (mismo aminoácido), de sentido erróneo (otro aminoácido) o sin sentido (parada)."),
            ("Inserción", "Agrega bases; si no son múltiplo de 3 corre el marco de lectura."),
            ("Deleción", "Quita bases; también puede correr el marco."),
            ("Duplicación", "Copia un segmento: materia prima de genes nuevos."),
            ("Inversión / Translocación", "Reordenamientos cromosómicos grandes."),
            ("Tasa humana", "~1,2 × 10⁻⁸ por base y por generación (~60-70 mutaciones nuevas por persona)."),
            ("Selección natural", "Darwin y Wallace (1858): sobreviven y se reproducen más los mejor adaptados."),
        ],
    },
    Tema {
        clave: "reproduccion",
        titulo: "Reproducción, clonación, gemelos y mellizos",
        entradas: &[
            ("Fisión binaria", "Bacterias: una célula se divide en dos copias. E. coli puede hacerlo cada ~20 min."),
            ("Gemación / esporas", "Levaduras, hidras y hongos."),
            ("Clonación natural", "Estolones de fresa, rizomas, tubérculos de papa, partenogénesis (pulgones, algunos lagartos)."),
            ("Clonación artificial", "Transferencia nuclear: oveja Dolly (Roslin, 1996). Primeros primates clonados: Zhong Zhong y Hua Hua (2018)."),
            ("Reproducción sexual", "Meiosis + fecundación: cada hijo recibe la mitad del genoma de cada progenitor."),
            ("Gemelos (monocigóticos)", "Un cigoto que se divide: genoma prácticamente idéntico. ~3-4 por cada 1.000 partos, tasa casi constante en el mundo."),
            ("Mellizos (dicigóticos)", "Dos óvulos fecundados por dos espermatozoides: comparten ~50% como hermanos. Tasa variable (genética, edad materna)."),
            ("Armadillo de nueve bandas", "Siempre pare cuatrillizos idénticos (poliembrionía)."),
        ],
    },
    Tema {
        clave: "trofico",
        titulo: "Cadena alimenticia y niveles tróficos",
        entradas: &[
            ("Productores", "Autótrofos: plantas, algas, cianobacterias. Capturan energía solar."),
            ("Consumidores primarios", "Herbívoros: comen productores (conejo, vaca, saltamontes)."),
            ("Consumidores secundarios", "Carnívoros u omnívoros que comen herbívoros (zorro, rana)."),
            ("Consumidores terciarios", "Depredadores tope (lobo, águila, orca)."),
            ("Descomponedores", "Hongos y bacterias: devuelven los nutrientes de los cadáveres al suelo."),
            ("Regla del 10%", "Lindeman (1942): solo ~10% de la energía pasa de un nivel al siguiente; por eso hay pirámides."),
        ],
    },
    Tema {
        clave: "dietas",
        titulo: "Tipos de dieta",
        entradas: &[
            ("Autótrofo", "Fabrica su alimento (fotosíntesis o quimiosíntesis)."),
            ("Herbívoro", "Solo vegetales. Intestinos largos, microbiota que digiere celulosa."),
            ("Carnívoro", "Solo animales. Ej: felinos (carnívoros estrictos)."),
            ("Omnívoro", "Ambos. Ej: humano, oso, cerdo, cuervo."),
            ("Detritívoro / descomponedor", "Materia muerta: lombrices, hongos, bacterias."),
        ],
    },
    Tema {
        clave: "animales",
        titulo: "Clases de animales",
        entradas: &[
            ("Mamíferos", "Glándulas mamarias, pelo, endotermos. ~6.500 especies."),
            ("Aves", "Plumas, huevos con cáscara, endotermos. ~11.000 especies."),
            ("Reptiles", "Escamas, huevo amniota, ectotermos. ~12.000 especies."),
            ("Anfibios", "Metamorfosis, piel permeable. ~8.700 especies."),
            ("Peces", "Branquias y aletas. ~36.000 especies (el grupo vertebrado más grande)."),
            ("Artrópodos", "Exoesqueleto de quitina; insectos, arácnidos, crustáceos. >1 millón de especies descritas."),
            ("Moluscos", "Cuerpo blando, a menudo con concha: caracoles, pulpos, almejas."),
        ],
    },
    Tema {
        clave: "plantas",
        titulo: "Tipos de plantas",
        entradas: &[
            ("Briófitas", "Musgos y hepáticas: sin vasos conductores, necesitan agua para reproducirse."),
            ("Pteridófitas", "Helechos: vasos conductores, se reproducen por esporas."),
            ("Gimnospermas", "Semillas desnudas (piñas): pinos, abetos, ginkgo, cícadas."),
            ("Angiospermas monocotiledóneas", "Flores y frutos, un cotiledón: pastos, maíz, palmas, orquídeas."),
            ("Angiospermas dicotiledóneas", "Dos cotiledones: rosas, frijol, roble, cafeto."),
            ("Fotosíntesis C3 / C4 / CAM", "C4 (maíz, caña) rinde más con calor; CAM (cactus, piña) abre estomas de noche."),
        ],
    },
    Tema {
        clave: "razas",
        titulo: "Razas y variedades (selección artificial)",
        entradas: &[
            ("Perro (Canis lupus familiaris)", "Domesticado hace >15.000 años; ~360 razas reconocidas (FCI): chihuahua, pastor alemán, labrador, gran danés."),
            ("Gato (Felis catus)", "Siamés, persa, maine coon, esfinge, bengalí."),
            ("Vaca (Bos taurus / B. indicus)", "Holstein (leche), Angus (carne), cebú Brahman (trópico), Normando."),
            ("Caballo (Equus caballus)", "Árabe, pura sangre inglés, criollo colombiano, percherón."),
            ("Gallina (Gallus gallus domesticus)", "Leghorn, Rhode Island Red, Brahma."),
            ("Maíz (Zea mays)", "Domesticado del teocintle en México hace ~9.000 años."),
            ("Café (Coffea arabica)", "Variedades Típica, Borbón, Caturra, Castillo (Colombia), Geisha."),
        ],
    },
];

/// Especies con datos reales usadas para sembrar el bioma.
/// rasgos = [tamaño, velocidad, visión, dieta, fertilidad, longevidad] normalizados 0..1.
pub struct Plantilla {
    pub nombre: &'static str,
    pub cientifico: &'static str,
    pub reino: Reino,
    pub clase: &'static str,
    pub masa_kg: f64,
    pub vida_anios: f64,
    pub rasgos: [f64; 6],
    /// Temperatura media anual óptima (°C) y tolerancia (°C) de su nicho climático.
    pub t_opt: f32,
    pub t_tol: f32,
    /// Lluvia anual mínima que necesita (mm).
    pub lluvia_min: f32,
    /// Biomas donde vive en la naturaleza.
    pub habitat: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reino {
    Plantae,
    Animalia,
    Fungi,
}

pub const PLANTILLAS: &[Plantilla] = &[
    // ── Productores ──
    Plantilla { nombre: "Pasto", cientifico: "Poaceae", reino: Reino::Plantae, clase: "Monocotiledónea", masa_kg: 0.01, vida_anios: 2.0, rasgos: [0.2, 0.0, 0.0, 0.0, 0.7, 0.3], t_opt: 16.0, t_tol: 14.0, lluvia_min: 250.0, habitat: "pradera, sabana, bosque templado" },
    Plantilla { nombre: "Trébol", cientifico: "Trifolium repens", reino: Reino::Plantae, clase: "Dicotiledónea", masa_kg: 0.005, vida_anios: 3.0, rasgos: [0.15, 0.0, 0.0, 0.0, 0.8, 0.35], t_opt: 12.0, t_tol: 9.0, lluvia_min: 500.0, habitat: "pradera, bosque templado" },
    Plantilla { nombre: "Roble", cientifico: "Quercus robur", reino: Reino::Plantae, clase: "Angiosperma dicotiledónea", masa_kg: 5000.0, vida_anios: 500.0, rasgos: [0.85, 0.0, 0.0, 0.0, 0.3, 0.95], t_opt: 11.0, t_tol: 6.0, lluvia_min: 600.0, habitat: "bosque templado" },
    Plantilla { nombre: "Pino", cientifico: "Pinus sylvestris", reino: Reino::Plantae, clase: "Gimnosperma conífera", masa_kg: 2000.0, vida_anios: 300.0, rasgos: [0.75, 0.0, 0.0, 0.0, 0.35, 0.9], t_opt: 1.0, t_tol: 7.0, lluvia_min: 300.0, habitat: "taiga" },
    Plantilla { nombre: "Liquen", cientifico: "Cladonia rangiferina", reino: Reino::Plantae, clase: "Simbiosis hongo-alga", masa_kg: 0.001, vida_anios: 50.0, rasgos: [0.1, 0.0, 0.0, 0.0, 0.6, 0.6], t_opt: -8.0, t_tol: 7.0, lluvia_min: 100.0, habitat: "tundra" },
    Plantilla { nombre: "Saguaro", cientifico: "Carnegiea gigantea", reino: Reino::Plantae, clase: "Cactácea (CAM)", masa_kg: 2000.0, vida_anios: 150.0, rasgos: [0.6, 0.0, 0.0, 0.0, 0.3, 0.9], t_opt: 23.0, t_tol: 8.0, lluvia_min: 50.0, habitat: "desierto, matorral" },
    Plantilla { nombre: "Acacia", cientifico: "Vachellia tortilis", reino: Reino::Plantae, clase: "Leguminosa", masa_kg: 1500.0, vida_anios: 100.0, rasgos: [0.7, 0.0, 0.0, 0.0, 0.4, 0.85], t_opt: 25.0, t_tol: 6.0, lluvia_min: 300.0, habitat: "sabana, matorral" },
    Plantilla { nombre: "Ceiba", cientifico: "Ceiba pentandra", reino: Reino::Plantae, clase: "Angiosperma dicotiledónea", masa_kg: 20000.0, vida_anios: 300.0, rasgos: [0.95, 0.0, 0.0, 0.0, 0.35, 0.95], t_opt: 26.0, t_tol: 4.0, lluvia_min: 1500.0, habitat: "selva tropical" },
    Plantilla { nombre: "Helecho", cientifico: "Pteridium aquilinum", reino: Reino::Plantae, clase: "Pteridófita", masa_kg: 0.5, vida_anios: 10.0, rasgos: [0.3, 0.0, 0.0, 0.0, 0.75, 0.4], t_opt: 22.0, t_tol: 8.0, lluvia_min: 1200.0, habitat: "selva tropical, bosque templado húmedo" },
    // ── Herbívoros ──
    Plantilla { nombre: "Conejo", cientifico: "Oryctolagus cuniculus", reino: Reino::Animalia, clase: "Mamífero", masa_kg: 2.0, vida_anios: 9.0, rasgos: [0.25, 0.6, 0.45, 0.1, 0.8, 0.25], t_opt: 13.0, t_tol: 11.0, lluvia_min: 250.0, habitat: "pradera, bosque templado" },
    Plantilla { nombre: "Venado", cientifico: "Odocoileus virginianus", reino: Reino::Animalia, clase: "Mamífero", masa_kg: 70.0, vida_anios: 15.0, rasgos: [0.5, 0.6, 0.55, 0.05, 0.4, 0.45], t_opt: 10.0, t_tol: 10.0, lluvia_min: 400.0, habitat: "bosque templado" },
    Plantilla { nombre: "Reno", cientifico: "Rangifer tarandus", reino: Reino::Animalia, clase: "Mamífero", masa_kg: 120.0, vida_anios: 15.0, rasgos: [0.55, 0.55, 0.55, 0.05, 0.35, 0.45], t_opt: -6.0, t_tol: 9.0, lluvia_min: 100.0, habitat: "tundra, taiga" },
    Plantilla { nombre: "Camello", cientifico: "Camelus dromedarius", reino: Reino::Animalia, clase: "Mamífero", masa_kg: 500.0, vida_anios: 40.0, rasgos: [0.7, 0.4, 0.6, 0.08, 0.25, 0.7], t_opt: 25.0, t_tol: 9.0, lluvia_min: 30.0, habitat: "desierto" },
    Plantilla { nombre: "Cebra", cientifico: "Equus quagga", reino: Reino::Animalia, clase: "Mamífero", masa_kg: 300.0, vida_anios: 25.0, rasgos: [0.6, 0.65, 0.55, 0.06, 0.35, 0.55], t_opt: 25.0, t_tol: 6.0, lluvia_min: 300.0, habitat: "sabana" },
    Plantilla { nombre: "Mono aullador", cientifico: "Alouatta palliata", reino: Reino::Animalia, clase: "Mamífero primate", masa_kg: 7.0, vida_anios: 20.0, rasgos: [0.35, 0.45, 0.6, 0.2, 0.35, 0.5], t_opt: 26.0, t_tol: 4.0, lluvia_min: 1500.0, habitat: "selva tropical" },
    // ── Omnívoros ──
    Plantilla { nombre: "Jabalí", cientifico: "Sus scrofa", reino: Reino::Animalia, clase: "Mamífero", masa_kg: 90.0, vida_anios: 12.0, rasgos: [0.55, 0.45, 0.5, 0.5, 0.45, 0.4], t_opt: 12.0, t_tol: 12.0, lluvia_min: 300.0, habitat: "bosque templado, pradera" },
    Plantilla { nombre: "Oso pardo", cientifico: "Ursus arctos", reino: Reino::Animalia, clase: "Mamífero", masa_kg: 300.0, vida_anios: 25.0, rasgos: [0.75, 0.45, 0.6, 0.55, 0.25, 0.6], t_opt: 3.0, t_tol: 9.0, lluvia_min: 300.0, habitat: "taiga, bosque templado" },
    Plantilla { nombre: "Humano", cientifico: "Homo sapiens", reino: Reino::Animalia, clase: "Mamífero primate", masa_kg: 70.0, vida_anios: 73.0, rasgos: [0.55, 0.5, 0.7, 0.5, 0.15, 0.9], t_opt: 20.0, t_tol: 14.0, lluvia_min: 250.0, habitat: "casi todos los biomas (gracias a la cultura y la tecnología)" },
    // ── Carnívoros ──
    Plantilla { nombre: "Zorro", cientifico: "Vulpes vulpes", reino: Reino::Animalia, clase: "Mamífero", masa_kg: 6.0, vida_anios: 5.0, rasgos: [0.4, 0.65, 0.65, 0.8, 0.45, 0.35], t_opt: 10.0, t_tol: 14.0, lluvia_min: 150.0, habitat: "casi todos los biomas templados" },
    Plantilla { nombre: "Zorro ártico", cientifico: "Vulpes lagopus", reino: Reino::Animalia, clase: "Mamífero", masa_kg: 4.0, vida_anios: 5.0, rasgos: [0.38, 0.6, 0.65, 0.85, 0.45, 0.35], t_opt: -8.0, t_tol: 8.0, lluvia_min: 50.0, habitat: "tundra" },
    Plantilla { nombre: "Lobo", cientifico: "Canis lupus", reino: Reino::Animalia, clase: "Mamífero", masa_kg: 40.0, vida_anios: 8.0, rasgos: [0.6, 0.7, 0.75, 0.95, 0.35, 0.5], t_opt: 4.0, t_tol: 11.0, lluvia_min: 200.0, habitat: "taiga, bosque templado, tundra" },
    Plantilla { nombre: "León", cientifico: "Panthera leo", reino: Reino::Animalia, clase: "Mamífero felino", masa_kg: 190.0, vida_anios: 14.0, rasgos: [0.7, 0.7, 0.7, 0.95, 0.35, 0.5], t_opt: 25.0, t_tol: 6.0, lluvia_min: 250.0, habitat: "sabana" },
    Plantilla { nombre: "Jaguar", cientifico: "Panthera onca", reino: Reino::Animalia, clase: "Mamífero felino", masa_kg: 90.0, vida_anios: 12.0, rasgos: [0.65, 0.65, 0.7, 0.95, 0.3, 0.5], t_opt: 26.0, t_tol: 4.0, lluvia_min: 1200.0, habitat: "selva tropical" },
    // ── Descomponedores ──
    Plantilla { nombre: "Champiñón", cientifico: "Agaricus bisporus", reino: Reino::Fungi, clase: "Basidiomiceto", masa_kg: 0.02, vida_anios: 1.0, rasgos: [0.15, 0.05, 0.3, 0.0, 0.7, 0.3], t_opt: 15.0, t_tol: 15.0, lluvia_min: 200.0, habitat: "casi todos los biomas" },
];

pub fn plantilla(nombre: &str) -> Option<&'static Plantilla> {
    PLANTILLAS.iter().find(|p| p.nombre.eq_ignore_ascii_case(nombre))
}

pub fn imprimir(clave: Option<&str>) {
    match clave.and_then(|c| TEMAS.iter().find(|t| t.clave == c)) {
        Some(t) => {
            println!("{}\n{}", t.titulo, "─".repeat(t.titulo.chars().count()));
            for (k, v) in t.entradas {
                println!("  {k:<30} {v}");
            }
        }
        None => {
            println!("Temas disponibles (genesis saber <tema>):");
            for t in TEMAS {
                println!("  {:<14} {}", t.clave, t.titulo);
            }
            println!("\nEspecies para sembrar el bioma:");
            for p in PLANTILLAS {
                println!("  {:<10} {:<24} {:<16} {:>7} kg  {:>4} años", p.nombre, p.cientifico, p.clase, p.masa_kg, p.vida_anios);
            }
        }
    }
}
