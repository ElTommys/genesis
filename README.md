# Universo

Simulador por escalas escrito en Rust, sin dependencias externas. Cada escala sale de las reglas de la anterior:

| Escala | Módulo | Estado |
|---|---|---|
| 0 · Partículas (protón, neutrón, electrón: carga, masa, spin) | `src/particula.rs` | ✅ |
| 1 · Átomo (n, l, mₗ, mₛ · Aufbau · Pauli · Hund → tabla periódica) | `src/cuantica.rs` | ✅ |
| 2 · Mundo 3D (Lennard-Jones, Coulomb, enlaces, termostato) | `src/mundo.rs` | ✅ |
| 3 · Química (ángulos de enlace, electronegatividad, energías reales) | — | siguiente |
| 4 · Bioquímica (aminoácidos, ADN, membranas) | — | |
| 5 · Célula · 6 · Organismos · 7 · Bioma | — | |

## Uso

```
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
