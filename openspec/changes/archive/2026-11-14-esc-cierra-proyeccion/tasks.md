# Tasks

## 1. Tests (RED)

- [ ] 1.1 `esc_action`: `(true, true)` → `CloseProjection`; `(true, false)` → `CancelEdit`; `(false, true)` → `CloseProjection`; `(false, false)` → `None`.
- [ ] 1.2 `dispatch_key` (con editor activo y proyección activa): `Esc` cierra la proyección y **no** cancela la edición (la edición sigue abierta).
- [ ] 1.3 `dispatch_key` (con editor activo y proyección inactiva): `Esc` cancela la edición.

## 2. Implementación (GREEN)

- [ ] 2.1 `EscAction::CancelEdit`; nueva `esc_action`; `dispatch_key` usa `esc_action` para `Esc` antes de la rama de edición. `cargo test` en verde.

## 3. Cierre (REFACTOR)

- [ ] 3.1 `cargo fmt --all -- --check` y clippy 1.100 (`-D warnings`) limpios.
- [ ] 3.2 Archivar el change (`openspec archive`) al mergear.
