//! Estado compartido y render de páginas de la UI GPUI.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use gpui::{Context, RenderImage};
use typst_layout::PagedDocument;

use tracing::warn;

use crate::Notes;

/// Número máximo de páginas rasterizadas en caché antes de desalojar la menos usada.
pub const CACHE_CAP: usize = 6;

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
    /// Ruta del documento cargado.
    pub file_path: Option<String>,
    /// Fuente Typst del documento (para reimportar notas).
    typ_content: Option<String>,
    /// Notas del presentador.
    pub notes: Notes,
    /// Panel de notas visible.
    pub notes_visible: bool,
    /// Proyección (ventana de audiencia) activa.
    pub projection: bool,
    doc: Option<Arc<PagedDocument>>,
    cache: HashMap<usize, Arc<RenderImage>>,
    /// Orden de uso de la caché (LRU): el frente es el menos usado recientemente.
    cache_order: VecDeque<usize>,
    /// Páginas cuya rasterización está en curso (en segundo plano).
    pending: HashSet<usize>,
}

impl Deck {
    /// Deck vacío sin documento.
    pub fn empty() -> Self {
        Self {
            page: 0,
            total: 0,
            dark: false,
            error: None,
            file_path: None,
            typ_content: None,
            notes: Notes::new(),
            notes_visible: true,
            projection: true,
            doc: None,
            cache: HashMap::new(),
            cache_order: VecDeque::new(),
            pending: HashSet::new(),
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
        if let Err(msg) = deck.open_file(path) {
            deck.error = Some(msg);
        }
        deck
    }

    /// Abre (o reabre) un documento desde `path`, reconstruyendo notas y caché.
    pub fn open_file(&mut self, path: &str) -> Result<(), String> {
        let text =
            std::fs::read_to_string(path).map_err(|e| format!("Error leyendo fichero: {e}"))?;
        let world = crate::TypstWorld::new(&text, path);
        match typst::compile::<PagedDocument>(&world).output {
            Ok(doc) => {
                self.total = doc.pages().len();
                self.doc = Some(Arc::new(doc));
                self.cache.clear();
                self.cache_order.clear();
                self.pending.clear();
                self.page = 0;
                self.file_path = Some(path.to_string());
                self.notes = Notes::with_db(path, &text, self.total);
                self.typ_content = Some(text);
                self.error = None;
                Ok(())
            }
            Err(diags) => {
                let msg: Vec<String> = diags.iter().map(|d| format!("{d:?}")).collect();
                let msg = format!("Error compilando Typst:\n{}", msg.join("\n"));
                self.error = Some(msg.clone());
                Err(msg)
            }
        }
    }

    /// Reconstruye las notas a partir del documento Typst cargado.
    pub fn reimport_notes(&mut self) -> Result<(), String> {
        let Some(content) = self.typ_content.clone() else {
            return Err("No hay documento cargado".to_string());
        };
        let total = self.total;
        self.notes
            .reimport(&content, total)
            .map_err(|e| e.to_string())
    }

    /// Sincroniza el estado de las notas tras cambiar de página.
    fn after_nav(&mut self) {
        if self.notes.editing
            && self.notes.dirty
            && let Err(e) = self.notes.save_current()
        {
            warn!("No se pudo guardar la nota al navegar: {e}");
        }
        self.notes.load_page(self.page);
    }

    /// Va a la primera página.
    pub fn first(&mut self) {
        self.page = 0;
        self.after_nav();
    }

    /// Retrocede una página (sin salir del rango).
    pub fn prev(&mut self) {
        if self.page > 0 {
            self.page -= 1;
        }
        self.after_nav();
    }

    /// Avanza una página (sin salir del rango).
    pub fn next(&mut self) {
        if self.page + 1 < self.total {
            self.page += 1;
        }
        self.after_nav();
    }

    /// Va a la última página.
    pub fn last(&mut self) {
        if self.total > 0 {
            self.page = self.total - 1;
        }
        self.after_nav();
    }

    /// Alterna el tema claro/oscuro.
    pub fn toggle_theme(&mut self) {
        self.dark = !self.dark;
    }

    /// Alterna la visibilidad del panel de notas.
    pub fn toggle_notes(&mut self) {
        self.notes_visible = !self.notes_visible;
    }

    /// Índice de la página siguiente, si existe.
    pub fn next_index(&self) -> Option<usize> {
        let n = self.page + 1;
        (n < self.total).then_some(n)
    }

    /// `true` si se está editando algo (cuerpo de nota o título).
    pub fn is_editing(&self) -> bool {
        self.notes.editing || self.notes.heading_editing
    }

    /// `true` si hay página anterior.
    pub fn has_prev(&self) -> bool {
        self.page > 0
    }

    /// `true` si hay página siguiente.
    pub fn has_next(&self) -> bool {
        self.next_index().is_some()
    }

    /// Devuelve la imagen de la página `idx` si ya está rasterizada.
    ///
    /// Si no lo está, lanza su rasterización en **segundo plano** (fuera del hilo
    /// de UI) y devuelve `None`; al terminar, la inserta en la caché y notifica.
    /// Así el hilo de UI nunca bloquea al rasterizar una página.
    pub fn image_at(&mut self, idx: usize, cx: &mut Context<Self>) -> Option<Arc<RenderImage>> {
        if let Some(image) = self.cache.get(&idx).cloned() {
            self.touch(idx);
            return Some(image);
        }
        let Some(doc) = self.doc.clone() else {
            return None;
        };
        if idx >= doc.pages().len() {
            return None;
        }
        if self.pending.insert(idx) {
            cx.spawn(async move |this, cx| {
                let rendered = cx
                    .background_executor()
                    .spawn(async move { render_page(&doc, idx) })
                    .await;
                this.update(cx, |deck, cx| {
                    deck.pending.remove(&idx);
                    if let Some((width, height, data)) = rendered {
                        deck.insert_cached(idx, width, height, data, cx);
                    }
                    cx.notify();
                })
                .ok();
            })
            .detach();
        }
        None
    }

    /// Marca `idx` como la entrada más recientemente usada de la caché.
    fn touch(&mut self, idx: usize) {
        if let Some(pos) = self.cache_order.iter().position(|&i| i == idx) {
            self.cache_order.remove(pos);
        }
        self.cache_order.push_back(idx);
    }

    /// Inserta en la caché una página ya rasterizada, desalojando la LRU si hace falta.
    fn insert_cached(
        &mut self,
        idx: usize,
        width: u32,
        height: u32,
        data: Vec<u8>,
        cx: &mut Context<Self>,
    ) {
        // Evita duplicados si dos peticiones solapadas acabaron a la vez.
        if self.cache.contains_key(&idx) {
            return;
        }
        let Some(rgba) = image::RgbaImage::from_raw(width, height, data) else {
            warn!("No se pudo construir la imagen de la página {idx}");
            return;
        };
        let image = Arc::new(RenderImage::new(vec![image::Frame::new(rgba)]));
        // Desaloja LRU mientras se supere el límite, liberando también la textura GPU.
        while self.cache.len() >= CACHE_CAP {
            match eviction_candidate(&self.cache_order, idx) {
                Some(victim) => {
                    self.cache_order.retain(|&i| i != victim);
                    if let Some(old) = self.cache.remove(&victim) {
                        cx.drop_image(old, None);
                    }
                }
                None => break,
            }
        }
        self.cache.insert(idx, image);
        self.cache_order.push_back(idx);
    }

    /// Imagen de la página actual (o `None` si aún se está rasterizando).
    pub fn current_image(&mut self, cx: &mut Context<Self>) -> Option<Arc<RenderImage>> {
        self.image_at(self.page, cx)
    }

    /// Imagen de la página siguiente, si existe (o `None` si aún se rasteriza).
    pub fn next_image(&mut self, cx: &mut Context<Self>) -> Option<Arc<RenderImage>> {
        let idx = self.next_index()?;
        self.image_at(idx, cx)
    }

    /// `true` si la página `idx` se está rasterizando ahora mismo.
    pub fn is_pending(&self, idx: usize) -> bool {
        self.pending.contains(&idx)
    }
}

/// Rasteriza la página `idx` a RGBA premultiplicado (BGRA no premultiplicado).
///
/// Función pura, pensada para ejecutarse **fuera del hilo de UI**.
fn render_page(doc: &PagedDocument, idx: usize) -> Option<(u32, u32, Vec<u8>)> {
    let page = doc.pages().get(idx)?;
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
    Some((width, height, data))
}

/// Candidato a desalojar de la caché LRU: la entrada más antigua que no sea `keep`.
///
/// Devuelve `None` si el orden está vacío o solo contiene `keep`.
pub fn eviction_candidate(order: &VecDeque<usize>, keep: usize) -> Option<usize> {
    order.iter().copied().find(|&i| i != keep)
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
    fn navigation_flags() {
        let mut d = deck_with(3);
        assert!(!d.has_prev());
        assert!(d.has_next());
        d.last();
        assert!(d.has_prev());
        assert!(!d.has_next());
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
    fn toggle_notes_flips() {
        let mut d = Deck::empty();
        assert!(d.notes_visible);
        d.toggle_notes();
        assert!(!d.notes_visible);
    }

    #[test]
    fn load_invalid_path_sets_error() {
        let d = Deck::load("/nonexistent/no-existe-jamas.typ");
        assert!(d.error.is_some(), "debe registrar error de lectura");
        assert_eq!(d.total, 0);
        assert!(d.doc.is_none());
    }

    #[test]
    fn load_sets_path_and_total() {
        let path = std::env::temp_dir().join("typshow_deck_test.typ");
        std::fs::write(&path, "= Hola\n\nTexto de prueba\n").unwrap();
        let d = Deck::load(path.to_str().unwrap());
        assert!(d.error.is_none(), "error inesperado: {:?}", d.error);
        assert!(d.total >= 1, "debe tener al menos una pagina");
        assert_eq!(d.file_path.as_deref(), path.to_str());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn render_page_produces_rgba_pixels() {
        let path = std::env::temp_dir().join("typshow_deck_render_test.typ");
        std::fs::write(&path, "= Hola\n\nTexto de prueba\n").unwrap();
        let d = Deck::load(path.to_str().unwrap());
        assert!(d.error.is_none(), "error inesperado: {:?}", d.error);
        let doc = d.doc.as_ref().expect("documento compilado");
        let (w, h, data) = render_page(doc, 0).expect("render de la pagina 0");
        assert!(w > 0 && h > 0, "dimensiones validas");
        assert_eq!(data.len(), (w as usize) * (h as usize) * 4, "RGBA completo");
        assert!(
            render_page(doc, d.total).is_none(),
            "fuera de rango -> None"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn eviction_prefers_oldest_but_keeps_current() {
        let order: VecDeque<usize> = [0, 1, 2].into_iter().collect();
        assert_eq!(eviction_candidate(&order, 2), Some(0), "el mas antiguo");
        assert_eq!(eviction_candidate(&order, 0), Some(1), "nunca la actual");
        let single: VecDeque<usize> = [5].into_iter().collect();
        assert_eq!(eviction_candidate(&single, 5), None, "no desaloja la unica");
        let empty: VecDeque<usize> = VecDeque::new();
        assert_eq!(eviction_candidate(&empty, 0), None);
    }

    #[test]
    fn cache_cap_is_positive() {
        assert!(CACHE_CAP >= 2, "debe caber al menos actual + siguiente");
    }

    #[test]
    fn rgba_to_bgra_unpremultiplies() {
        // Opaco: solo cambia el orden de canales.
        let mut data = vec![255, 0, 0, 255];
        rgba_to_bgra(&mut data);
        assert_eq!(data, [0, 0, 255, 255]);

        // Rojo premultiplicado con alpha 128 -> rojo a plena intensidad.
        let mut data = vec![128, 0, 0, 128];
        rgba_to_bgra(&mut data);
        assert_eq!(data, [0, 0, 254, 128]);

        // Totalmente transparente: no se toca.
        let mut data = vec![0, 0, 0, 0];
        rgba_to_bgra(&mut data);
        assert_eq!(data, [0, 0, 0, 0]);

        // Caso general premultiplicado (cada canal <= alpha).
        let mut data = vec![10, 20, 30, 40];
        rgba_to_bgra(&mut data);
        assert_eq!(data, [191, 127, 63, 40]);
    }
}
