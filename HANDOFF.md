# Handoff — typshow — 2026-11-13 00:25

## Objetivo
Migrar la UI de Typshow de `egui`/`eframe` a `GPUI` (issue #12) por fases. **PR #14 MERGEADO a `development`** (hito 3.12/M4). Quedan las fases finales.

## Estado
- **PR #14** (`feature/gpui-migration` → `development`): **MERGED** (merge commit `91d215b`, 2026-11-12 23:20Z). CI verde (tests, fmt, clippy).
- Hecho y mergeado: Fases 1/3 (doble ventana, render asíncrono, caché LRU), 3.8–3.12/M4 (editor `TextEdit` + `EntityInputHandler`: selección teclado/ratón, IME y portapapeles Ctrl+C/X/V, verificado manualmente en Niri), 2.5 (apertura asíncrona de `.typ`).
- Hecho: perfil dev optimizado en `Cargo.toml` (`[profile.dev.package."*"] opt-level = 3`): el render de página en debug baja de ~0.7-0.9 s a ~20 ms.
- Hecho: CI arreglada (deps de sistema de GPUI + caché de Cargo) y cero warnings de clippy.
- `development` local sincronizado con origin; working tree limpio.

## Pendiente (PLAN-002 sigue EN CURSO)
- **2.2** desacoplar `Notes` de egui.
- **Fases 5-7**: tema/iconos, GPUI por defecto, retirada de `egui`/`eframe` y cierre.
- Decidir qué hacer con el **PR #13** (`feature/gpui-spike`), que sigue ABIERTO.
- Decidir si se borra la rama `feature/gpui-migration` (ya mergeada) o se reutiliza para las fases 5-7.

## Riesgos y notas
- Para el binario definitivo usar `cargo build --release`.
- El clippy del runner es más nuevo que el local: verificar lints nuevos con `rustup run 1.99.1 cargo clippy --all-targets -- -D warnings`.
