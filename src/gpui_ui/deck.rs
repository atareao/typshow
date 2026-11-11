//! Estado compartido y render de páginas de la UI GPUI.

use std::collections::HashMap;
use std::sync::Arc;

use gpui::RenderImage;
use typst_layout::PagedDocument;

/// Estado de la presentación compartido entre las ventanas.
pub struct Deck {
    /// Página actual (0-based).
    pub page: usize,
    /// Número total de páginas.
    pub total: usize,
    /// Tema oscuro activo.
    pub dark: bool,
    /// Mensaje de error, si lo hay.
    pub error: Option<String>,
    doc: Option<PagedDocument>,
    cache: HashMap<usize, Arc<RenderImage>>,
}

impl Deck {
    /// Deck vacío sin documento.
    pub fn empty() -> Self {
        Self {
            page: 0,
            total: 0,
            dark: false,
            error: None,
            doc: None,
            cache: HashMap::new(),
        }
    }

    /// Deck con un mensaje de error.
    pub fn with_error(msg: impl Into<String>) -> Self {
        let mut deck = Self::empty();
        deck.error = Some(msg.into());
        deck
    }

    /// Carga y compila un documento Typst desde `path`.
    pub fn load(path: &str) -> Self {
        let mut deck = Self::empty();
        match std::fs::read_to_string(path) {
            Ok(text) => {
                let world = crate::TypstWorld::new(&text, path);
                match typst::compile::<PagedDocument>(&world).output {
                    Ok(doc) => {
                        deck.total = doc.pages().len();
                        deck.doc = Some(doc);
                    }
                    Err(diags) => {
                        let msg: Vec<String> = diags.iter().map(|d| format!("{d:?}")).collect();
                        deck.error = Some(format!("Error compilando Typst:\n{}", msg.join("\n")));
                    }
                }
            }
            Err(e) => deck.error = Some(format!("Error leyendo fichero: {e}")),
        }
        deck
    }

    /// Va a la primera página.
    pub fn first(&mut self) {
        self.page = 0;
    }

    /// Retrocede una página (sin salir del rango).
    pub fn prev(&mut self) {
        if self.page > 0 {
            self.page -= 1;
        }
    }

    /// Avanza una página (sin salir del rango).
    pub fn next(&mut self) {
        if self.page + 1 < self.total {
            self.page += 1;
        }
    }

    /// Va a la última página.
    pub fn last(&mut self) {
        if self.total > 0 {
            self.page = self.total - 1;
        }
    }

    /// Alterna el tema claro/oscuro.
    pub fn toggle_theme(&mut self) {
        self.dark = !self.dark;
    }

    /// Índice de la página siguiente, si existe.
    pub fn next_index(&self) -> Option<usize> {
        let n = self.page + 1;
        (n < self.total).then_some(n)
    }

    /// Devuelve (renderizando y cacheando si hace falta) la imagen de la página `idx`.
    pub fn image_at(&mut self, idx: usize) -> Option<Arc<RenderImage>> {
        if let Some(image) = self.cache.get(&idx) {
            return Some(image.clone());
        }
        let page = self.doc.as_ref()?.pages().get(idx)?;
        let pt_width = page.frame.size().x.to_pt();
        let pixel_per_pt = if pt_width > 0.0 {
            1920.0 / pt_width
        } else {
            2.0
        };
        let options = typst_render::RenderOptions {
            pixel_per_pt: typst::utils::Scalar::new(pixel_per_pt),
            render_bleed: false,
        };
        let pixmap = typst_render::render(page, &options);
        let (width, height) = (pixmap.width(), pixmap.height());
        let mut data = pixmap.data().to_vec();
        rgba_to_bgra(&mut data);
        let rgba = image::RgbaImage::from_raw(width, height, data)?;
        let render_image = Arc::new(RenderImage::new(vec![image::Frame::new(rgba)]));
        self.cache.insert(idx, render_image.clone());
        Some(render_image)
    }

