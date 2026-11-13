## MODIFIED Requirements

### Requirement: Render de páginas Typst

La app SHALL compilar el documento Typst y renderizar cada página como imagen,
preservando los colores: cada píxel SHALL rasterizarse con los canales correctos
(convertidos al formato BGRA que espera GPUI), sin intercambiar rojo y azul.

#### Scenario: Página renderizada

- **WHEN** el documento Typst compila correctamente
- **THEN** la página actual se muestra como imagen en las ventanas

#### Scenario: Colores preservados en todo píxel

- **WHEN** una página contiene una forma de color plano (p. ej. amarillo)
- **THEN** todos sus píxeles se rasterizan con el color correcto (el amarillo no se
  convierte en cian)

#### Scenario: Error de compilación

- **WHEN** el documento Typst no compila
- **THEN** la app muestra un mensaje de error en lugar de una página en blanco
