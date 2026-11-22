# Mapeo por rango de guiones de grano grueso

Issue #59 · Plan `plans/PLAN-012.md` · Change OpenSpec `openspec/changes/mapeo-rango-guion/`.

## Qué es

Extensión del guion embebido (#52) para que los **guiones de grano grueso** —los que no
usan secciones `##`, sino **bloques por tramos de tiempo** (`0:00 – 2:00  🎬 INTRO`,
`2:00 – 6:30  📋 BLOQUE 1 — …`, …)— también viajen dentro del `.typ` como notas del
presentador. De ~34 episodios, solo 8 casaban 1:1 con el mapeo por secciones `##`; este
modo cubre el resto sin cambiar cómo se invoca el pipeline.

## Ficheros implicados

- `src/notes_db.rs` — todo el mapeo por rango:
  - `timeline_regions` / `script_timeline_blocks` — trocean el guion en bloques.
  - `section_page_ranges` / `fixed_sections` — rangos de página por sección `=`.
  - `map_blocks_to_sections` — mapeo ordinal/proporcional (pura).
  - `embed_script_ranges_into_typ` — escritura replicada en cada página del rango.
  - `embed_script_into_typ` / `guion_embed_files` — fallback automático.
- `src/main.rs` — subcomando `guion-embed` (firma **sin cambios**).
- `tests/pipeline_test.rs` — `guion_rango_no_aparece_en_el_pdf`.

## Cómo funciona

1. **Troceado**: `script_timeline_blocks(guion)` separa por líneas `─` (≥10) y considera
   bloque cada región cuya primera línea no vacía **empieza** por un tramo `H:MM – H:MM`.
   El cuerpo del bloque es la cabecera **más** las regiones siguientes hasta la próxima
   cabecera. Se ignora el preámbulo (título del episodio).
2. **Rangos**: `section_page_ranges(typ)` da, por cada sección `=` del `.typ`, el rango de
   páginas que cubre (su página hasta la anterior a la siguiente `=`), reproduciendo el
   reparto posicional de `build_page_seeds`. La portada (`title:`) no es sección.
3. **Mapeo**: `map_blocks_to_sections(b, s)` asigna bloques a secciones. Identidad si
   `b == s`; reparto proporcional monótono si `b < s`; **agregación** (varios bloques en
   una sección) si `b > s`.
4. **Escritura**: el Markdown de cada bloque se escribe como bloque embebido
   (`// >>> typshow:notes … :end`) en **cada página** de su rango, de modo que el
   presentador ve los puntos del tramo mientras lo presenta. Idempotente.
5. **Fallback** en `guion-embed`: con secciones `##` ⇒ comportamiento previo (1:1 por
   orden); sin `##` y con tramos ⇒ mapeo por rango; ninguna de las dos ⇒ 0 bloques +
   `tracing::warn!`, sin abortar. Sin secciones `=` en el `.typ` ⇒ `warn!` y no escribe.

## Uso

```bash
# Igual que siempre: el modo se elige solo según la forma del guion
typshow guion-embed guion-episodio.md episodio.typ
```

El pipeline del podcast no necesita cambios de invocación.

## Variables / configuración

- Límites de detección: separador de línea `─` con **≥10** repeticiones; cabecera con
  marca de tiempo `H:MM` separada por ` – `, ` - `, `—` o `~`.
- Nada configurable por entorno; sin cambios de esquema SQLite.

## Cómo habilitar / desplegar

Instalar el binario (`cargo install --path .` o `~/.cargo/bin/typshow`) y ejecutar
`guion-embed`. No hay migración ni flag.

## Decisiones

- El modo se elige por la **forma** del guion (presencia de `##` vs. bloques por tramos),
  no por un flag: mantiene el pipeline determinista y sin cambios.
- El texto del bloque se **replica** en todas las páginas de su rango (el presentador ve
  los puntos durante todo el tramo), en vez de repartirlo entre páginas.
- El reparto cuando el número de bloques y de secciones difiere es **monótono y cubriendo
  todas las secciones** para que ninguna quede sin guion.

## Limitaciones conocidas

- **Cola sin marca horaria**: el material de producción posterior al último bloque (p. ej.
  `CHECKLIST` del 812) se anexa al cuerpo de la **última** sección, porque el guion no
  trae una marca explícita de fin. Inofensivo (viaja como nota del presentador).
- **Hora en una línea de cuerpo**: una línea intermedia con un rango horario textual
  (p. ej. «Repaso: 0:00 – 2:00») puede truncar el bloque y descartarse.
- Solo se consideran secciones `=` (nivel 1); las `==` de dentro caen en el rango de su `=`.

## Follow-ups

- Verificación sobre el episodio 812 real: hecha (guion visible como notas, invisible en
  PDF); cubierta a nivel de píxel por `tests/pipeline_test.rs`.
- Si se quiere recortar la cola de producción, añadir al guion una marca explícita de fin
  (requiere tocar el generador/pipeline del guion).
