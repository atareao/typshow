# Release 0.2.1

Fecha: 2026-11-13 · Tipo: patch · Tag: `v0.2.1`

## Qué incluye

- 🐛 **fix(gpui)**: corrección del bug de color al rasterizar. `render_page` llamaba
  `gpui::swap_rgba_pa_to_bgra` con el buffer entero, pero esa función opera sobre un solo píxel:
  solo se convertía el primero y el resto quedaba RGBA leído como BGRA → R y B intercambiados
  (el amarillo se veía cian). Ahora se itera por píxel con `as_chunks_mut::<4>()`.
  Issue #28 · PR #27.
- ♻️ **refactor(gpui)**: `as_chunks_mut` para satisfacer `clippy::chunks_exact_to_as_chunks` (nuevo en 1.100).
- 👷 **ci(release)**: `release.yml` disparable con `actions: write`; publish a crates.io no bloqueante;
  `Cargo.lock` sincronizado en cada bump; bump breaking en 0.x → `--minor`.
- 📝 **docs**: `release-0.2.0`, `fix-canales-color-rasterizado`, planes y handoff; change OpenSpec archivado.

## Verificación

- Tests: 168 lib + 2 integración en verde; `cargo fmt` y clippy 1.100.0 (`-D warnings`) limpios.
- Verificación visual en Niri: el logo (Tux) de una presentación muestra el amarillo correcto (antes cian).

## Artefactos

- `typshow-x86_64-unknown-linux-gnu.tar.gz`
- `typshow-aarch64-unknown-linux-gnu.tar.gz`

## Limitaciones conocidas

- crates.io: no publicable mientras `gpui`/`gpui_platform` sean dependencias git de Zed. El job `publish` es no bloqueante.

## Follow-ups

- Verificación visual del resto de elementos (tema claro/oscuro, los 17 iconos, markdown de las notas, `Esc` cierra la proyección).
- Editor: undo/redo, navegación por palabras, panel de notas redimensionable.
