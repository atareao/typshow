//! Paleta de colores claro/oscuro (estilo Adwaita mínimo) para la UI GPUI.

use gpui::{Rgba, rgb};

/// Colores base de la interfaz para un tema dado.
pub struct Palette {
    pub bg: Rgba,
    pub fg: Rgba,
    pub border: Rgba,
    pub accent: Rgba,
    pub on_accent: Rgba,
}

/// Devuelve la paleta del tema claro u oscuro.
pub fn palette(dark: bool) -> Palette {
    if dark {
        Palette {
            bg: rgb(0x1e1e1e),
            fg: rgb(0xffffff),
            border: rgb(0x3a3a3a),
            accent: rgb(0x3584e4),
            on_accent: rgb(0xffffff),
        }
    } else {
        Palette {
            bg: rgb(0xfafafa),
            fg: rgb(0x1e1e1e),
            border: rgb(0xd0d0d0),
            accent: rgb(0x3584e4),
            on_accent: rgb(0xffffff),
        }
    }
}
