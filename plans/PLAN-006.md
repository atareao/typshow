# PLAN-006 — Renderizar la cursiva en el Markdown de las notas

- **Issue:** #33
- **Rama:** `fix/markdown-cursiva`
- **Estado:** implementado, pendiente de merge

## Objetivo

Que el panel de notas renderice el formato inline completo (negrita, cursiva y código),
sin mostrar los marcadores.

## Alcance

- `src/gpui_ui/presenter.rs`: nueva variante `InlineStyle::Italic`; `inline_runs` reconoce
  `*…*` (tras `**` y `` ` ``); `inline_text` aplica `FontStyle::Italic`.
- `src/notes.rs`: `parse_md` pasa a `pub(crate)` para el test end-to-end (sin cambio de
  comportamiento).
- Sin dependencias nuevas.

## Tareas

- [x] 1. Tests (RED): italic en `inline_runs`, precedencia bold, `snake_case` no es italic, offsets, end-to-end vía `parse_md`.
- [x] 2. Implementación (GREEN): `InlineStyle::Italic` + rama en `inline_runs` + `font_style` en `inline_text`.
- [x] 3. Guarda `!inner.starts_with('*')` para no perder marcadores sin cerrar.
- [x] 4. Auditoría `@rust-reviewer`: aprobado; nit del contrato de la guarda aplicado.
- [x] 5. `cargo fmt` y clippy 1.100 limpios; `cargo test` en verde.
- [ ] 6. Archivar el change OpenSpec al mergear.
