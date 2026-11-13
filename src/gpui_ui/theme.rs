//! Tokens de diseño Adwaita/GTK4 (tema claro y oscuro) para la UI GPUI.
//!
//! Este módulo es el **único punto de verdad** del aspecto visual: los colores
//! salen de [`palette`] y las medidas (radios, espaciados, alturas de control y
//! tamaños de fuente) de las constantes [`RADIUS_BUTTON`], [`SPACING_MD`], etc.
//! El pintado (`presenter.rs`, `audience.rs`) no debe contener colores, radios
//! ni paddings literales.
//!
//! Los valores siguen la paleta de libadwaita 1.x sobre GTK4. Dos desviaciones
//! conscientes respecto a Adwaita, ambas por accesibilidad:
//!
//! * `muted` (etiqueta atenuada) es algo más marcado que el `dim-label` de
//!   Adwaita (`rgba(0,0,0,.55)` / `rgba(255,255,255,.5)`) para superar el 4.5:1
//!   de WCAG AA también sobre `headerbar_bg` y `surface`.
//! * `accent` rellena los botones con el `#3584e4` de Adwaita y texto blanco
//!   (contraste 3.77:1): cumple AA para texto grande y para componentes de
//!   interfaz (WCAG 1.4.11), no para texto normal; `accent_text` es el acento
//!   legible sobre fondos neutros.
//!
//! Todos los tokens del módulo están documentados; los que aún no consume el
//! pintado se marcan como reservados.

use gpui::{Rgba, rgb, rgba};

// ---------------------------------------------------------------------------
// Geometría: escala de libadwaita (multiplos de 3) y radios de GTK4.
// ---------------------------------------------------------------------------

/// Radio de botones, entradas y píldoras (`6px`).
pub const RADIUS_BUTTON: f32 = 6.;
/// Radio de tarjetas, paneles y vistas (`12px`).
pub const RADIUS_CARD: f32 = 12.;

/// Espaciado mínimo (`3px`).
pub const SPACING_XXS: f32 = 3.;
/// Espaciado pequeño (`6px`).
pub const SPACING_SM: f32 = 6.;
/// Espaciado medio (`12px`).
pub const SPACING_MD: f32 = 12.;
/// Espaciado grande (`18px`). Reservado: aún sin consumidor en el pintado.
pub const SPACING_LG: f32 = 18.;
/// Espaciado máximo (`24px`).
pub const SPACING_XL: f32 = 24.;

/// Altura de un control Adwaita (botón, entrada de texto): `34px`.
pub const CONTROL_HEIGHT: f32 = 34.;
/// Padding horizontal de un control.
pub const CONTROL_PADDING_X: f32 = SPACING_MD;
/// Padding interior de la ventana.
pub const WINDOW_PADDING: f32 = SPACING_MD;
/// Padding interior de los paneles que muestran una página.
pub const PANE_PADDING: f32 = SPACING_XL;
/// Ancho del panel lateral de notas.
pub const NOTES_PANEL_WIDTH: f32 = 320.;

/// Grosor de bordes, separadores y subrayados (`1px`).
pub const BORDER_WIDTH: f32 = 1.;
/// Grosor del cursor de texto (`2px`).
pub const CARET_WIDTH: f32 = 2.;

/// Cuerpo de texto y etiquetas de control (`11pt`).
pub const FONT_SIZE_BODY: f32 = 14.;
/// Texto secundario, pistas y etiquetas de panel (`11pt` → `small`).
pub const FONT_SIZE_CAPTION: f32 = 11.;
/// Título de ventana / cabecera (`17pt` → `title-1`).
pub const FONT_SIZE_TITLE: f32 = 17.;
/// Subtítulo de notas (`15px`): paso intermedio entre [`FONT_SIZE_TITLE`] y
/// [`FONT_SIZE_BODY`] para los títulos de nivel 2.
pub const FONT_SIZE_SUBTITLE: f32 = 15.;

/// Familia monoespaciada para bloques de código de las notas.
///
/// `Adwaita Mono` acompaña a la paleta de libadwaita; si no está instalada,
/// GPUI cae a la pila de fuentes del sistema (texto proporcional, sin romper el
/// pintado).
pub const FONT_FAMILY_MONO: &str = "Adwaita Mono";

