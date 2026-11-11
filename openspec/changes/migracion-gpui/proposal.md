# Proposal

## Why

El spike `gpui-spike` (PR #13) demostró que GPUI cubre los tres pilares de Typshow:
doble ventana nativa, audiencia a pantalla completa y render de páginas Typst, con
estado compartido vía `Entity`. El responsable del proyecto quiere adoptar **GPUI como
framework de UI por defecto**. Este change define la migración de la app de
`egui`/`eframe` a GPUI, **por fases y sin dejar la app inservible** durante el proceso.

## What Changes

- Sustituir la UI `egui`/`eframe` por una UI GPUI, reutilizando `TypstWorld`/pipeline de
  render, la lógica de navegación de `AppState` y la persistencia de notas
  (`Notes`/`NotesDb`).
- **Estrategia de coexistencia**: durante la migración conviven dos binarios —
  `typshow` (egui, actual) y `typshow-gpui` (nuevo)—. Al alcanzar la paridad, GPUI pasa a
  ser el binario por defecto y se retira egui.
- Reproducir en GPUI la funcionalidad actual: ventana presenter (navegación, paneles
  *Actual*+*Siguiente*, panel de notas, abrir fichero, toggles de tema/proyección),
  ventana audiencia fullscreen, atajos de teclado y clic, tema estilo Adwaita e iconos.
- Dependencia `gpui`/`gpui_platform` vía git (rev fijado), ya presente en `Cargo.toml`.

## Capabilities

### New Capabilities
- `gpui-ui`: interfaz de Typshow basada en GPUI (presenter + audiencia sincronizados,
  render de páginas Typst, navegación, panel de notas, tema e integración de sistema).

### Modified Capabilities
<!-- Ninguna: openspec/specs está vacío, no hay specs base que modificar. -->

## Impact

- Nuevos módulos GPUI; `main.rs` pasa a `gpui_platform::application().run(...)`.
- Se reutilizan `render.rs` (`TypstWorld`), `app.rs` (`AppState`) y
  `notes.rs`/`notes_db.rs`.
- `theme.rs` (egui) se reescribe como estilos GPUI; `egui-phosphor` se sustituye.
- Al final de la migración se retiran `egui`/`eframe` y sus dependencias.
- Afecta al empaquetado/release (cambio de dependencias y de binario por defecto).
