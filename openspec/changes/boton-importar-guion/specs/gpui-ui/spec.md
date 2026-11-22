## ADDED Requirements

### Requirement: Botón «Importar guion» en el panel de notas

El panel de notas SHALL ofrecer un botón **«Importar guion»**, visible cuando hay un
documento `.typ` cargado. Al pulsarlo SHALL abrirse un diálogo de fichero filtrado a `.md`
para elegir el guion; al elegir uno SHALL importarse en el `.typ` abierto (embebido + refresco
de notas) y SHALL mostrarse un mensaje con el resultado. Sin documento cargado, el botón
MUST NOT abrir el diálogo y SHALL mostrarse un error. Mientras haya un diálogo en curso
MUST NOT abrirse otro (sin reentradas).

#### Scenario: Abrir el diálogo de guion

- **GIVEN** un `.typ` cargado
- **WHEN** el usuario pulsa «Importar guion»
- **THEN** se abre un diálogo de fichero filtrado a `.md`

#### Scenario: Importar el guion elegido

- **GIVEN** el diálogo de guion abierto
- **WHEN** el usuario elige un `.md`
- **THEN** el guion se embebe en el `.typ` abierto y el panel de notas se refresca con el resultado

#### Scenario: Sin documento cargado

- **GIVEN** que no hay ningún `.typ` cargado
- **WHEN** el usuario intenta importar un guion
- **THEN** no se abre el diálogo y se muestra un error

#### Scenario: Sin reentradas

- **GIVEN** un diálogo de guion en curso
- **WHEN** el usuario vuelve a pulsar «Importar guion»
- **THEN** no se abre un segundo diálogo

#### Scenario: Feedback del resultado

- **GIVEN** un guion importado con éxito
- **WHEN** termina la importación
- **THEN** se muestra el número de bloques importados