/// Colores base de la interfaz para un tema dado.
///
/// Todos los campos son opacos salvo [`Palette::shadow`], que lleva alfa.
///
/// Tokens **reservados** (definidos y documentados, todavía sin consumidor en el
/// pintado; previstos para las tareas 5.2/6): [`Palette::surface_alt`],
/// [`Palette::active`], [`Palette::warning`], [`Palette::success`] y
/// [`Palette::shadow`].
pub struct Palette {
    /// Fondo de la ventana (chrome de la app): `#fafafa` claro, `#242424` oscuro.
    pub bg: Rgba,
    /// Fondo de vista: el lienzo donde se pinta el documento.
    pub view_bg: Rgba,
    /// Fondo de tarjetas, entradas y popovers (`#ffffff` / `#303030`).
    pub surface: Rgba,
    /// Superficie secundaria para filas alternas o zonas hundidas.
    /// Reservado: aún sin consumidor en el pintado.
    pub surface_alt: Rgba,
    /// Fondo de barra superior / encabezado (`#ebebeb` / `#303030`).
    pub headerbar_bg: Rgba,
    /// Color de texto principal.
    pub fg: Rgba,
    /// Texto atenuado (etiquetas secundarias).
    pub muted: Rgba,
    /// Texto deshabilitado.
    pub disabled_fg: Rgba,
    /// Fondo de control deshabilitado.
    pub disabled_bg: Rgba,
    /// Color de bordes y separadores.
    pub border: Rgba,
    /// Borde resaltado (más marcado que [`Palette::border`]).
    pub border_strong: Rgba,
    /// Acento de Adwaita: foco, resaltado, borde de selección y relleno de los
    /// botones sugeridos.
    pub accent: Rgba,
    /// Primer plano sobre [`Palette::accent`].
    pub accent_fg: Rgba,
    /// Texto con color de acento legible sobre fondos neutros (pistas, enlaces).
    pub accent_text: Rgba,
    /// Fondo de acento en estado hover.
    pub accent_hover: Rgba,
    /// Fondo de un control neutro en estado hover.
    pub hover: Rgba,
    /// Fondo de un control neutro pulsado/activo. Reservado: aún sin consumidor
    /// en el pintado.
    pub active: Rgba,
    /// Color de aviso/advertencia. Reservado: aún sin consumidor en el pintado.
    pub warning: Rgba,
    /// Color de éxito.
    /// Reservado: aún sin consumidor en el pintado.
    pub success: Rgba,
    /// Color destructivo/error.
    pub danger: Rgba,
    /// Primer plano sobre [`Palette::danger`].
    pub on_danger: Rgba,
    /// Color de sombra (con alfa; `rgba()` espera `0xRRGGBBAA`).
    /// Reservado: aún sin consumidor en el pintado.
    pub shadow: Rgba,
}

