# Spec Delta

## Purpose

Interfaz de Typshow basada en GPUI que sustituye a egui: ventana de presentador y
ventana de audiencia sincronizadas, render de páginas Typst, navegación, panel de notas,
tema e integración con el sistema.

## ADDED Requirements

### Requirement: Ventana de presentador
La app SHALL mostrar una ventana de presentador con la página actual, la previsualización
de la página siguiente y una barra de navegación.

#### Scenario: Paneles Actual y Siguiente
- **WHEN** la app arranca con un documento válido
- **THEN** el presentador muestra la página actual y, debajo, la página siguiente

#### Scenario: Sin página siguiente
- **WHEN** la página actual es la última
- **THEN** el panel de "Siguiente" indica el fin de la presentación en lugar de una página

#### Scenario: Barra de navegación
- **WHEN** el presentador se muestra
- **THEN** ofrece los controles Inicio, Anterior, Siguiente y Final, deshabilitados cuando
  no tienen efecto (p. ej. "Anterior" en la primera página)

### Requirement: Ventana de audiencia a pantalla completa
La app SHALL abrir una ventana de audiencia a pantalla completa que muestre la página
actual.

#### Scenario: Audiencia a pantalla completa
- **WHEN** la proyección está activa
- **THEN** existe una ventana a pantalla completa con la página actual, sin decoraciones

#### Scenario: Cierre de la audiencia
- **WHEN** el usuario pulsa Esc en la ventana de audiencia
- **THEN** la ventana de audiencia se cierra y la proyección se desactiva

### Requirement: Estado sincronizado entre ventanas
Las dos ventanas SHALL compartir el estado de la presentación (página actual, tema) y
reflejar los cambios de forma coherente.

#### Scenario: Cambio de página sincronizado
- **WHEN** el presentador cambia de página
- **THEN** la ventana de audiencia muestra la nueva página

#### Scenario: Cambio de tema sincronizado
- **WHEN** se alterna el tema
- **THEN** ambas ventanas cambian de tema de forma coherente

### Requirement: Render de páginas Typst
La app SHALL compilar el documento Typst y renderizar cada página como imagen.

#### Scenario: Página renderizada
- **WHEN** el documento Typst compila correctamente
- **THEN** la página actual se muestra como imagen en las ventanas

#### Scenario: Error de compilación
- **WHEN** el documento Typst no compila
- **THEN** la app muestra un mensaje de error en lugar de una página en blanco

### Requirement: Navegación por teclado y ratón
La app SHALL permitir navegar entre páginas con el teclado y el ratón.

#### Scenario: Navegación con teclado
- **WHEN** el usuario pulsa las teclas de avance/retroceso (flechas, `h`/`j`/`k`/`l`,
  `g`/`G`, espacio, Home/End)
- **THEN** la página actual cambia respetando los límites del documento

#### Scenario: Navegación con clic
- **WHEN** el usuario hace clic izquierdo o derecho sobre la página
- **THEN** la página retrocede o avanza respectivamente

### Requirement: Abrir documento
La app SHALL permitir abrir un fichero `.typ` desde la interfaz.

#### Scenario: Abrir fichero
- **WHEN** el usuario elige "Abrir" y selecciona un `.typ`
- **THEN** el documento se carga y se muestra su primera página

### Requirement: Toggle de proyección
La app SHALL permitir mostrar u ocultar la ventana de audiencia.

#### Scenario: Alternar proyección
- **WHEN** el usuario activa/desactiva la proyección
- **THEN** la ventana de audiencia se abre o se cierra en consecuencia

### Requirement: Panel de notas
La app SHALL mostrar y permitir editar las notas del presentador asociadas a la página
actual, con persistencia en la base de datos.

#### Scenario: Ver nota de la página
- **WHEN** la página actual tiene una nota
- **THEN** el panel de notas muestra su contenido (con formato Markdown)

#### Scenario: Editar y guardar nota
- **WHEN** el usuario edita una nota y guarda
- **THEN** el cambio se persiste y se refleja al volver a esa página

#### Scenario: Reimportar notas desde el .typ
- **WHEN** el usuario pulsa "Reimportar"
- **THEN** las notas se reconstruyen a partir del documento Typst

### Requirement: Tema claro/oscuro estilo Adwaita
La app SHALL ofrecer un tema claro y uno oscuro con estética GTK4/Adwaita.

#### Scenario: Alternar tema
- **WHEN** el usuario alterna el tema
- **THEN** los colores de la interfaz cambian entre claro y oscuro de forma coherente

### Requirement: Iconos
La app SHALL mostrar iconos en los controles de la interfaz.

#### Scenario: Iconos visibles
- **WHEN** se muestran los controles (abrir, tema, proyección, notas, navegación)
- **THEN** cada control muestra su icono correspondiente

### Requirement: Binario por defecto
Al completarse la migración, GPUI SHALL ser el framework de UI por defecto de Typshow.

#### Scenario: Binario por defecto en GPUI
- **WHEN** la migración finaliza
- **THEN** el binario `typshow` usa GPUI y `egui`/`eframe` se han retirado de las
  dependencias

## MODIFIED Requirements
<!-- Ninguna -->

## REMOVED Requirements
<!-- La UI egui se retira al final; se documentará aquí si aplica -->
