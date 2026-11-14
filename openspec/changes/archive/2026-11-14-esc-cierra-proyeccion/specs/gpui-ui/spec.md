## MODIFIED Requirements

### Requirement: Toggle de proyección

La app SHALL permitir mostrar u ocultar la ventana de audiencia. Además, `Esc` SHALL cerrar
la proyección siempre que esté activa, con independencia de que haya una edición de notas en
curso.

#### Scenario: Alternar proyección

- **WHEN** el usuario activa/desactiva la proyección
- **THEN** la ventana de audiencia se abre o se cierra en consecuencia

#### Scenario: Esc cierra la proyección sin edición

- **GIVEN** la proyección está activa y no hay ninguna edición en curso
- **WHEN** el usuario pulsa `Esc`
- **THEN** la ventana de audiencia se cierra y la proyección queda inactiva

#### Scenario: Esc cierra la proyección durante la edición

- **GIVEN** la proyección está activa y el usuario está editando una nota
- **WHEN** el usuario pulsa `Esc`
- **THEN** la ventana de audiencia se cierra y la proyección queda inactiva
- **AND** la edición permanece abierta (no se descarta el texto que el usuario escribía)

#### Scenario: Esc sin proyección cancela la edición

- **GIVEN** la proyección no está activa y el usuario está editando una nota
- **WHEN** el usuario pulsa `Esc`
- **THEN** la edición se cancela y no ocurre nada más

#### Scenario: Esc sin proyección ni edición

- **GIVEN** la proyección no está activa y el usuario no está editando
- **WHEN** el usuario pulsa `Esc`
- **THEN** no ocurre nada
