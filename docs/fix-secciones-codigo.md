# Detección de secciones `##` fuera de bloques de código

Issue #63 · Plan `plans/PLAN-013.md` · Change OpenSpec `openspec/changes/fix-secciones-codigo/`.

## Qué es

Corrige la detección de secciones `## …` del guion para que el **Markdown embebido en
bloques de código** no se confunda con cabeceras. Es la pieza que faltaba para que el
**mapeo por rango** (#59) se aplique a las escaletas que embeben Markdown.

## Ficheros implicados

- `src/notes_db.rs` — `script_section_bodies` y sus helpers nuevos: `is_section_heading`,
  `indent_width`, `fence_marker`, `fenced_line_flags`.
- `tests/` (unit tests en `notes_db.rs`) — `seccion_dentro_de_fence_ignorada`,
  `cabecera_indentada_ignorada`, `secciones_reales_intactas`,
  `tramos_con_markdown_embebido_usa_rango`, `fences_mas_largos_y_tildes`.

## Cómo funciona

`script_section_bodies` recorre las líneas junto a `fenced_line_flags(markdown)` (un flag
por línea que marca si está dentro de un bloque de código cercado). Una línea es cabecera
solo si:

1. **no** está dentro de un fence (``` o ~~~, cerrado por otro fence del mismo carácter y
   longitud ≥, sin info-string al cerrar), y
2. pasa `is_section_heading`: empieza por `## ` o es exactamente `##`, con **indentación
   ≤ 3 espacios** (4+ espacios = código indentado en Markdown, no cabecera).

Antes bastaba `trim_start().starts_with("## ")`, lo que hacía que líneas como
`    ## Cómo usarla` dentro de un ```markdown contaran como secciones.

## Efecto

- Una escaleta con bloques de código vuelve al **modo por rango** (en vez de desviarse a la
  vía `##`): p. ej. la 807 pasa de 2 a 7 bloques.
- El modo `##` legítimo (cabeceras reales a columna 0) queda **intacto**.
- Idempotencia y no-render en PDF sin cambios.

## Variables / configuración

Ninguna; sin cambios de esquema, UI ni firma del CLI.

## Cómo habilitar / desplegar

Instalar el binario (`cargo install --path .`) y ejecutar `typshow guion-embed`. Re-ejecutar
sobre las escaletas que embeben Markdown para corregir sus notas.

## Decisiones

- La detección se hace **consciente de fences** en lugar de un heurístico de conteo: es la
  semántica correcta de Markdown.
- Se exige indentación ≤ 3: 4+ espacios es un bloque de código indentado, no una cabecera.

## Limitaciones / follow-ups (nits de auditoría, no bloqueantes)

- Un tabulador de indentación cuenta como 1 espacio (`\t## …` se trataría como sección); el
  spec habla solo de espacios.
- `is_section_heading` acepta `##` desnudo (sin espacio), ampliando la semántica previa;
  ambas formas son válidas en Markdown.
