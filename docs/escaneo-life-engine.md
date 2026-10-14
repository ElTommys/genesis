# Escaneo: The Life Engine vs Génesis

**Objetivo:** The Life Engine (MaxRobinsonTheGreat/LifeEngine), código fuente publicado en la rama `master`.
**Método:** análisis estático con REA (`analyze-javascript-application`): 43 archivos JS parseados, 19.114 nodos AST, 275 hallazgos, 0 fallos de parseo. Evidence `ev_e37bac89f078552bb5c42cfb6547aec4c1f6a3f875c30690d14db0cac51230ff`. Después se leyeron a mano los módulos clave. No se ejecutó el programa.
**Licencia:** GPL-3.0. **No se copia código.** De aquí solo se toman ideas de diseño, que no tienen derechos de autor. Si se copiara código, Génesis tendría que volverse GPL-3.

> El repositorio no incluye el bundle compilado (`dist/js/` solo trae el archivo de licencia), así que REA no encontró JavaScript en `dist`. Por eso el análisis se hizo sobre `src/`.

## Cómo funciona (observado en el código)

| Mecánica | The Life Engine | Archivo |
|---|---|---|
| Mundo | Rejilla 2D de celdas discretas | `Grid/GridMap.js` |
| Organismo | Conjunto de células con función: productora, boca, motora, asesina, armadura, ojo | `Organism/Cell/BodyCells/*` |
| Comida | Una **celda física** en la rejilla. Las productoras la ponen en una casilla vecina vacía (5% por tick) | `ProducerCell.js` |
| Energía y hambre | **No hay metabolismo**: nadie muere de hambre. Solo se muere por vejez o por ataque | `Organism.update()` |
| Reproducción | Al acumular comida igual al número de células. **La cría necesita espacio libre y un camino recto**; si no hay espacio, no nace | `Organism.reproduce()` |
| Longevidad | número de células × 100 (lo grande vive más) | `lifespan()` |
| Muerte | Cada célula del cadáver se convierte en una celda de comida (reciclaje directo, sin descomponedores) | `die()` |
| Depredación | Las células asesinas dañan a las vecinas; la armadura bloquea; la salud es el número de células | `KillerCell.js` |
| Mutación | Agregar, cambiar o quitar una célula (33% cada una). **La propia tasa de mutación también muta** (±1 por cría) | `mutate()`, `reproduce()` |
| Cerebro | Tabla de decisiones según el tipo de célula que ve: perseguir, huir o neutral. Muta cambiando una entrada | `Brain.js` |
| Especies | Hay especie nueva cada vez que la anatomía muta. Las que nunca llegaron a 10 individuos se descartan del registro fósil | `FossilRecord.js` |
| Reinicio | `auto_reset` cuando se extingue toda la vida | `WorldEnvironment.js` |

## Por qué su ecosistema no colapsa y el de Génesis sí

1. **Sin hambre no hay colapsos en cadena.** En Génesis los animales gastan energía cada tick (ley de Kleiber), y eso amplifica los auges y las caídas.
2. **El espacio es el freno.** En Life Engine la cría necesita una casilla libre, así que la densidad se regula sola. En Génesis las crías aparecen encima de la madre.
3. **La comida es finita y está ubicada.** Una celda de comida se come una sola vez y desaparece. No existe la "planta con reserva infinita".
4. **Cadáver = comida.** Los nutrientes vuelven de inmediato, sin pasar por los hongos.

## Qué tiene Génesis que Life Engine no tiene

- Física y química reales: partículas, mecánica cuántica, tabla periódica, enlaces, cargas δ±, ángulos VSEPR.
- ADN de verdad (A/C/G/T), código genético y traducción a proteína.
- Reproducción sexual con herencia de Mendel, gemelos y mellizos.
- Niveles tróficos explícitos, pirámide de biomasa, descomponedores, suelo, luz y temperatura (Q10).
- Una base de conocimiento científico real.

## Ideas para adoptar (reimplementadas desde cero)

1. **La cría necesita espacio libre a su alrededor.** Si no lo hay, no nace. Esto da regulación por densidad. *(Prioridad alta: ataca el colapso actual.)*
2. **Tasa de mutación heredable y mutable**, como gen propio. Así evoluciona la evolvabilidad.
3. **Descartar especies con poca población** del registro (menos de 10 individuos acumulados), para que no se inflen las especiaciones.
4. **Reinicio automático** al extinguirse todo, y contador de reinicios.
5. **Cerebro como tabla de decisiones** (ve X → persigue, huye o ignora), como paso intermedio entre las reglas fijas de `bioma.rs` y la red neuronal de `mente.rs`.

## Límites de este escaneo

- Análisis estático: no se observó el programa corriendo, así que el equilibrio que se le atribuye es una inferencia hecha a partir del código.
- Solo se leyeron a fondo los módulos de organismo, células, cerebro y especies. Controladores, renderizado y editor quedaron fuera.
