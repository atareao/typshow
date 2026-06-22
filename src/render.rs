use egui::{ColorImage, TextureHandle};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::OnceLock;
use std::thread;
use pdf_render::pdf_syntax::Pdf;
use pdf_render::{render, RenderSettings, pdf_interpret::InterpreterSettings};
use typst::diag::FileResult;
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{FileId, Source};
use typst::text::{Font, FontBook};
use typst::Library;
use typst::LibraryExt;
use typst::utils::{LazyHash, Scalar};
use typst_layout::PagedDocument;
use tracing::{info, debug, error};

// ── Font system (lazy, once) ──────────────────────────────────────────

static SYSTEM_FONTS: OnceLock<(LazyHash<FontBook>, Vec<Font>)> = OnceLock::new();

fn get_system_fonts() -> &'static (LazyHash<FontBook>, Vec<Font>) {
    SYSTEM_FONTS.get_or_init(|| {
        let start = std::time::Instant::now();

        let mut fonts: Vec<Font> = typst_assets::fonts()
            .flat_map(|data| Font::iter(Bytes::new(data.to_vec())))
            .collect();

        let mut db = fontdb::Database::new();
        db.load_system_fonts();

        let targets = [
            "cantarell", "roboto", "inter", "arial", "helvetica",
            "liberation sans", "noto sans", "dejavu sans",
            "liberation serif", "noto serif", "dejavu serif",
            "liberation mono", "dejavu sans mono", "noto mono",
        ];

        for face in db.faces() {
            let matches = face.families.iter().any(|(family, _)| {
                let lower = family.to_lowercase();
                targets.iter().any(|t| lower.contains(t))
            });
            if matches {
                if let fontdb::Source::File(ref path) = face.source {
                    if let Ok(data) = std::fs::read(path) {
                        if let Some(font) = Font::new(Bytes::new(data), face.index) {
                            fonts.push(font);
                        }
                    }
                }
            }
        }

        let book = FontBook::from_fonts(&fonts);
        info!("Loaded {} total fonts in {:?}", fonts.len(), start.elapsed());
        (LazyHash::new(book), fonts)
    })
}

// ── Typst World ──────────────────────────────────────────────────────

pub struct TypstWorld {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    main_id: FileId,
    source: Source,
    root: std::path::PathBuf,
    packages: Vec<std::path::PathBuf>,
}

impl TypstWorld {
    pub fn new(text: &str, file_path: &str) -> Self {
        let (book, fonts) = get_system_fonts().clone();
        let vpath = typst::syntax::VirtualPath::new("main.typ").unwrap();
        let main_id = FileId::new(
            typst::syntax::RootedPath::new(typst::syntax::VirtualRoot::Project, vpath)
        );
        let source = Source::new(main_id, text.to_string());
        let root = Path::new(file_path).parent().unwrap_or(Path::new(".")).to_path_buf();

        let packages = Self::discover_package_dirs();

        Self {
            library: LazyHash::new(Library::default()),
            book,
            fonts,
            main_id,
            source,
            root,
            packages,
        }
    }

    fn discover_package_dirs() -> Vec<std::path::PathBuf> {
        let mut dirs = Vec::new();

        if let Ok(env_pkg) = std::env::var("TYPST_PACKAGES_DIR") {
            dirs.push(PathBuf::from(env_pkg));
        }

        if let Ok(data_home) = std::env::var("XDG_DATA_HOME") {
            dirs.push(PathBuf::from(data_home).join("typst/packages"));
        } else if let Ok(home) = std::env::var("HOME") {
            dirs.push(PathBuf::from(home).join(".local/share/typst/packages"));
        }

        if let Ok(cache_home) = std::env::var("XDG_CACHE_HOME") {
            dirs.push(PathBuf::from(cache_home).join("typst/packages"));
        } else if let Ok(home) = std::env::var("HOME") {
            dirs.push(PathBuf::from(home).join(".cache/typst/packages"));
        }

        dirs
    }

    fn resolve_path(&self, id: FileId) -> FileResult<std::path::PathBuf> {
        let vpath = id.vpath().get_with_slash().trim_start_matches('/');
        match id.root() {
            typst::syntax::VirtualRoot::Project => {
                Ok(self.root.join(vpath))
            }
            typst::syntax::VirtualRoot::Package(spec) => {
                let relative = format!("{}/{}/{}/{}", spec.namespace, spec.name, spec.version, vpath);
                for dir in &self.packages {
                    let candidate = dir.join(&relative);
                    if candidate.exists() {
                        return Ok(candidate);
                    }
                }
                Err(typst::diag::FileError::Package(
                    typst::diag::PackageError::NotFound(spec.clone()),
                ))
            }
        }
    }
}

