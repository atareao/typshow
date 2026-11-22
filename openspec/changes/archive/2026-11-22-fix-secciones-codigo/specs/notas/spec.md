## MODIFIED Requirements

### Requirement: Conversión del guion Markdown a bloques embebidos

typshow SHALL ofrecer un subcomando que convierta un guion Markdown en bloques embebidos
dentro de un `.typ`. Con secciones `## …`, cada sección SHALL asociarse a un heading de
slide por orden (1:1). La detección de secciones `## …` SHALL ignorar las que aparezcan
dentro de un **bloque de código cercado** (``` … ```) y las que tengan **indentación de
código** (4 o más espacios), de modo que el Markdown embebido en un guion no se confunda
con cabeceras. Sin secciones `##` pero con bloques por tramos de tiempo, SHALL aplicarse
el **mapeo por rango** a las secciones `=` (requisito «Mapeo por rango de guiones de grano
grueso»). La conversión MUST ser idempotente: reejecutarla sobre el mismo `.typ` MUST NOT
duplicar bloques.

#### Scenario: Conversión por orden de secciones

- **GIVEN** un `guion.md` con N secciones `## …` y un `.typ` con N headings de slide
- **WHEN** se ejecuta el subcomando
- **THEN** cada sección se escribe como bloque embebido tras su heading, en orden

#### Scenario: Conversión idempotente

- **GIVEN** un `.typ` que ya contiene los bloques generados
- **WHEN** se vuelve a ejecutar el subcomando
- **THEN** los bloques existentes se actualizan y no se duplican

#### Scenario: Secciones dentro de bloques de código se ignoran

- **GIVEN** un guion con cabeceras `## …` reales y, además, líneas `## …` dentro de un bloque de código cercado
- **WHEN** se detectan las secciones del guion
- **THEN** las líneas del bloque de código NO cuentan como secciones

#### Scenario: Cabecera con indentación de código no es sección

- **GIVEN** un guion con una línea `    ## …` (4 espacios) fuera de un fence
- **WHEN** se detectan las secciones del guion
- **THEN** esa línea NO cuenta como sección

#### Scenario: Un guion con bloques de código mantiene el modo por rango

- **GIVEN** una escaleta con bloques por tramos de tiempo y Markdown embebido en un bloque de código
- **WHEN** se ejecuta el subcomando
- **THEN** se aplica el mapeo por rango (no la vía `##`) y se escribe el guion completo
