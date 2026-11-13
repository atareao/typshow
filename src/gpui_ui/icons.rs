//! Iconos de la interfaz: Phosphor embebido como fuente.
//!
//! La UI GPUI pinta los iconos como glifos de texto, no como SVG: no hace falta
//! configurar un `AssetSource` y el renderizado es el mismo en Wayland/Niri.
//! La fuente se vendoriza en `assets/fonts/Phosphor.ttf` (ver
//! `assets/fonts/LICENSE-Phosphor.txt`) para no depender del crate
//! `egui-phosphor`, que se retira en la fase 6 de la migración.
//!
//! Uso: [`register`] una sola vez al arrancar la app y después [`label`] (icono
//! + texto, como en la app egui) o [`icon`] (glifo suelto) en cada control.

use std::borrow::Cow;

use gpui::{App, IntoElement, Rgba, SharedString, div, prelude::*, px};

use super::theme::SPACING_XXS;

/// Familia tipográfica interna de `assets/fonts/Phosphor.ttf`.
pub const FONT_FAMILY: &str = "Phosphor";

/// Fuente Phosphor vendorizada (incluida en el binario en tiempo de compilación).
pub const FONT_BYTES: &[u8] = include_bytes!("../../assets/fonts/Phosphor.ttf");

/// Abrir fichero (`FOLDER_OPEN`).
pub const FOLDER_OPEN: char = '\u{E256}';
/// Tema oscuro activo (`MOON`).
pub const MOON: char = '\u{E330}';
/// Tema claro activo (`SUN`).
pub const SUN: char = '\u{E472}';
/// Proyección / ventana de audiencia (`DESKTOP`).
pub const DESKTOP: char = '\u{E560}';
/// Notas visibles (`NOTE`).
pub const NOTE: char = '\u{E348}';
/// Notas ocultas (`NOTE_BLANK`).
pub const NOTE_BLANK: char = '\u{E34A}';
/// Navegación al inicio (`SKIP_BACK`).
pub const SKIP_BACK: char = '\u{E5A4}';
/// Navegación a la página anterior (`CARET_LEFT`).
pub const CARET_LEFT: char = '\u{E138}';
/// Navegación a la página siguiente (`CARET_RIGHT`).
pub const CARET_RIGHT: char = '\u{E13A}';
/// Navegación al final (`SKIP_FORWARD`).
pub const SKIP_FORWARD: char = '\u{E5A6}';
/// Guardar (`FLOPPY_DISK`).
pub const FLOPPY_DISK: char = '\u{E248}';
/// Cancelar (`X_CIRCLE`).
pub const X_CIRCLE: char = '\u{E4F8}';
/// Editar (`PENCIL_SIMPLE`).
pub const PENCIL_SIMPLE: char = '\u{E3B4}';
/// Nueva nota (`PLUS`).
pub const PLUS: char = '\u{E3D4}';
/// Eliminar nota (`TRASH`).
pub const TRASH: char = '\u{E4A6}';
/// Reimportar notas (`ARROWS_CLOCKWISE`).
pub const ARROWS_CLOCKWISE: char = '\u{E094}';

/// Todos los iconos de la interfaz, para validarlos de una vez.
pub const ALL: [char; 16] = [
    FOLDER_OPEN,
    MOON,
    SUN,
    DESKTOP,
    NOTE,
    NOTE_BLANK,
    SKIP_BACK,
    CARET_LEFT,
    CARET_RIGHT,
    SKIP_FORWARD,
    FLOPPY_DISK,
    X_CIRCLE,
    PENCIL_SIMPLE,
    PLUS,
    TRASH,
    ARROWS_CLOCKWISE,
];

/// Registra la fuente Phosphor en el sistema de texto de GPUI.
///
/// Debe llamarse una sola vez, al construir la app y antes de abrir ventanas.
pub fn register(cx: &App) -> gpui::Result<()> {
    cx.text_system().add_fonts(vec![Cow::Borrowed(FONT_BYTES)])
}

/// Glifo de un icono como `SharedString`.
///
/// `SharedString` envuelve un `SmolStr`, así que un `char` cabe inline: no hay
/// asignación en el heap ni por frame.
pub fn glyph(ch: char) -> SharedString {
    SharedString::from(ch)
}

