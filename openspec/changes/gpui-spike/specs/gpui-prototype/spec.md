# Spec Delta

## Purpose

Prototipo de interfaz GPUI que demuestra la viabilidad de sustituir egui en Typshow mediante dos ventanas sincronizadas y render de páginas Typst.

## ADDED Requirements

### Requirement: Doble ventana sincronizada
El prototipo SHALL mostrar dos ventanas nativas: una ventana de presentador en modo ventana y una ventana de audiencia a pantalla completa, que comparten la misma página actual.

#### Scenario: Ambas ventanas se abren
- **WHEN** se lanza el binario prototipo con un fichero Typst válido
- **THEN** se abren dos ventanas: presenter (windowed) y audiencia (fullscreen)

#### Scenario: El cambio de página se refleja en la audiencia
- **WHEN** el presentador cambia de página
- **THEN** la ventana de audiencia muestra la nueva página

### Requirement: Render de páginas Typst
El prototipo SHALL renderizar la página actual del documento Typst como una imagen y mostrarla en ambas ventanas.

#### Scenario: Página renderizada
- **WHEN** el documento Typst se carga correctamente
- **THEN** la página actual se muestra como imagen en las dos ventanas

#### Scenario: Error de compilación
- **WHEN** el documento Typst no compila
- **THEN** el prototipo muestra un mensaje de error en lugar de una página en blanco

### Requirement: Navegación
El prototipo SHALL ofrecer navegación a primera página, anterior, siguiente y última, tanto por controles en la UI como por teclado.

#### Scenario: Navegación con controles
- **WHEN** el usuario pulsa "Siguiente" en la ventana presentador
- **THEN** la página actual avanza una posición, respetando el límite de la última página

#### Scenario: Navegación con teclado
- **WHEN** el usuario pulsa las teclas de avance o retroceso de página
- **THEN** la página actual cambia de acuerdo con la tecla pulsada

### Requirement: Tema claro/oscuro
El prototipo SHALL permitir alternar entre un tema claro y uno oscuro.

#### Scenario: Alternar tema
- **WHEN** el usuario activa el cambio de tema
- **THEN** las dos ventanas cambian entre tema claro y oscuro de forma coherente

### Requirement: Informe de viabilidad
El spike SHALL producir un informe en `docs/gpui-spike.md` con hallazgos y un veredicto razonado sobre migrar o no a GPUI.

#### Scenario: Informe presente
- **WHEN** el spike finaliza
- **THEN** existe `docs/gpui-spike.md` con hallazgos y veredicto