# HANDOFF — Spike GPUI (issue #12)

## Estado
- **Rama:** `feature/gpui-spike` (desde `development`). Sin commitear todavía.
- **Change OpenSpec:** `openspec/changes/gpui-spike/` (válido con `--strict`).
- **Plan:** `plans/PLAN-001-ARCHIVED.md` (completado).
- **Informe:** `docs/gpui-spike.md`.

## Qué se ha hecho
Spike de viabilidad de GPUI en binario aislado `src/bin/gpui_spike.rs` (la app egui NO se tocó):
dos ventanas nativas (presenter + audiencia fullscreen), estado compartido `Entity<Deck>`,
render Typst→`RenderImage` (RGBA premultiplicado → BGRA no premultiplicado), navegación por
botones y teclado (incl. atajos vim `h`/`j`/`k`/`l`/`g`/`G`), toggle de tema, manejo de errores.

**Diseño UI final:** presenter con dos paneles apilados (*Actual* + *Siguiente*, `p_8` de
padding por panel); audiencia a pantalla completa con la imagen a `size_full()`/`Contain` y sin rótulos.

**Verificación visual (usuario, Niri/Wayland): OK** — dos ventanas abiertas, navegación
sincronizada, colores correctos, tema y fullscreen correctos.

## Verificación (real)
- `cargo build --bin gpui_spike` → Finished.
- `cargo test --bin gpui_spike` → 1 test ok (`rgba_to_bgra_unpremultiplies`).
- `cargo build` / `cargo test` completos → verdes (61 + 1 + 2 = 64).
- Arranque en Wayland: `timeout 10s cargo run --bin gpui_spike -- demo.typ` → exit 124 (viva, sin panic).

## Veredicto
Viabilidad técnica: **SÍ**. Recomendación: **NO migrar todavía** (dep no oficial, pre-1.0,
árbol de compilación enorme, pérdida de ecosistema egui). Detalle en `docs/gpui-spike.md`.

## Pendiente
- [ ] Commit + PR de `feature/gpui-spike` → `development` (gitflow, gitmoji).
- [ ] Confirmación visual del render (texto/página) en sesión gráfica.
- [ ] Decidir archivar el change o convertirlo en plan de migración por fases.
- [ ] `openspec archive gpui-spike` si se cierra el spike.

## Notas
- El servicio de subagentes estuvo intermitente (`ECONNREFUSED`); varias escrituras se hicieron
  vía shell como excepción documentada.
