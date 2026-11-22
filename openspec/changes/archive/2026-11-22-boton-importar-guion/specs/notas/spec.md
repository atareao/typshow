## ADDED Requirements

### Requirement: Importación del guion desde la aplicación

La aplicación SHALL poder importar un guion Markdown al `.typ` cargado ejecutando el mismo
embebido que el subcomando `guion-embed` (modo por rango o por secciones `##`): leer el
guion, escribir los bloques embebidos en el fichero del `.typ`, actualizar la fuente en
memoria y **reimportar** las notas. La reimportación SHALL ser la **selectiva por hash**:
refresca las notas no editadas y **preserva** las editadas a mano. Si el guion no tiene
secciones `##` ni bloques por tramos, MUST NOT escribirse nada y SHALL informarse del aviso.
La importación MUST ser idempotente: reimportar el mismo guion sobre el mismo `.typ` no
duplica bloques.

#### Scenario: Importar un guion de grano grueso

- **GIVEN** un `.typ` cargado y un guion con bloques por tramos de tiempo
- **WHEN** se importa el guion
- **THEN** el `.typ` se actualiza con los bloques y las notas del presentador se reimportan

#### Scenario: Se preservan las notas editadas a mano

- **GIVEN** un `.typ` con notas, algunas editadas a mano
- **WHEN** se importa un guion
- **THEN** las notas editadas se preservan y las no editadas se refrescan

#### Scenario: Guion sin secciones ni tramos

- **GIVEN** un guion sin secciones `##` ni cabeceras de tramos válidas
- **WHEN** se importa
- **THEN** no se escribe ningún bloque y se informa del aviso, sin abortar

#### Scenario: Importación idempotente

- **GIVEN** un `.typ` que ya contiene los bloques generados por el guion
- **WHEN** se vuelve a importar el mismo guion
- **THEN** los bloques se actualizan y no se duplican
