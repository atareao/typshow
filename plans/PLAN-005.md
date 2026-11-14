# PLAN-005 — Persistir la preferencia de tema entre sesiones

- **Issue:** #35
- **Rama:** `feat/persistir-tema`
- **Estado:** implementado, pendiente de merge

## Objetivo

Que la app recuerde si el usuario eligió tema claro u oscuro y arranque con él la próxima vez.

## Alcance

- Nuevo módulo `src/gpui_ui/config.rs`: resuelve `$XDG_CONFIG_HOME/typshow/theme` (respaldo `$HOME/.config/typshow/theme`) y lee/escribe `dark`/`light`.
- `src/gpui_ui/deck.rs`: `Deck` gana `theme_path: Option<PathBuf>`, carga el tema en `load` (inyectable en tests con `load_with`) y lo persiste en `toggle_theme`.
- Sin dependencias nuevas.

## Tareas

- [x] 1. Tests de `config` (round-trip, inexistente→None, inválido→None, crea directorio padre).
- [x] 2. Módulo `config` + registro en `mod.rs`.
- [x] 3. Tests de `Deck` (persistencia+recarga; sin ruta no escribe) e implementación.
- [x] 4. Auditoría `@rust-reviewer`: hallazgo (test no determinista) corregido.
- [x] 5. `cargo fmt` y clippy 1.100 limpios; `cargo test` 174 lib + 2 integración en verde.
- [ ] 6. Archivar el change OpenSpec al mergear.
