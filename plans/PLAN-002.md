# PLAN-002 — Migración de la UI a GPUI

- **Issue:** #12 (Migrar la interfaz a GPUI) — https://github.com/atareao/typshow/issues/12
- **Tipo:** Migración de framework de UI (por fases)
- **Change OpenSpec:** `openspec/changes/migracion-gpui/`
- **Branch:** `feature/gpui-migration`
- **Estado:** EN CURSO (Fases 1-4 completadas; M3 y M4 completados — render/apertura asíncrona, `rfd` no bloqueante, editor de notas con `EntityInputHandler`: selección teclado/ratón, IME y portapapeles. Pendiente: 2.2 desacoplar `Notes` de egui, fases 5-7). PR #14 listo para revisión.
- **Precedente:** spike `gpui-spike` (PR #13); veredicto de viabilidad en `docs/gpui-spike.md`

## Objetivo

Convertir **GPUI en el framework de UI por defecto** de Typshow, sustituyendo
`egui`/`eframe` con **paridad funcional** y sin dejar la app inservible durante el proceso.

## Estado actual (a migrar)

| Fichero | LOC | Rol | Reutilizable |
|---|---|---|---|
| `src/main.rs` | 138 | Arranque eframe, `setup_fonts`, CLI | Se reescribe (GPUI) |
| `src/presenter.rs` | 553 | `TypshowApp`: presenter egui (nav, notas, actual+siguiente, viewport) | Se reescribe |
| `src/fullscreen.rs` | 124 | Audiencia egui | Se reescribe |
| `src/app.rs` | 179 | `AppState` + `SharedState` (navegación) | Sí (dominio) |
| `src/render.rs` | 307 | `TypstWorld` + `Renderer` (typst → pixmap → textura) | `TypstWorld` sí; `get_page` (egui) se sustituye |
| `src/theme.rs` | 137 | Overrides Adwaita/GTK (egui) | Se reescribe |
| `src/notes.rs` | 519 | Notas + Markdown (integrado en UI) | Lógica sí; `draw` (egui) se reescribe |
| `src/notes_db.rs` | 751 | SQLite de notas (WAL) | Sí (backend) |
| `src/bin/gpui_spike.rs` | 434 | Prototipo GPUI (spike) | Base del núcleo |

## Estrategia

**Coexistencia por binarios**: durante la migración conviven `typshow` (egui) y
`typshow-gpui` (nuevo). Al alcanzar la paridad, GPUI pasa a ser `typshow` y se retira egui.
Se reutiliza el dominio (`AppState`, `Typ, `Notes`, `NotesDb`) y solo se reescribe
la capa de UI.

## Fases

1. **Preparación y estructura** — `src/gpui/` + binario `typshow-gpui` coexistiendo.
2. **Núcleo** — `Deck` con `TypstWorld` y caché de `RenderImage`; reutilizar `AppState`/`Notes`/`NotesDb`; manejo de errores.
3. **Presenter** — paneles Actual+Siguiente, barra de navegación, abrir fichero (`rfd`), toggle de proyección, toggle de tema, panel de notas, atajos y clic.
4. **Audiencia** — fullscreen, sincronización, atajos/clic/Esc, monitor secundario (Niri).
5. **Tema e iconos** — estilo Adwaita en GPUI; sustituir `egui-phosphor`.
6. **Framework por defecto** — `typshow` = GPUI; retirar `egui`/`eframe`/`egui-phosphor`.
7. **Cierre** — `docs/migracion-gpui.md`, verificación (`cargo build`/`test`/`clippy`).

## Fuera de alcance

- Funcionalidad nueva más allá de la paridad.
- Cambios en el pipeline de render Typst (se reutiliza).

## Criterio de éxito

GPUI es el framework por defecto de `typshow`, con **paridad funcional** completa (doble
ventana sincronizada, render, navegación, notas, tema e iconos), `cargo build`/`test`/`clippy`
en verde y egui retirado.

## Riesgos

- GPUI pre-1.0 con dependencia git (rev fijado) → actualizaciones manuales; posibles breaking changes.
- Compilación pesada del árbol de Zed → caché de CI; loop de desarrollo más lento.
- Sin `egui-phosphor` → embeber Phosphor (TTF) o migrar a SVG.
- Fullscreen / monitor secundario en Wayland/Niri → validar; conservar `niri msg` si aplica.
- Regresión si se cambia el binario por defecto antes de la paridad → mitigado por la coexistencia.