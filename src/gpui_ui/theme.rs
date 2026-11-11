//! Paleta de colores claro/oscuro (estilo Adwaita mínimo) para la UI GPUI.

use gpui::{Rgba, rgb};

/// Colores base de la interfaz para un tema dado.
pub struct Palette {
    /// Fondo general.
    pub bg: Rgba,
    /// Fondo de paneles/superficies elevadas.
    pub surface: Rgba,
    /// Color de texto principal.
    pub fg: Rgba,
    /// Color de texto atenuado (secundario, deshabilitado).
    pub muted: Rgba,
    /// Color de bordes y separadores.
    pub border: Rgba,
    /// Color de acento (botones activos).
    pub accent: Rgba,
    /// Texto/icono sobre el acento.
    pub on_accent: Rgba,
    /// Color de aviso/error.
    pub danger: Rgba,
}

/// Devuelve la paleta del tema claro u oscuro.
pub fn palette(dark: bool) -> Palette {
    if dark {
        Palette {
            bg: rgb(0x1e1e1e),
            surface: rgb(0x2a2a2a),
            fg: rgb(0xffffff),
            muted: rgb(0x9a9a9a),
            border: rgb(0x3a3a3a),
            accent: rgb(0x3584e4),
            on_accent: rgb(0xffffff),
            danger: rgb(0xc01c28),
        }
    } else {
        Palette {
            bg: rgb(0xfafafa),
            surface: rgb(0xffffff),
            fg: rgb(0x1e1e1e),
            muted: rgb(0x777777),
            border: rgb(0xd0d0d0),
            accent: rgb(0x3584e4),
            on_accent: rgb(0xffffff),
            danger: rgb(0xc01c28),
        }
    }
}
