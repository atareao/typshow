//! Estado compartido y render de páginas de la UI GPUI.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use gpui::{AppContext, Context, RenderImage};
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
    /// Hay una compilación Typst en curso (apertura asíncrona desde la UI).
    pub loading: bool,
    /// Generación del documento: se incrementa en cada carga para descartar
    /// resultados (compilaciones o rasterizaciones) de documentos obsoletos.
    doc_generation: u64,
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
            loading: false,
            doc_generation: 0,
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
        // `open_file` ya deja el mensaje de error en el deck si falla.
        let _ = deck.open_file(path);
        deck
    }

    /// Marca el inicio de una carga y devuelve la nueva generación de documento.
    ///
    /// Incrementa `doc_generation` (invalidando compilaciones y rasterizaciones
    /// en vuelo), activa `loading` y limpia el error previo.
    fn begin_load(&mut self) -> u64 {
        self.doc_generation = self.doc_generation.wrapping_add(1);
        self.loading = true;
        self.error = None;
        self.doc_generation
    }

    /// `true` si `generation` corresponde al documento actual.
    fn accepts_generation(&self, generation: u64) -> bool {
        generation == self.doc_generation
    }

    /// Abre (o reabre) un documento desde `path`, reconstruyendo notas y caché.
    ///
    /// Variante síncrona, pensada para el arranque por CLI. La UI debe usar
    /// [`Deck::open_file_async`] para no bloquear el hilo de UI.
    pub fn open_file(&mut self, path: &str) -> Result<(), String> {
        let generation = self.begin_load();
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) => {
                let msg = format!("Error leyendo fichero: {e}");
                self.loading = false;
                self.error = Some(msg.clone());
                return Err(msg);
            }
        };
        let result = compile_typst(&text, path);
        // Sin `cx` no hay texturas que liberar: la caché está vacía al arrancar.
        self.apply_compiled(generation, path, text, None, result, None)
    }

    /// Abre un documento en **segundo plano**, sin bloquear el hilo de UI.
    ///
    /// Marca `loading`, limpia el error previo y lanza la lectura, la compilación
    /// y la apertura de la base de datos de notas en el executor de fondo; al
    /// terminar aplica el resultado en el hilo de UI mediante
    /// [`Deck::apply_compiled`] y notifica. Si mientras tanto se pidió otra
    /// apertura, el resultado se descarta por generación.
    pub fn open_file_async(&mut self, path: &str, cx: &mut Context<Self>) {
        let generation = self.begin_load();
        cx.notify();
        let path = path.to_string();
        cx.spawn(async move |this, cx| {
            let (path, text, notes, result) = cx
                .background_spawn(async move {
                    let text = match std::fs::read_to_string(&path) {
                        Ok(text) => text,
                        Err(e) => {
                            let msg = format!("Error leyendo fichero: {e}");
                            return (path, String::new(), None, Err(msg));
                        }
                    };
                    let result = compile_typst(&text, &path);
                    // SQLite + migración de notas fuera del hilo de UI.
                    let notes = match &result {
                        Ok(doc) => Some(Notes::with_db(&path, &text, doc.pages().len())),
                        Err(_) => None,
                    };
                    (path, text, notes, result)
                })
                .await;
            this.update(cx, |deck, cx| {
                let _ = deck.apply_compiled(generation, &path, text, notes, result, Some(cx));
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Aplica el resultado de compilar `text` (leído de `path`) al estado del deck.
    ///
    /// Es **no-op** si `generation` no es la generación actual (llegó tarde una carga
    /// anterior). En éxito resetea página, caché y notas; en error registra el
    /// mensaje y **conserva** el documento y la caché anteriores para no dejar la
    /// presentación en blanco. En ambos casos termina el estado de carga.
    fn apply_compiled(
        &mut self,
        generation: u64,
        path: &str,
        text: String,
        notes: Option<Notes>,
        result: Result<PagedDocument, String>,
        cx: Option<&mut Context<Self>>,
    ) -> Result<(), String> {
        if !self.accepts_generation(generation) {
            return Ok(());
        }
        self.loading = false;
        match result {
            Ok(doc) => {
                let total = doc.pages().len();
                self.clear_cache(cx);
                self.total = total;
                self.doc = Some(Arc::new(doc));
                self.page = 0;
                self.file_path = Some(path.to_string());
                let mut notes = notes.unwrap_or_else(|| Notes::with_db(path, &text, total));
                notes.total_pages = total;
                self.notes = notes;
                self.typ_content = Some(text);
                self.error = None;
                Ok(())
            }
            Err(msg) => {
                self.error = Some(msg.clone());
                Err(msg)
            }
        }
    }

    /// Vacía la caché de páginas, liberando las texturas GPU si hay `cx`.
    fn clear_cache(&mut self, cx: Option<&mut Context<Self>>) {
        match cx {
            Some(cx) => {
                for (_, image) in self.cache.drain() {
                    cx.drop_image(image, None);
                }
            }
            None => self.cache.clear(),
        }
        self.cache_order.clear();
        self.pending.clear();
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
        // Mientras se compila un documento nuevo no se rasteriza el anterior:
        // está a punto de ser reemplazado y el trabajo se tiraría.
        if self.loading {
            return None;
        }
        if let Some(image) = self.cache.get(&idx).cloned() {
            self.touch(idx);
            return Some(image);
        }
        let doc = self.doc.clone()?;
        if idx >= doc.pages().len() {
            return None;
        }
        let generation = self.doc_generation;
        if self.pending.insert(idx) {
            cx.spawn(async move |this, cx| {
                let rendered = cx
                    .background_executor()
                    .spawn(async move { render_page(&doc, idx) })
                    .await;
                this.update(cx, |deck, cx| {
                    // Descarta el resultado si el documento cambió mientras tanto.
                    if !deck.accepts_generation(generation) {
                        return;
                    }
                    deck.pending.remove(&idx);
                    if let Some((width, height, data)) = rendered {
                        for old in deck.insert_cached(generation, idx, width, height, data) {
                            cx.drop_image(old, None);
                        }
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
    ///
    /// Es no-op si `generation` no es la generación actual. Devuelve las imágenes
    /// desalojadas para que el llamador libere sus texturas GPU.
    fn insert_cached(
        &mut self,
        generation: u64,
        idx: usize,
        width: u32,
        height: u32,
        data: Vec<u8>,
    ) -> Vec<Arc<RenderImage>> {
        let mut evicted = Vec::new();
        if !self.accepts_generation(generation) {
            return evicted;
        }
        // Evita duplicados si dos peticiones solapadas acabaron a la vez.
        if self.cache.contains_key(&idx) {
            return evicted;
        }
        let Some(rgba) = image::RgbaImage::from_raw(width, height, data) else {
            warn!("No se pudo construir la imagen de la página {idx}");
            return evicted;
        };
        let image = Arc::new(RenderImage::new(vec![image::Frame::new(rgba)]));
        // Desaloja LRU mientras se supere el límite.
        while self.cache.len() >= CACHE_CAP {
            match eviction_candidate(&self.cache_order, idx) {
                Some(victim) => {
                    self.cache_order.retain(|&i| i != victim);
                    if let Some(old) = self.cache.remove(&victim) {
                        evicted.push(old);
                    }
                }
                None => break,
            }
        }
        self.cache.insert(idx, image);
        self.cache_order.push_back(idx);
        evicted
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

/// Compila `text` como documento Typst paginado.
///
/// Función pura (sin estado de UI), apta para ejecutarse **fuera del hilo de UI**.
/// Un pánico del compilador se captura para que `loading` nunca quede colgado.
fn compile_typst(text: &str, path: &str) -> Result<PagedDocument, String> {
    let compiled = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let world = crate::TypstWorld::new(text, path);
        typst::compile::<PagedDocument>(&world).output
    }));
    match compiled {
        Ok(Ok(doc)) => Ok(doc),
        Ok(Err(diags)) => {
            let msg: Vec<String> = diags.iter().map(|d| format!("{d:?}")).collect();
            Err(format!("Error compilando Typst:\n{}", msg.join("\n")))
        }
        Err(_) => Err("Error interno compilando Typst".to_string()),
    }
}

/// Rasteriza la página `idx` y devuelve BGRA no premultiplicado (listo para `RenderImage`).
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
/// canónica de GPUI (evita bugs si cambia el formato interno). Esa función opera
/// sobre **un solo píxel**, así que se invoca una vez por cada píxel del buffer
/// (mismo patrón que el propio `svg_renderer` de GPUI).
pub fn rgba_to_bgra(data: &mut [u8]) {
    debug_assert_eq!(
        data.len() % 4,
        0,
        "el buffer debe ser RGBA (longitud múltiplo de 4)"
    );
    for pixel in data.chunks_exact_mut(4) {
        gpui::swap_rgba_pa_to_bgra(pixel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deck_with(total: usize) -> Deck {
        let mut d = Deck::empty();
        d.total = total;
        d
    }

    /// Elimina la base de datos de notas (y sus sidecars WAL/SHM) de un `.typ` de prueba.
    fn cleanup_db(typ_path: &std::path::Path) {
        for ext in ["typshow.db", "typshow.db-wal", "typshow.db-shm"] {
            let _ = std::fs::remove_file(typ_path.with_extension(ext));
        }
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
        cleanup_db(&path);
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
        cleanup_db(&path);
    }

    /// Compila `text` en un `PagedDocument` de prueba (mismo camino que `open_file`).
    fn compiled(text: &str) -> PagedDocument {
        let world = crate::TypstWorld::new(text, "/tmp/typshow_deck_apply_test.typ");
        typst::compile::<PagedDocument>(&world)
            .output
            .expect("el Typst de prueba debe compilar")
    }

    /// Inserta una entrada ficticia en la caché y en `pending` para comprobar su limpieza.
    fn seed_cache(d: &mut Deck) {
        let rgba = image::RgbaImage::from_raw(1, 1, vec![0, 0, 0, 255]).expect("rgba 1x1");
        d.cache
            .insert(0, Arc::new(RenderImage::new(vec![image::Frame::new(rgba)])));
        d.cache_order.push_back(0);
        d.pending.insert(0);
    }

    #[test]
    fn apply_compiled_ok_resets_state() {
        let mut d = Deck::empty();
        d.page = 2;
        d.total = 5;
        d.error = Some("error previo".to_string());
        d.file_path = Some("/tmp/viejo.typ".to_string());
        d.typ_content = Some("= Viejo".to_string());
        seed_cache(&mut d);

        let text = "= Hola\n\nTexto de prueba\n".to_string();
        let doc = compiled(&text);
        let expected_total = doc.pages().len();
        let generation = d.begin_load();
        let r = d.apply_compiled(
            generation,
            "/tmp/nuevo.typ",
            text.clone(),
            Some(Notes::new()),
            Ok(doc),
            None,
        );
        assert!(r.is_ok());

        assert_eq!(d.total, expected_total, "total del documento nuevo");
        assert_eq!(d.page, 0, "vuelve a la primera página");
        assert_eq!(d.file_path.as_deref(), Some("/tmp/nuevo.typ"));
        assert_eq!(d.typ_content.as_deref(), Some(text.as_str()));
        assert!(d.doc.is_some(), "documento aplicado");
        assert!(d.error.is_none(), "limpia el error previo");
        assert!(d.cache.is_empty(), "limpia la caché");
        assert!(d.cache_order.is_empty(), "limpia el orden LRU");
        assert!(
            d.pending.is_empty(),
            "limpia las rasterizaciones pendientes"
        );
        assert_eq!(d.notes.total_pages, expected_total, "notas reconstruidas");
    }

    #[test]
    fn apply_compiled_error_keeps_previous_doc() {
        let mut d = Deck::empty();
        let text = "= Hola\n\nTexto de prueba\n".to_string();
        let doc = compiled(&text);
        let previous_total = doc.pages().len();
        let generation = d.begin_load();
        d.apply_compiled(
            generation,
            "/tmp/bueno.typ",
            text,
            Some(Notes::new()),
            Ok(doc),
            None,
        )
        .expect("aplica el documento bueno");
        assert!(d.doc.is_some());

        let generation = d.begin_load();
        let r = d.apply_compiled(
            generation,
            "/tmp/malo.typ",
            "= Roto".to_string(),
            None,
            Err("Error compilando Typst: roto".to_string()),
            None,
        );
        assert!(r.is_err(), "el error se propaga");

        assert_eq!(
            d.error.as_deref(),
            Some("Error compilando Typst: roto"),
            "registra el error"
        );
        assert!(d.doc.is_some(), "conserva el documento anterior");
        assert_eq!(d.total, previous_total, "conserva el total anterior");
        assert_eq!(
            d.file_path.as_deref(),
            Some("/tmp/bueno.typ"),
            "conserva la ruta anterior"
        );
    }

    #[test]
    fn apply_compiled_clears_loading() {
        let mut d = Deck::empty();
        let generation = d.begin_load();
        assert!(d.loading);
        let text = "= Hola\n\nTexto de prueba\n".to_string();
        d.apply_compiled(
            generation,
            "/tmp/ok.typ",
            text,
            Some(Notes::new()),
            Ok(compiled("= Hola\n\nTexto de prueba\n")),
            None,
        )
        .expect("aplica el documento");
        assert!(!d.loading, "el éxito termina la carga");

        let generation = d.begin_load();
        assert!(d.loading);
        d.apply_compiled(
            generation,
            "/tmp/ko.typ",
            "= Roto".to_string(),
            None,
            Err("Error compilando Typst: roto".to_string()),
            None,
        )
        .expect_err("el error se propaga");
        assert!(!d.loading, "el error también termina la carga");
    }

    #[test]
    fn apply_compiled_error_keeps_cache() {
        let mut d = Deck::empty();
        seed_cache(&mut d);
        let generation = d.begin_load();
        let r = d.apply_compiled(
            generation,
            "/tmp/malo.typ",
            "= Roto".to_string(),
            None,
            Err("boom".to_string()),
            None,
        );
        assert!(r.is_err());
        assert_eq!(d.cache.len(), 1, "conserva la caché anterior");
        assert_eq!(d.cache_order.len(), 1, "conserva el orden LRU");
    }

    #[test]
    fn apply_compiled_zero_pages() {
        let mut d = Deck::empty();
        d.page = 3;
        d.total = 7;
        let generation = d.begin_load();
        let doc = PagedDocument::new(Default::default(), Default::default());
        let r = d.apply_compiled(
            generation,
            "/tmp/vacio.typ",
            String::new(),
            Some(Notes::new()),
            Ok(doc),
            None,
        );
        assert!(r.is_ok());
        assert_eq!(d.total, 0, "documento sin páginas");
        assert_eq!(d.page, 0, "vuelve a la primera");
        assert!(d.doc.is_some(), "el documento vacío se aplica");
        assert!(!d.has_next(), "sin páginas no hay siguiente");
        assert_eq!(d.next_index(), None);
        assert_eq!(d.notes.total_pages, 0);
    }

    #[test]
    fn stale_generation_is_ignored() {
        let mut d = Deck::empty();
        let old = d.begin_load();
        let new = d.begin_load();
        let text = "= Hola\n\nTexto de prueba\n".to_string();
        let r = d.apply_compiled(
            old,
            "/tmp/viejo.typ",
            text.clone(),
            Some(Notes::new()),
            Ok(compiled(&text)),
            None,
        );
        assert!(r.is_ok(), "una generación obsoleta no es un error");
        assert!(d.doc.is_none(), "no aplica el documento viejo");
        assert_eq!(d.doc_generation, new, "la generación vigente no cambia");
        assert!(d.loading, "la carga vigente sigue en curso");
    }

    #[test]
    fn insert_cached_ignores_stale_generation() {
        let mut d = Deck::empty();
        let old = d.begin_load();
        let _new = d.begin_load();
        let evicted = d.insert_cached(old, 0, 1, 1, vec![0, 0, 0, 255]);
        assert!(evicted.is_empty());
        assert!(d.cache.is_empty(), "no inserta con generación obsoleta");
        let evicted = d.insert_cached(d.doc_generation, 0, 1, 1, vec![0, 0, 0, 255]);
        assert!(evicted.is_empty());
        assert!(d.cache.contains_key(&0), "inserta con la generación actual");
    }

    #[test]
    fn begin_load_sets_loading_and_clears_error() {
        let mut d = Deck::empty();
        d.error = Some("previo".to_string());
        let g1 = d.begin_load();
        assert!(d.loading, "activa el estado de carga");
        assert!(d.error.is_none(), "limpia el error previo");
        let g2 = d.begin_load();
        assert_eq!(g2, g1 + 1, "cada carga incrementa la generación");
    }

    #[test]
    fn open_file_ok_leaves_loading_false() {
        let path = std::env::temp_dir().join("typshow_deck_ok_test.typ");
        std::fs::write(&path, "= Hola\n\nTexto de prueba\n").unwrap();
        let mut d = Deck::empty();
        let r = d.open_file(path.to_str().unwrap());
        assert!(r.is_ok(), "error inesperado: {r:?}");
        assert!(!d.loading, "termina la carga");
        assert!(d.error.is_none(), "sin error");
        assert!(d.total >= 1, "debe tener al menos una pagina");
        let _ = std::fs::remove_file(&path);
        cleanup_db(&path);
    }

    #[test]
    fn open_file_invalid_typst_returns_err() {
        let path = std::env::temp_dir().join("typshow_deck_invalid_test.typ");
        std::fs::write(&path, "#let x = \n").unwrap();
        let mut d = Deck::empty();
        let r = d.open_file(path.to_str().unwrap());
        assert!(r.is_err(), "Typst inválido debe devolver Err");
        assert!(d.error.is_some(), "registra el error");
        assert!(!d.loading, "termina la carga");
        assert!(d.doc.is_none(), "no hay documento");
        let _ = std::fs::remove_file(&path);
        cleanup_db(&path);
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
        const { assert!(CACHE_CAP >= 2, "debe caber al menos actual + siguiente") };
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

    #[test]
    fn rgba_to_bgra_converts_every_pixel() {
        // Dos píxeles opacos con R y B distintos: rojo y azul. Cada uno debe
        // intercambiar R y B, no solo el primero.
        // Nota: el verde (0,255,0) es punto fijo del intercambio, así que no
        // sirve para distinguir "solo el primer píxel" de "todos los píxeles".
        let mut data = vec![255, 0, 0, 255, 0, 0, 255, 255];
        rgba_to_bgra(&mut data);
        assert_eq!(
            data,
            [0, 0, 255, 255, 255, 0, 0, 255],
            "cada píxel debe tener R y B intercambiados"
        );
    }

    #[test]
    fn render_page_preserves_colors() {
        let path = std::env::temp_dir().join("typshow_deck_render_colors_test.typ");
        std::fs::write(
            &path,
            "#set page(margin: 0pt)\n#rect(width: 100%, height: 100%, fill: rgb(\"#ffff00\"))\n",
        )
        .unwrap();
        let d = Deck::load(path.to_str().unwrap());
        assert!(d.error.is_none(), "error inesperado: {:?}", d.error);
        let doc = d.doc.as_ref().expect("documento compilado");
        let (w, h, data) = render_page(doc, 0).expect("render de la pagina 0");
        assert_eq!(data.len(), (w as usize) * (h as usize) * 4, "RGBA completo");
        let amarillo_bgra = [0u8, 255, 255, 255];
        for (i, pixel) in data.chunks_exact(4).enumerate() {
            assert_eq!(
                pixel,
                amarillo_bgra,
                "pixel {i} de {} no es amarillo BGRA (desfase de canales)",
                w as usize * h as usize
            );
        }
        let _ = std::fs::remove_file(&path);
        cleanup_db(&path);
    }
}
