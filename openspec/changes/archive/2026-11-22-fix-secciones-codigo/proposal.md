# Proposal

## Why

`script_section_bodies` (feature #52, `src/notes_db.rs`) reconoce una sección `## …` con
`line.trim_start().starts_with("## ")`. Los guiones que **embeben Markdown en bloques de
código** (p. ej. la escaleta del 807, con `    ## Cómo usarla` dentro de un fence
```markdown) hacen que esas líneas cuenten como secciones. Entonces
`embed_script_into_typ` abandona el **modo por rango** (#59) y usa la vía `##` 1:1,
escribiendo notas erróneas y dejando sin guion al resto de secciones. Issue #63.

## What Changes

- `script_section_bodies` pasa a ser **consciente de fences y de la sintaxis ATX**:
  - ignora los `##` que están dentro de un bloque de código cercado (``` … ```);
  - solo acepta cabeceras con indentación ≤ 3 espacios (4+ = código indentado, no cabecera).
- Con ello, una escaleta con bloques de código vuelve al **modo por rango** y el modo `##`
  queda robusto ante guiones que embeben Markdown.

## Capabilities

### Modified Capabilities

- `notas`: el requisito «Conversión del guion Markdown a bloques embebidos» precisa que la
  detección de secciones `## …` ignore las que están dentro de bloques de código y las que
  llevan indentación de código (≥ 4 espacios).

## Impact

- `src/notes_db.rs`: `script_section_bodies` (+ helper de fences). Sin cambios de esquema,
  UI ni firma del CLI.
- Re-ejecución de `guion-embed` sobre las escaletas afectadas (dato externo).
