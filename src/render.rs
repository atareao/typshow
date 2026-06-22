use egui::{ColorImage, TextureHandle};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use typst::diag::FileResult;
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{FileId, Source};
use typst::text::{Font, FontBook};
use typst::Library;
use typst::LibraryExt;
use typst::utils::{LazyHash, Scalar};
use typst_layout::PagedDocument;
use tracing::{info, debug};

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
            "liberation sans", "noto", "dejavu sans",
            "liberation serif", "dejavu serif",
            "liberation mono", "dejavu sans mono",
            "emoji", "symbol",
            "ubuntu", "droid", "fira",
            "source han", "source sans", "source serif",
            "carlito", "caladea",
        ];

        for face in db.faces() {
            let matches = face.families.iter().any(|(family, _)| {
                let lower = family.to_lowercase();
                targets.iter().any(|t| lower.contains(t))
            });
            if matches {
                if let fontdb::Source::File(path) = &face.source {
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

// ── Typst document wrapper ──────────────────────────────────────────

pub struct TypstDoc {
    pub _world: TypstWorld,
    pub document: PagedDocument,
}

// ── Renderer ─────────────────────────────────────────────────────────

pub struct Renderer {
    cache: HashMap<usize, TextureHandle>,
    loaded_doc: Option<TypstDoc>,
    current_file_path: Option<String>,
    target_width: f64,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            loaded_doc: None,
            current_file_path: None,
            target_width: 1920.0,
        }
    }

    pub fn load_file(&mut self, path: &str) -> Result<(), String> {
        info!("Renderer - Cargando documento Typst: {}", path);
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Error leyendo archivo: {:?}", e))?;
        let world = TypstWorld::new(&content, path);
        match typst::compile::<PagedDocument>(&world).output {
            Ok(document) => {
                self.loaded_doc = Some(TypstDoc { _world: world, document });
                self.current_file_path = Some(path.to_string());
                self.clear_cache();
                info!("Renderer - Typst compilado con éxito.");
                Ok(())
            }
            Err(diags) => {
                let msg: Vec<String> = diags.iter().map(|d| format!("{:?}", d)).collect();
                Err(format!("Error compilando Typst: {}", msg.join("\n")))
            }
        }
    }

    pub fn get_page(
        &mut self,
        ctx: &egui::Context,
        page_idx: usize,
    ) -> Option<TextureHandle> {
        if let Some(texture) = self.cache.get(&page_idx) {
            return Some(texture.clone());
        }

        let doc = self.loaded_doc.as_ref()?;
        if page_idx >= doc.document.pages().len() {
            return None;
        }

        let start_time = std::time::Instant::now();
        let page = &doc.document.pages()[page_idx];
        let pt_width = page.frame.size().x.to_pt();
        let pixel_per_pt = if pt_width > 0.0 { self.target_width / pt_width } else { 2.0 };

        let options = typst_render::RenderOptions {
            pixel_per_pt: Scalar::new(pixel_per_pt as f64),
            render_bleed: false,
        };

        let pixmap = typst_render::render(page, &options);
        let width = pixmap.width() as usize;
        let height = pixmap.height() as usize;
        let color_image = ColorImage::from_rgba_premultiplied([width, height], pixmap.data());

        let texture = ctx.load_texture(
            format!("page_{}", page_idx),
            color_image,
            egui::TextureOptions::LINEAR,
        );

        debug!("Renderer - Página {} renderizada en {:?}", page_idx + 1, start_time.elapsed());
        self.cache.insert(page_idx, texture.clone());
        Some(texture)
    }

    pub fn clear_cache(&mut self) {
        info!("Limpiando caché de páginas.");
        self.cache.clear();
    }

    pub fn page_count(&self) -> usize {
        self.loaded_doc.as_ref().map_or(0, |d| d.document.pages().len())
    }
}