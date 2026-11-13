# Proposal

## Why

Typshow depende hoy de `egui`/`eframe` para su UI de doble ventana. El issue #12 pide evaluar GPUI (el framework de Zed) como alternativa. Antes de comprometer una migración completa, necesitamos un prototipo que valide multi-ventana, render de páginas Typst y sincronización de estado en GPUI.

## What Changes

- Añadir un binario prototipo aislado (`src/bin/gpui_spike.rs`) que use GPUI sin alterar la app egui existente.
- Dependencia de GPUI vía git desde `zed-industries/zed` (con `rev` fijado).
- Abrir dos ventanas GPUI nativas: presenter (windowed) y audiencia (fullscreen).
- Compartir estado (página actual, total, tema) entre ventanas mediante `Entity`.
- Renderizar páginas Typst a `RenderImage` y mostrarlas con `img()`.
- Navegación Start/Prev/Next/End con sincronización entre ventanas.
- Entregar informe de viabilidad en `docs/gpui-spike.md`.
- **NO** se elimina `egui` en este change.

## Capabilities

### New Capabilities
- `gpui-prototype`: prototipo de UI GPUI con doble ventana sincronizada y render de páginas Typst, usado para evaluar la viabilidad de migrar Typshow.

### Modified Capabilities
<!-- Ninguna -->

## Impact

- Nuevo binario `src/bin/gpui_spike.rs`; `Cargo.toml` (dependencias `gpui`/`gpui_platform` vía git); `docs/gpui-spike.md`.
- No afecta a la app egui actual ni al empaquetado/release.