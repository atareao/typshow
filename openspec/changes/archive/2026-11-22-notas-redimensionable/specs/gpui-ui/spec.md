## ADDED Requirements

### Requirement: Panel de notas redimensionable

La ventana de presentador SHALL permitir ajustar el **ancho** del panel de notas
arrastrando el asa situada entre la previsualización y el panel. El ancho SHALL quedar
siempre comprendido entre `NOTES_PANEL_MIN_WIDTH` y `NOTES_PANEL_MAX_WIDTH` (inclusive),
y el ancho elegido SHALL **persistir entre sesiones**, restaurándose al arrancar. El
ancho por defecto, cuando no hay preferencia guardada, SHALL ser `NOTES_PANEL_WIDTH`.

#### Scenario: Ajuste por arrastre

- **GIVEN** la ventana de presentador con el panel de notas visible
- **WHEN** el usuario arrastra el asa `handle-notas` hacia la izquierda
- **THEN** el ancho del panel aumenta de forma continua siguiendo el puntero

#### Scenario: Límite mínimo

- **GIVEN** un ancho actual cercano al mínimo
- **WHEN** el usuario arrastra el asa de modo que el ancho quede por debajo del mínimo
- **THEN** el ancho se fija en `NOTES_PANEL_MIN_WIDTH` y no disminuye más

#### Scenario: Límite máximo

- **GIVEN** un ancho actual cercano al máximo
- **WHEN** el usuario arrastra el asa de modo que el ancho supere el máximo
- **THEN** el ancho se fija en `NOTES_PANEL_MAX_WIDTH` y no aumenta más

#### Scenario: Persistencia al soltar

- **GIVEN** un ancho ajustado por arrastre distinto del guardado
- **WHEN** el usuario suelta el asa
- **THEN** el ancho saneado se escribe en la preferencia de ancho de notas

#### Scenario: Restauración al arrancar

- **GIVEN** una preferencia de ancho de notas guardada y válida
- **WHEN** la app arranca
- **THEN** el panel de notas se muestra con ese ancho

#### Scenario: Preferencia inválida o ausente

- **GIVEN** una preferencia de ancho de notas ausente, no numérica o fuera de rango
- **WHEN** la app arranca
- **THEN** el panel de notas usa `NOTES_PANEL_WIDTH` como ancho por defecto

#### Scenario: Feedback de cursor

- **GIVEN** el panel de notas visible
- **WHEN** el puntero se sitúa sobre el asa y durante el arrastre
- **THEN** el cursor indica redimensionado horizontal
