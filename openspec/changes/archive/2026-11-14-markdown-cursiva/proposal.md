# Proposal

## Why

El panel de notas renderiza el Markdown inline de forma incompleta: la **cursiva** (`*…*` /
`_…_`) no se reconoce y se muestran los asteriscos literales. Solo funcionan la negrita y el
código.

## What Changes

- Añadir soporte de **cursiva** al renderizado inline de las notas (junto a negrita y código).
- Garantizar que el formato inline se aplica de forma uniforme en títulos, párrafos y
  elementos de lista.
- (Nota) La negrita y el código ya funcionan; se añaden tests que lo fijan para evitar
  regresiones.

## Capabilities

### New Capabilities

<!-- Ninguna -->

### Modified Capabilities

- `gpui-ui`: el requirement "Panel de notas" pasa a exigir el formato Markdown inline completo
  (negrita, cursiva y código) sin los marcadores visibles.

## Impact

- `src/gpui_ui/presenter.rs` (`InlineStyle`, `inline_runs`, `inline_text`).
- Sin cambios de API pública ni de dependencias.
