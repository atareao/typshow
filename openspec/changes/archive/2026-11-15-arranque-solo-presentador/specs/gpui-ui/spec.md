## MODIFIED Requirements

### Requirement: Toggle de proyección

La app SHALL permitir mostrar u ocultar la ventana de audiencia. La proyección
SHALL arrancar inactiva, de modo que un arranque sin documento abra únicamente la
ventana del presentador. Además, `Esc` SHALL cerrar la proyección siempre que esté
activa, con independencia de que haya una edición de notas en curso.

#### Scenario: Alternar proyección

- **WHEN** el usuario activa/desactiva la proyección
- **THEN** la ventana de audiencia se abre o se cierra en consecuencia

#### Scenario: Arranque sin documento solo abre el presentador

- **GIVEN** la app arranca sin documento (p. ej. lanzada sin argumento de fichero)
- **WHEN** la ventana del presentador se muestra
- **THEN** la proyección está inactiva y NO se abre la ventana de audiencia

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

### Requirement: Tema claro/oscuro estilo Adwaita

La app SHALL ofrecer un tema claro y uno oscuro con estética GTK4/Adwaita, y SHALL
recordar la última preferencia de tema elegida por el usuario entre sesiones,
también cuando la app se arranca sin documento.

#### Scenario: Alternar tema

- **WHEN** el usuario alterna el tema
- **THEN** los colores de la interfaz cambian entre claro y oscuro de forma coherente

#### Scenario: Persistencia del tema

- **WHEN** el usuario elige un tema (claro u oscuro) y vuelve a abrir la app
- **THEN** la app arranca con ese mismo tema

#### Scenario: Persistencia del tema sin documento

- **GIVEN** la app se arranca sin documento
- **WHEN** el usuario alterna el tema y vuelve a abrir la app sin documento
- **THEN** la app arranca con la preferencia de tema guardada

#### Scenario: Sin preferencia guardada

- **WHEN** no hay ninguna preferencia de tema guardada (o el fichero no es válido)
- **THEN** la app arranca en el tema claro por defecto

## ADDED Requirements

### Requirement: Cierre de la aplicación con el presentador

Cerrar la ventana del presentador SHALL cerrar toda la aplicación, incluida la
ventana de audiencia si estuviera abierta. El usuario MUST NOT tener que cerrar
cada ventana manualmente.

#### Scenario: Cerrar el presentador cierra la app

- **GIVEN** la app está abierta con la ventana del presentador
- **WHEN** el usuario cierra la ventana del presentador
- **THEN** la aplicación termina por completo

#### Scenario: Cerrar el presentador con la proyección activa

- **GIVEN** la proyección está activa y la ventana de audiencia está abierta
- **WHEN** el usuario cierra la ventana del presentador
- **THEN** se cierran también la ventana de audiencia y la aplicación por completo
