## ADDED Requirements

### Requirement: Mapeo por rango de guiones de grano grueso

Cuando un guion no contiene secciones `## …` pero sí **bloques por tramos de tiempo**
(delimitados por separadores de línea `─` y una cabecera con marca de tiempo y título,
p. ej. `2:00 – 6:30  📋 BLOQUE 1 — …`), typshow SHALL trocearlo en esos bloques en orden
e ignorar el preámbulo anterior al primer bloque y el material posterior al último. Los
bloques SHALL mapearse a las **secciones `=` del `.typ`**, donde cada sección cubre un
**rango de páginas** (su página y las siguientes hasta la anterior a la siguiente `=`).
El Markdown del bloque SHALL escribirse como bloque embebido en **cada página** de su
rango. El mapeo SHALL ser **ordinal** (n-ésimo bloque ↔ n-ésima sección `=`); cuando el
número de bloques y de secciones difiera, SHALL repartirse **proporcionalmente** de forma
monótona y cubriendo todas las secciones. La conversión MUST ser idempotente y el bloque
MUST NOT alterar el renderizado del `.typ`.

#### Scenario: Detección de bloques por tramos de tiempo

- **GIVEN** un guion con cabeceras `0:00 – 2:00  🎬 INTRO`, `2:00 – 6:30  📋 BLOQUE 1 — …`
  separadas por líneas `─`
- **WHEN** typshow lee el guion
- **THEN** obtiene los bloques en orden, cada uno con su cuerpo, e ignora el preámbulo
  anterior al primer bloque

#### Scenario: Mapeo ordinal a secciones `=`

- **GIVEN** un guion con B bloques y un `.typ` con S secciones `=` donde B = S
- **WHEN** se ejecuta la conversión por rango
- **THEN** el bloque n se escribe como bloque embebido en todas las páginas de la sección `=` n

#### Scenario: Reparto proporcional

- **GIVEN** un guion con B bloques y un `.typ` con S secciones `=` donde B ≠ S
- **WHEN** se ejecuta la conversión por rango
- **THEN** cada bloque se asigna a una sección por reparto proporcional, de forma monótona
  y cubriendo todas las secciones

#### Scenario: Replicación en cada página del rango

- **GIVEN** una sección `=` que cubre las páginas p…q
- **WHEN** se escribe el bloque de esa sección
- **THEN** el Markdown del bloque aparece embebido en cada una de las páginas p…q

#### Scenario: Fallback a secciones `##`

- **GIVEN** un guion con secciones `## …`
- **WHEN** se ejecuta la conversión
- **THEN** se aplica el comportamiento previo (mapeo 1:1 por orden) y no el mapeo por rango

#### Scenario: Guion sin bloques reconocibles

- **GIVEN** un guion sin secciones `##` ni cabeceras de tramos válidas
- **WHEN** se ejecuta la conversión
- **THEN** no se escribe ningún bloque y se registra un aviso, sin abortar

#### Scenario: El guion de grano grueso no se renderiza

- **GIVEN** un `.typ` con bloques generados por mapeo por rango
- **WHEN** Typst compila el documento a PDF
- **THEN** ninguna línea del guion aparece en el PDF ni en la ventana de audiencia

## MODIFIED Requirements

### Requirement: Conversión del guion Markdown a bloques embebidos

typshow SHALL ofrecer un subcomando que convierta un guion Markdown en bloques embebidos
dentro de un `.typ`. Con secciones `## …`, cada sección SHALL asociarse a un heading de
slide por orden (1:1). Sin secciones `##` pero con bloques por tramos de tiempo, SHALL
aplicarse el **mapeo por rango** a las secciones `=` (requisito «Mapeo por rango de
guiones de grano grueso»). La conversión MUST ser idempotente: reejecutarla sobre el mismo
`.typ` MUST NOT duplicar bloques.

#### Scenario: Conversión por orden de secciones

- **GIVEN** un `guion.md` con N secciones `## …` y un `.typ` con N headings de slide
- **WHEN** se ejecuta el subcomando
- **THEN** cada sección se escribe como bloque embebido tras su heading, en orden

#### Scenario: Conversión idempotente

- **GIVEN** un `.typ` que ya contiene los bloques generados
- **WHEN** se vuelve a ejecutar el subcomando
- **THEN** los bloques existentes se actualizan y no se duplican