impl typst::World for TypstWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.main_id
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main_id {
            Ok(self.source.clone())
        } else {
            let path = self.resolve_path(id)?;
            let content = std::fs::read_to_string(&path)
                .map_err(|e| typst::diag::FileError::from_io(e, &path))?;
            Ok(Source::new(id, content))
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        let path = self.resolve_path(id)?;
        let data = std::fs::read(&path)
            .map_err(|e| typst::diag::FileError::from_io(e, &path))?;
        Ok(Bytes::new(data))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    fn today(&self, _offset: Option<Duration>) -> Option<Datetime> {
        Datetime::from_ymd(2026, 6, 22)
    }
}

// ── Dual document enum ───────────────────────────────────────────────

pub struct TypstDoc {
    pub _world: TypstWorld,
    pub document: PagedDocument,
}

pub enum LoadedDocument {
    Pdf(Pdf),
    Typst(TypstDoc),
}

// ── Render commands ──────────────────────────────────────────────────

pub enum RenderCommand {
    LoadFile(String),
    StartPrecache {
        ctx: egui::Context,
        start_page: usize,
        total_pages: usize,
    },
    PriorityRender {
        ctx: egui::Context,
        page_idx: usize,
        total_pages: usize,
    },
}

// ── Renderer ─────────────────────────────────────────────────────────

pub struct PdfRenderer {
    cache: HashMap<usize, TextureHandle>,
    tx: Sender<RenderCommand>,
    rx: Receiver<(usize, ColorImage)>,
    requested: HashSet<usize>,
    current_file_path: Option<String>,
}

impl PdfRenderer {
    pub fn new() -> Self {
        let (tx, rx_cmd) = channel::<RenderCommand>();
        let (tx_res, rx) = channel::<(usize, ColorImage)>();

        thread::spawn(move || {
            let mut loaded: Option<LoadedDocument> = None;
            let mut pending_cmd: Option<RenderCommand> = None;

            loop {
                let cmd = if let Some(c) = pending_cmd.take() {
                    c
                } else {
                    match rx_cmd.recv() {
                        Ok(c) => c,
                        Err(_) => break,
                    }
                };

                match cmd {
                    RenderCommand::LoadFile(path) => {
                        info!("Worker Thread - Cargando documento: {}", path);
                        let ext = Path::new(&path)
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("");

                        match ext {
                            "typ" => {
                                match std::fs::read_to_string(&path) {
                                    Ok(content) => {
                                        let world = TypstWorld::new(&content, &path);
                                        match typst::compile::<PagedDocument>(&world).output {
                                            Ok(document) => {
                                                loaded = Some(LoadedDocument::Typst(TypstDoc { _world: world, document }));
                                                info!("Worker Thread - Typst compilado con éxito.");
                                            }
                                            Err(e) => error!("Worker Thread - Error compilando Typst: {:?}", e),
                                        }
                                    }
                                    Err(e) => error!("Worker Thread - Error leyendo Typst: {:?}", e),
                                }
                            }
                            _ => {
                                match std::fs::read(&path) {
                                    Ok(data) => {
                                        match Pdf::new(data) {
                                            Ok(pdf) => {
                                                loaded = Some(LoadedDocument::Pdf(pdf));
                                                info!("Worker Thread - PDF cargado con éxito.");
                                            }
                                            Err(e) => error!("Worker Thread - Error al analizar PDF: {:?}", e),
                                        }
                                    }
                                    Err(e) => error!("Worker Thread - Error al leer archivo: {:?}", e),
                                }
                            }
                        }
                    }
                    RenderCommand::PriorityRender { ctx, page_idx, total_pages } => {
                        if let Some(ref doc) = loaded {
                            render_single_page(&ctx, doc, page_idx, &tx_res);

                            pending_cmd = Some(RenderCommand::StartPrecache {
                                ctx: ctx.clone(),
                                start_page: page_idx + 1,
                                total_pages,
                            });
                        }
                    }
                    RenderCommand::StartPrecache { ctx, start_page, total_pages } => {
                        if let Some(ref doc) = loaded {
                            let mut current_precache = start_page;
                            while current_precache < total_pages {
                                if let Ok(new_cmd) = rx_cmd.try_recv() {
                                    debug!("Worker Thread - Interrumpiendo pre-caché para procesar comando prioritario.");
                                    pending_cmd = Some(new_cmd);
                                    break;
                                }

                                render_single_page(&ctx, doc, current_precache, &tx_res);
                                current_precache += 1;

                                std::thread::sleep(std::time::Duration::from_millis(15));
                            }
                        }
                    }
                }
            }
        });

        Self {
            cache: HashMap::new(),
            tx,
            rx,
            requested: HashSet::new(),
            current_file_path: None,
        }
    }