    /// Imagen de la página actual.
    pub fn current_image(&mut self) -> Option<Arc<RenderImage>> {
        self.image_at(self.page)
    }

    /// Imagen de la página siguiente, si existe.
    pub fn next_image(&mut self) -> Option<Arc<RenderImage>> {
        let idx = self.next_index()?;
        self.image_at(idx)
    }
}

/// Convierte RGBA premultiplicado (salida de Typst/tiny-skia) a BGRA no premultiplicado
/// (formato interno de `RenderImage`).
///
/// Delega en `gpui::swap_rgba_pa_to_bgra` para no divergir de la implementación
/// canónica de GPUI (evita bugs si cambia el formato interno).
pub fn rgba_to_bgra(data: &mut [u8]) {
    debug_assert_eq!(
        data.len() % 4,
        0,
        "el buffer debe ser RGBA (longitud múltiplo de 4)"
    );
    gpui::swap_rgba_pa_to_bgra(data);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deck_with(total: usize) -> Deck {
        let mut d = Deck::empty();
        d.total = total;
        d
    }

    #[test]
    fn navigation_respects_bounds() {
        let mut d = deck_with(3);
        d.prev();
        assert_eq!(d.page, 0, "no retrocede en la primera");
        d.next();
        assert_eq!(d.page, 1);
        d.next();
        d.next();
        assert_eq!(d.page, 2, "no avanza más allá de la última");
        d.first();
        assert_eq!(d.page, 0);
        d.last();
        assert_eq!(d.page, 2);
    }

    #[test]
    fn next_index_reports_last() {
        let mut d = deck_with(2);
        assert_eq!(d.next_index(), Some(1));
        d.last();
        assert_eq!(d.next_index(), None);
    }

    #[test]
    fn empty_deck_has_no_pages() {
        let mut d = Deck::empty();
        d.next();
        d.last();
        assert_eq!(d.page, 0);
        assert_eq!(d.next_index(), None);
    }

    #[test]
    fn toggle_theme_flips() {
        let mut d = Deck::empty();
        assert!(!d.dark);
        d.toggle_theme();
        assert!(d.dark);
        d.toggle_theme();
        assert!(!d.dark);
    }

    #[test]
    fn load_invalid_path_sets_error() {
        let d = Deck::load("/nonexistent/no-existe-jamas.typ");
        assert!(d.error.is_some(), "debe registrar error de lectura");
        assert_eq!(d.total, 0);
        assert!(d.doc.is_none());
    }

    #[test]
    fn load_and_cache_page() {
        let path = std::env::temp_dir().join("typshow_deck_test.typ");
        std::fs::write(&path, "= Hola\n\nTexto de prueba\n").unwrap();
        let mut d = Deck::load(path.to_str().unwrap());
        assert!(d.error.is_none(), "error inesperado: {:?}", d.error);
        assert!(d.total >= 1, "debe tener al menos una pagina");

        let a = d.image_at(0).expect("imagen de la pagina 0");
        let b = d.image_at(0).expect("imagen de la pagina 0 (bis)");
        assert!(
            Arc::ptr_eq(&a, &b),
            "la segunda llamada debe venir de la cache"
        );

        assert!(d.image_at(d.total).is_none(), "fuera de rango -> None");
        assert!(d.current_image().is_some());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn rgba_to_bgra_unpremultiplies() {
        let mut data = vec![255, 0, 0, 255];
        rgba_to_bgra(&mut data);
        assert_eq!(data, [0, 0, 255, 255]);

        let mut data = vec![128, 0, 0, 128];
        rgba_to_bgra(&mut data);
        assert_eq!(data, [0, 0, 254, 128]);

        let mut data = vec![0, 0, 0, 0];
        rgba_to_bgra(&mut data);
        assert_eq!(data, [0, 0, 0, 0]);

        let mut data = vec![10, 20, 30, 40];
        rgba_to_bgra(&mut data);
        assert_eq!(data, [191, 127, 63, 40]);
    }
}
