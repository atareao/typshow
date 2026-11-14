# Design

## Context

El panel de notas parsea el Markdown con `pulldown-cmark` en `Notes::parse_md`
(`src/notes.rs`), que **re-inserta los marcadores** en el texto del elemento: `**` para
`Tag::Strong`, `*` para `Tag::Emphasis` y `` ` `` para `Event::Code`. Después,
`note_element` → `inline_text` (`src/gpui_ui/presenter.rs`) vuelve a partir ese texto con
`inline_runs` y aplica el formato con `StyledText::with_highlights`.

## Goals / Non-Goals

**Goals:**
- Soporte de cursiva con los mismos tokens del tema.

**Non-Goals:**
- Enlaces, imágenes, tablas ni anidamiento de formatos inline.

## Decisions

- **Añadir `InlineStyle::Italic`**, reconocido por `inline_runs` con un único `*…*`, y
  aplicado con `HighlightStyle { font_style: Some(FontStyle::Italic), .. }`.
- **Solo un asterisco, no guion bajo, en `inline_runs`**: el texto que llega ya pasó por
  `parse_md`, que normaliza `_…_` y `*…*` a `*`. Tratar `_` aquí rompería palabras con
  guion bajo (`snake_case`), que pulldown-cmark deja como texto plano. Así, `_cursiva_`
  funciona igual porque llega como `*cursiva*`.
- **Precedencia**: `**` (negrita) se comprueba antes que `*` (cursiva).

## Risks / Trade-offs

- Anidamiento (`***x***`) no se interpreta; se comporta como hoy (sin pánico, marcadores
  literales en el peor caso).
