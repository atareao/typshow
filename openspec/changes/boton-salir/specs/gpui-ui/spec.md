## ADDED Requirements

### Requirement: Salida de la aplicación

La app SHALL ofrecer un control explícito de salida en la barra de controles del
presentador y SHALL reconocer el atajo `Ctrl+Q`, ambos terminando la aplicación al
completo (presentador y audiencia). El cierre SHALL ser directo: el usuario MUST NOT
tener que confirmar ni cerrar ninguna ventana manualmente. El atajo MUST funcionar
también mientras hay una edición de nota en curso.

#### Scenario: El botón «Salir» cierra la app

- **GIVEN** la app está abierta con la ventana del presentador
- **WHEN** el usuario pulsa el botón «Salir» de la barra superior (junto a «Tema»)
- **THEN** la aplicación termina por completo, sin confirmación

#### Scenario: El botón «Salir» cierra la app con la proyección activa

- **GIVEN** la proyección está activa y la ventana de audiencia está abierta
- **WHEN** el usuario pulsa el botón «Salir»
- **THEN** se cierran también la ventana de audiencia y la aplicación por completo

#### Scenario: Ctrl+Q cierra la app desde el presentador

- **GIVEN** la ventana del presentador tiene el foco y no hay ninguna edición en curso
- **WHEN** el usuario pulsa `Ctrl+Q`
- **THEN** la aplicación termina por completo

#### Scenario: Ctrl+Q cierra la app durante la edición de una nota

- **GIVEN** el usuario está editando una nota (cuerpo o título) en el presentador
- **WHEN** el usuario pulsa `Ctrl+Q`
- **THEN** la aplicación termina por completo
- **AND** el atajo no se entrega al editor como texto ni se consume como edición