/// Devuelve la paleta del tema claro u oscuro.
pub fn palette(dark: bool) -> Palette {
    if dark {
        Palette {
            bg: rgb(0x242424),
            view_bg: rgb(0x1e1e1e),
            surface: rgb(0x303030),
            surface_alt: rgb(0x383838),
            headerbar_bg: rgb(0x303030),
            fg: rgb(0xffffff),
            // Adwaita usa `rgba(255,255,255,.5)`; se sube a `#a0a0a0` para
            // mantener 4.5:1 también sobre `surface`.
            muted: rgb(0xa0a0a0),
            disabled_fg: rgb(0x5e5c64),
            disabled_bg: rgb(0x2b2b2b),
            border: rgb(0x3d3846),
            border_strong: rgb(0x5e5c64),
            accent: rgb(0x3584e4),
            accent_fg: rgb(0xffffff),
            accent_text: rgb(0x78aeed),
            accent_hover: rgb(0x4a90e8),
            // Overlay blanco sobre `surface` (`#303030`): hover ≈10 % y active ≈15 %.
            hover: rgb(0x454545),
            active: rgb(0x4f4f4f),
            warning: rgb(0xe5a50a),
            success: rgb(0x2ec27e),
            danger: rgb(0xc01c28),
            on_danger: rgb(0xffffff),
            shadow: rgba(0x00000080),
        }
    } else {
        Palette {
            bg: rgb(0xfafafa),
            view_bg: rgb(0xffffff),
            surface: rgb(0xffffff),
            surface_alt: rgb(0xf6f5f4),
            headerbar_bg: rgb(0xebebeb),
            fg: rgb(0x1e1e1e),
            // Adwaita usa `rgba(0,0,0,.55)`; se sube a `#676767` para mantener
            // 4.5:1 también sobre `headerbar_bg` (`#ebebeb`) y `surface`.
            muted: rgb(0x676767),
            disabled_fg: rgb(0xb0b0b0),
            disabled_bg: rgb(0xeeeeee),
            border: rgb(0xd8d8d8),
            border_strong: rgb(0xb7b7b7),
            accent: rgb(0x3584e4),
            accent_fg: rgb(0xffffff),
            accent_text: rgb(0x1c71d8),
            accent_hover: rgb(0x3077cd),
            // Overlay negro sobre `surface` (`#ffffff`): hover ≈6 % y active ≈11 %.
            hover: rgb(0xf0f0f0),
            active: rgb(0xe3e3e3),
            warning: rgb(0xe5a50a),
            success: rgb(0x2ec27e),
            danger: rgb(0xc01c28),
            on_danger: rgb(0xffffff),
            shadow: rgba(0x0000002e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Umbral WCAG AA para texto normal.
    const AA_TEXT: f32 = 4.5;
    /// Umbral WCAG AA para texto grande y componentes de interfaz (1.4.11).
    const AA_UI: f32 = 3.0;

    /// Luminancia relativa WCAG 2.x de un color (se asume opaco).
    fn relative_luminance(color: Rgba) -> f32 {
        fn channel(value: f32) -> f32 {
            if value <= 0.03928 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        }
        channel(color.r) * 0.2126 + channel(color.g) * 0.7152 + channel(color.b) * 0.0722
    }

    /// Ratio de contraste WCAG entre dos colores (1.0 = idénticos, 21.0 = extremo).
    fn contrast(a: Rgba, b: Rgba) -> f32 {
        let (la, lb) = (relative_luminance(a), relative_luminance(b));
        let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
        (hi + 0.05) / (lo + 0.05)
    }

    /// Todos los tokens de color de una paleta, para comprobar que existen todos.
    fn all_tokens(p: &Palette) -> [Rgba; 21] {
        [
            p.bg,
            p.view_bg,
            p.surface,
            p.surface_alt,
            p.headerbar_bg,
            p.fg,
            p.muted,
            p.disabled_fg,
            p.disabled_bg,
            p.border,
            p.border_strong,
            p.accent,
            p.accent_fg,
            p.accent_text,
            p.accent_hover,
            p.hover,
            p.active,
            p.warning,
            p.success,
            p.danger,
            p.on_danger,
        ]
    }

    #[test]
    fn palettes_conservan_todos_los_tokens_opacos() {
        for dark in [true, false] {
            let p = palette(dark);
            for (i, token) in all_tokens(&p).into_iter().enumerate() {
                assert!(
                    token.a == 1.0,
                    "token {i} del tema dark={dark} no es opaco: {token:?}"
                );
            }
            assert!(p.shadow.a < 1.0, "la sombra debe llevar alfa");
        }
    }

    #[test]
    fn tema_claro_y_oscuro_difieren_en_fondos_y_texto() {
        let (dark, light) = (palette(true), palette(false));
        assert_ne!(dark.bg, light.bg, "el fondo debe diferir entre temas");
        assert_ne!(dark.view_bg, light.view_bg);
        assert_ne!(dark.surface, light.surface);
        assert_ne!(dark.fg, light.fg, "el texto debe diferir entre temas");
        assert_ne!(dark.muted, light.muted);
        assert_ne!(dark.disabled_fg, light.disabled_fg);
        assert_eq!(
            dark.accent, light.accent,
            "el acento de Adwaita es el mismo en ambos temas"
        );
    }

    #[test]
    fn tokens_de_estado_y_acento_son_distintos_de_su_base() {
        /// Contraste mínimo exigible a un estado hover: por debajo es imperceptible.
        const MIN_HOVER_CONTRAST: f32 = 1.10;

        for dark in [true, false] {
            let p = palette(dark);
            let hover = contrast(p.hover, p.surface);
            assert!(
                hover >= MIN_HOVER_CONTRAST,
                "hover sobre surface (dark={dark}): {hover:.2} < {MIN_HOVER_CONTRAST}"
            );
            // El estado pulsado debe notarse más que el hover: en claro oscurece
            // (overlay negro) y en oscuro aclara (overlay blanco), como Adwaita.
            let active = contrast(p.active, p.surface);
            assert!(
                active > hover,
                "active debe destacar más que hover (dark={dark}): {active:.2} <= {hover:.2}"
            );
            assert_ne!(p.active, p.hover);
            assert_ne!(p.accent_hover, p.accent);
            assert_ne!(p.disabled_bg, p.surface);
            assert_ne!(p.border_strong, p.border);
            assert_ne!(p.surface_alt, p.surface);
        }
    }

    #[test]
    fn texto_principal_cumple_aa_sobre_fondos_y_superficies() {
        for dark in [true, false] {
            let p = palette(dark);
            for (nombre, fondo) in [
                ("bg", p.bg),
                ("view_bg", p.view_bg),
                ("surface", p.surface),
                ("surface_alt", p.surface_alt),
                ("headerbar_bg", p.headerbar_bg),
            ] {
                let ratio = contrast(p.fg, fondo);
                assert!(
                    ratio >= AA_TEXT,
                    "fg sobre {nombre} (dark={dark}): {ratio:.2} < {AA_TEXT}"
                );
            }
        }
    }

    #[test]
    fn texto_atenuado_cumple_aa_sobre_fondos_y_superficies() {
        for dark in [true, false] {
            let p = palette(dark);
            for (nombre, fondo) in [
                ("bg", p.bg),
                ("surface", p.surface),
                ("view_bg", p.view_bg),
                ("headerbar_bg", p.headerbar_bg),
            ] {
                let ratio = contrast(p.muted, fondo);
                assert!(
                    ratio >= AA_TEXT,
                    "muted sobre {nombre} (dark={dark}): {ratio:.2} < {AA_TEXT}"
                );
            }
        }
    }

    #[test]
    fn acento_y_su_texto_cumplen_el_minimo_de_componentes() {
        for dark in [true, false] {
            let p = palette(dark);
            let ratio = contrast(p.accent_fg, p.accent);
            assert!(
                ratio >= AA_UI,
                "accent_fg sobre accent (dark={dark}): {ratio:.2} < {AA_UI}"
            );
            // El acento rellena botones: el texto blanco de Adwaita da 3.77:1.
            assert_eq!(p.accent_fg, rgb(0xffffff));
        }
    }

    #[test]
    fn texto_de_acento_y_destructivo_son_legibles() {
        for dark in [true, false] {
            let p = palette(dark);
            for (nombre, fondo) in [("view_bg", p.view_bg), ("bg", p.bg)] {
                let link = contrast(p.accent_text, fondo);
                assert!(
                    link >= AA_TEXT,
                    "accent_text sobre {nombre} (dark={dark}): {link:.2} < {AA_TEXT}"
                );
            }
            let banner = contrast(p.on_danger, p.danger);
            assert!(
                banner >= AA_TEXT,
                "on_danger sobre danger (dark={dark}): {banner:.2} < {AA_TEXT}"
            );
        }
    }

    #[test]
    fn geometria_sigue_la_escala_adwaita() {
        assert_eq!(RADIUS_BUTTON, 6.0);
        assert_eq!(RADIUS_CARD, 12.0);
        assert_eq!(CONTROL_HEIGHT, 34.0);
        assert_eq!(BORDER_WIDTH, 1.0);
        assert_eq!(CARET_WIDTH, 2.0);

        let escala = [SPACING_XXS, SPACING_SM, SPACING_MD, SPACING_LG, SPACING_XL];
        assert_eq!(escala, [3.0, 6.0, 12.0, 18.0, 24.0]);
        assert!(
            escala.windows(2).all(|w| w[1] > w[0]),
            "la escala de espaciado debe ser creciente"
        );

        assert_eq!(FONT_SIZE_CAPTION, 11.0);
        assert_eq!(FONT_SIZE_BODY, 14.0);
        assert_eq!(FONT_SIZE_SUBTITLE, 15.0);
        assert_eq!(FONT_SIZE_TITLE, 17.0);
        // Comparaciones entre constantes: se comprueban en tiempo de compilación.
        const { assert!(FONT_SIZE_TITLE > FONT_SIZE_SUBTITLE) };
        const { assert!(FONT_SIZE_SUBTITLE > FONT_SIZE_BODY) };
        const { assert!(FONT_SIZE_BODY > FONT_SIZE_CAPTION) };

        assert_eq!(CONTROL_PADDING_X, SPACING_MD);
        assert_eq!(PANE_PADDING, SPACING_XL);
        assert_eq!(WINDOW_PADDING, SPACING_MD);
        assert_eq!(NOTES_PANEL_WIDTH, 320.0);
    }
}