/// Etiqueta de un control: icono + texto en una fila.
///
/// El icono va en su propio `div` con [`FONT_FAMILY`] para que el glifo se
/// resuelva con Phosphor y **no** por el fallback del sistema: GPUI crea un
/// único span con la fuente primaria cuando no hay `font_fallbacks` y
/// cosmic-text devuelve la primera cara de la fontdb que tenga ese codepoint
/// (a menudo otra fuente, porque los iconos viven en el área de uso privado).
///
/// El icono no fija `text_color`: hereda el del control (`disabled_fg`,
/// `accent_fg`, `danger`…), igual que el texto.
pub fn label(icon: char, text: impl Into<SharedString>) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(SPACING_XXS))
        .child(
            div()
                .flex_shrink_0()
                .font_family(FONT_FAMILY)
                .child(glyph(icon)),
        )
        .child(text.into())
}

/// Pinta un glifo de la fuente Phosphor con el tamaño y color dados.
pub fn icon(ch: char, size: f32, color: Rgba) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .font_family(FONT_FAMILY)
        .text_size(px(size))
        .text_color(color)
        .child(glyph(ch))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parsea la fuente vendorizada; falla con un mensaje útil si está corrupta.
    fn face() -> ttf_parser::Face<'static> {
        ttf_parser::Face::parse(FONT_BYTES, 0).expect("Phosphor.ttf debe ser un TTF válido")
    }

    #[test]
    fn glyph_devuelve_el_caracter_del_icono() {
        assert_eq!(glyph(CARET_LEFT).as_ref(), "\u{E138}");
        assert_eq!(glyph(ARROWS_CLOCKWISE).as_ref(), "\u{E094}");
    }

    #[test]
    fn los_codepoints_coinciden_con_la_tabla_dorada() {
        // Tabla dorada: (nombre, constante, codepoint documentado). Un typo en
        // una constante o un cambio de codepoint rompe este test.
        let tabla: [(&str, char, char); 16] = [
            ("FOLDER_OPEN", FOLDER_OPEN, '\u{E256}'),
            ("MOON", MOON, '\u{E330}'),
            ("SUN", SUN, '\u{E472}'),
            ("DESKTOP", DESKTOP, '\u{E560}'),
            ("NOTE", NOTE, '\u{E348}'),
            ("NOTE_BLANK", NOTE_BLANK, '\u{E34A}'),
            ("SKIP_BACK", SKIP_BACK, '\u{E5A4}'),
            ("CARET_LEFT", CARET_LEFT, '\u{E138}'),
            ("CARET_RIGHT", CARET_RIGHT, '\u{E13A}'),
            ("SKIP_FORWARD", SKIP_FORWARD, '\u{E5A6}'),
            ("FLOPPY_DISK", FLOPPY_DISK, '\u{E248}'),
            ("X_CIRCLE", X_CIRCLE, '\u{E4F8}'),
            ("PENCIL_SIMPLE", PENCIL_SIMPLE, '\u{E3B4}'),
            ("PLUS", PLUS, '\u{E3D4}'),
            ("TRASH", TRASH, '\u{E4A6}'),
            ("ARROWS_CLOCKWISE", ARROWS_CLOCKWISE, '\u{E094}'),
        ];
        for (nombre, actual, esperado) in tabla {
            assert_eq!(
                actual, esperado,
                "{nombre} no coincide con su codepoint documentado"
            );
            assert!(ALL.contains(&actual), "{nombre} no está en ALL");
        }
    }

    #[test]
    fn todos_los_iconos_tienen_codepoint_distinto() {
        let mut codepoints = ALL.to_vec();
        codepoints.sort_unstable();
        codepoints.dedup();
        assert_eq!(
            codepoints.len(),
            ALL.len(),
            "hay codepoints repetidos en ALL"
        );
    }

    #[test]
    fn la_fuente_embebida_declara_la_familia_phosphor() {
        let face = face();
        let familias: Vec<String> = face
            .names()
            .into_iter()
            .filter(|name| name.name_id == ttf_parser::name_id::FAMILY && name.is_unicode())
            .filter_map(|name| name.to_string())
            .collect();
        assert!(
            familias.iter().any(|familia| familia == FONT_FAMILY),
            "la fuente no declara la familia {FONT_FAMILY:?}: {familias:?}"
        );
    }

    #[test]
    fn la_fuente_embebida_contiene_el_glifo_de_cada_icono() {
        let face = face();
        for ch in ALL {
            assert!(
                face.glyph_index(ch).is_some(),
                "la fuente no contiene el glifo U+{:04X}",
                ch as u32
            );
        }
    }
}
