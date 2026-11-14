# PLAN-007 — Esc cierra la proyección aunque se esté editando una nota

- **Issue:** #34
- **Rama:** `fix/esc-cierra-proyeccion`
- **Estado:** implementado, pendiente de merge

## Objetivo

Que `Esc` cierre la proyección (ventana de audiencia) siempre que esté activa, también
mientras se edita una nota.

## Alcance

- `src/gpui_ui/presenter.rs`: `EscAction::CancelEdit`; nueva precedencia en `esc_action`
  (proyección primero); `dispatch_key` consulta `esc_action` para `Esc` antes de la rama de
  edición. La edición permanece abierta al cerrar la proyección.
- Sin dependencias nuevas.

## Tareas

- [x] 1. Tests (RED): precedencia de `esc_action`; `Esc` con edición + proyección cierra la proyección y deja la edición abierta; `Esc` con edición sin proyección cancela la edición.
- [x] 2. Implementación (GREEN): `EscAction::CancelEdit` + nueva `esc_action` + `dispatch_key`.
- [x] 3. Auditoría `@rust-reviewer`: aprobado; nit del modelo del editor aplicado.
- [x] 4. `cargo fmt` y clippy 1.100 limpios; `cargo test` en verde.
- [ ] 5. Archivar el change OpenSpec al mergear.