    pub fn get_page(
        &mut self,
        ctx: &egui::Context,
        file_path: &Option<String>,
        total_pages: usize,
        page_idx: usize,
    ) -> Option<TextureHandle> {
        while let Ok((idx, color_image)) = self.rx.try_recv() {
            let texture = ctx.load_texture(
                format!("page_{}", idx),
                color_image,
                egui::TextureOptions::LINEAR
            );
            debug!("Renderer - Recibida página {} renderizada.", idx);
            self.cache.insert(idx, texture);
            self.requested.insert(idx);
        }

        if let Some(texture) = self.cache.get(&page_idx) {
            return Some(texture.clone());
        }

        if let Some(path) = file_path {
            if self.current_file_path.as_ref() != Some(path) {
                info!("Renderer - Detectado cambio de archivo. Iniciando pre-carga completa.");
                self.current_file_path = Some(path.clone());
                self.clear_cache();
                let _ = self.tx.send(RenderCommand::LoadFile(path.clone()));
                let _ = self.tx.send(RenderCommand::StartPrecache {
                    ctx: ctx.clone(),
                    start_page: 0,
                    total_pages,
                });
            }

            if page_idx < total_pages && !self.requested.contains(&page_idx) {
                debug!("Renderer - Solicitando RENDER URGENTE para la página {}", page_idx);
                self.requested.insert(page_idx);
                let _ = self.tx.send(RenderCommand::PriorityRender {
                    ctx: ctx.clone(),
                    page_idx,
                    total_pages,
                });
            }
        }

        None
    }

    pub fn clear_cache(&mut self) {
        info!("Limpiando el caché de páginas renderizadas y peticiones.");
        self.cache.clear();
        self.requested.clear();
    }
}

// ── Render helpers ──────────────────────────────────────────────────

fn render_single_page(
    ctx: &egui::Context,
    doc: &LoadedDocument,
    page_idx: usize,
    tx_res: &Sender<(usize, ColorImage)>,
) {
    match doc {
        LoadedDocument::Pdf(pdf) => {
            if page_idx >= pdf.pages().len() {
                return;
            }

            let start_time = std::time::Instant::now();
            let page = &pdf.pages()[page_idx];
            let (w, _h) = page.render_dimensions();
            let target_width = 1920.0;
            let scale = if w > 0.0 { target_width / w } else { 1.0 };

            let render_settings = RenderSettings {
                x_scale: scale,
                y_scale: scale,
                bg_color: pdf_render::vello_cpu::color::palette::css::WHITE,
                quality: pdf_render::RasterQuality::Speed,
                ..Default::default()
            };

            let pixmap = render(page, &InterpreterSettings::default(), &render_settings);

            let width = pixmap.width() as usize;
            let height = pixmap.height() as usize;
            let pixels = pixmap.data_as_u8_slice();
            let color_image = ColorImage::from_rgba_premultiplied([width, height], pixels);

            info!(
                "Worker Thread - Página PDF {} renderizada en {:?}",
                page_idx + 1,
                start_time.elapsed()
            );

            let _ = tx_res.send((page_idx, color_image));
        }
        LoadedDocument::Typst(typst_doc) => {
            if page_idx >= typst_doc.document.pages().len() {
                return;
            }

            let start_time = std::time::Instant::now();
            let page = &typst_doc.document.pages()[page_idx];
            let pt_width = page.frame.size().x.to_pt();
            let target_width = 1920.0;
            let pixel_per_pt = if pt_width > 0.0 { target_width / pt_width } else { 2.0 };

            let options = typst_render::RenderOptions {
                pixel_per_pt: Scalar::new(pixel_per_pt),
                render_bleed: false,
            };

            let pixmap = typst_render::render(page, &options);

            let width = pixmap.width() as usize;
            let height = pixmap.height() as usize;
            let pixels = pixmap.data();
            let color_image = ColorImage::from_rgba_premultiplied([width, height], pixels);

            info!(
                "Worker Thread - Página Typst {} renderizada en {:?}",
                page_idx + 1,
                start_time.elapsed()
            );

            let _ = tx_res.send((page_idx, color_image));
        }
    }

    ctx.request_repaint();
}