## MODIFIED Requirements

### Requirement: Tema claro/oscuro estilo Adwaita

La app SHALL ofrecer un tema claro y uno oscuro con estética GTK4/Adwaita, y SHALL
recordar la última preferencia de tema elegida por el usuario entre sesiones.

#### Scenario: Alternar tema

- **WHEN** el usuario alterna el tema
- **THEN** los colores de la interfaz cambian entre claro y oscuro de forma coherente

#### Scenario: Persistencia del tema

- **WHEN** el usuario elige un tema (claro u oscuro) y vuelve a abrir la app
- **THEN** la app arranca con ese mismo tema

#### Scenario: Sin preferencia guardada

- **WHEN** no hay ninguna preferencia de tema guardada (o el fichero no es válido)
- **THEN** la app arranca en el tema claro por defecto
