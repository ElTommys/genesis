# GÉNESIS · del átomo a la vida

Simulador por escalas escrito en Rust, sin dependencias externas. Cada escala sale de las reglas de la anterior:

## Jerarquía de la materia (columna vertebral del simulador)

Cada nivel se construye solo con las reglas del anterior:

1. **Nivel químico (abiótico):** partícula subatómica (quarks → protón/neutrón; electrón) → **átomo** → **molécula** → **macromolécula** (proteínas, lípidos, ADN) → **complejo supramolecular / orgánulo** (membrana, mitocondria, núcleo).
2. **Nivel celular:** **célula**, la unidad mínima con vida (neurona, glóbulo rojo, célula vegetal).
3. **Nivel tisular y orgánico:** **tejido** (epitelial, muscular, nervioso, conectivo; en plantas: meristemos, parénquima, xilema y floema) → **órgano** → **sistema o aparato**.
4. **Nivel de organismo:** el ser vivo completo, con todos sus sistemas en equilibrio.

| Escala | Módulo | Estado |
|---|---|---|
| 0 · Partículas: protón (uud), neutrón (udd), electrón; carga, masa y spin | `particula.rs`, `fisica.rs` | ✅ |
| 0 · Quarks y gluones: carga de color, fuerza fuerte (visual) | `vista_atomo.rs` | ✅ visual |
| 1 · Núcleo: R = 1.2·A^⅓ fm, energía de enlace (Weizsäcker), defecto de masa E = mc² | `fisica.rs` | ✅ |
| 1 · Átomo cuántico: n, l, mₗ, mₛ · Aufbau · Pauli · Hund · Z_ef de Slater · ψ y \|ψ\|² · niveles y fotones | `cuantica.rs`, `fisica.rs` | ✅ |
| 1 · Tabla periódica, moles y masa molar | `cuantica.rs`, `fisica.rs` | ✅ |
| 2 · Moléculas reales desde SMILES: geometría por física, enlaces simples/dobles/triples/aromáticos, tipo de enlace, energías, masa molar, dipolo | `quimica.rs`, `vista_molecula.rs` | ✅ |
| 2 · Reacciones: balance (Lavoisier) y ΔH por energías de enlace | `quimica.rs` | ✅ |
| 2-3 · Química dinámica: átomos que chocan y reaccionan solos | `mundo.rs` | ✅ |
| 3 · Fisión y fusión (U-235 → productos; cadenas de decaimiento) | — | siguiente |
| 4 · Bioquímica: agua, aminoácidos, ADN/ARN, código genético, proteínas | `vida.rs` | 🟡 parcial |
| 4 · Metabolismo: ATP, glucólisis, ciclo de Krebs, cadena respiratoria, fotosíntesis | — | pendiente |
| 5 · Célula: membrana, bomba Na⁺/K⁺, orgánulos, vacuola; animal vs vegetal; mitosis y meiosis | — | pendiente |
| 6 · Tejidos animales: epitelial, conectivo, sanguíneo, muscular, nervioso → órganos y sistemas | — | pendiente |
| 6 · Tejidos vegetales: meristemos, parénquima (clorofila), xilema y floema, epidermis y estomas (intercambio gaseoso) | — | pendiente |
| 6 · Digestión: los alimentos se descomponen en glucosa, aminoácidos y lípidos, que alimentan el ATP y la síntesis de proteínas | — | pendiente |
| 7 · Organismo → población → comunidad → ecosistema: densidad, estructura de edades, riqueza, índice de Shannon, pirámide de biomasa | `bioma.rs`, `vista_mundo.rs` | ✅ |
| 8 · Biomas por Whittaker (tundra, taiga, bosque, pradera, desierto, matorral, sabana, selva) con relieve, clima (−6.5 °C/km) y lluvia orográfica | `clima.rs` | ✅ |
| 8 · 23 especies reales con nicho climático, dependencia de la densidad e inmigración | `conocimiento.rs`, `bioma.rs` | ✅ |
| 9 · Biosfera: cambio climático global editable (los biomas se desplazan) | `clima.rs`, `vista_mundo.rs` | ✅ |

## Uso

```
cargo run --release -- mundo           # BIOSFERA 3D: zoom de organismo a bioma
cargo run --release -- atomo U        # microscopio cuántico: zoom del átomo a los quarks
cargo run --release -- molecula glucosa  # molécula 3D (o cualquier SMILES: molecula "CCO")
cargo run --release -- reaccion "C6H12O6 + 6O2 -> 6CO2 + 6H2O"
cargo run --release -- tabla
cargo run --release -- elemento Fe
cargo run --release -- simular --atomos H:400,O:200,C:50 --caja 14 --temp 1.0 --pasos 20000
cargo run --release -- simular --atomos Na+:50,Cl-:50 --caja 12
```

Para ver todas las variables que se pueden editar: `cargo run --release -- ayuda`.

## Limitaciones conocidas (por ahora)
- Aufbau puro: no reproduce las excepciones como Cr y Cu (3d⁵4s¹, 3d¹⁰4s¹).
- Los neutrones salen de una aproximación del isótopo más común.
- La química usa un modelo simplificado: valencia + resortes. Todavía no tiene geometría ni energías reales.
- El archivo `.cargo/config.toml` fija un SDK de macOS para evitar un error del linker en el Mac de desarrollo. En otro Mac, borra esa línea.
