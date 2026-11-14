# Tasks

## 1. Tests (RED)

- [ ] 1.1 `inline_runs`: `*cursiva*` → tramo `Italic` (y que `**negrita**` sigue siendo `Bold`).
- [ ] 1.2 `inline_layout`: los `spans` incluyen el rango `Italic`.
- [ ] 1.3 `snake_case` (`foo_bar_baz`) no se interpreta como cursiva.
- [ ] 1.4 (end-to-end) `Notes::parse_md("_cursiva_")` + `inline_layout` da un tramo `Italic`.

## 2. Implementación (GREEN)

- [ ] 2.1 `InlineStyle::Italic`; reconocer `*…*` en `inline_runs` (tras `**` y `` ` ``); aplicar `font_style: Italic` en `inline_text`. `cargo test` en verde.

## 3. Cierre (REFACTOR)

- [ ] 3.1 `cargo fmt --all -- --check` y clippy 1.100 (`-D warnings`) limpios.
- [ ] 3.2 Archivar el change (`openspec archive`) al mergear.
