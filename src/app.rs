use std::sync::Arc;
use parking_lot::Mutex;
use tracing::{info, debug};
use crate::render::PdfRenderer;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DocumentSource {
    Pdf,
    Typst,
}

pub struct AppState {
    pub document: Option<DocumentSource>,
    pub current_page: usize,
    pub total_pages: usize,
    pub dark_mode: bool,
    pub file_path: Option<String>,
    pub renderer: PdfRenderer,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            document: None,
            current_page: 0,
            total_pages: 0,
            dark_mode: false,
            file_path: None,
            renderer: PdfRenderer::new(),
        }
    }

    pub fn load_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match ext {
            "typ" => {
                info!("Cargando documento Typst: {}", path);
                let content = std::fs::read_to_string(path)?;
                let world = crate::render::TypstWorld::new(&content, path);
                match typst::compile::<typst_layout::PagedDocument>(&world).output {
                    Ok(doc) => {
                        self.total_pages = doc.pages().len();
                        self.document = Some(DocumentSource::Typst);
                        info!("Typst compilado con éxito. Total páginas: {}", self.total_pages);
                    }
                    Err(diags) => {
                        let msg: Vec<String> = diags.iter().map(|d| format!("{:?}", d)).collect();
                        return Err(format!("Error compilando Typst: {}", msg.join("\n")).into());
                    }
                }
            }
            _ => {
                info!("Cargando PDF: {}", path);
                let data = std::fs::read(path)?;
                let pdf = pdf_render::pdf_syntax::Pdf::new(data)
                    .map_err(|e| format!("Error loading PDF: {:?}", e))?;
                self.total_pages = pdf.pages().len();
                self.document = Some(DocumentSource::Pdf);
                info!("PDF cargado con éxito. Total páginas: {}", self.total_pages);
            }
        }

        self.current_page = 0;
        self.file_path = Some(path.to_string());
        self.renderer.clear_cache();
        Ok(())
    }

    pub fn next_page(&mut self) {
        if self.current_page + 1 < self.total_pages {
            let old_page = self.current_page;
            self.current_page += 1;
            debug!("Transición de página: {} -> {}", old_page + 1, self.current_page + 1);
        }
    }

    pub fn prev_page(&mut self) {
        if self.current_page > 0 {
            let old_page = self.current_page;
            self.current_page -= 1;
            debug!("Transición de página: {} -> {}", old_page + 1, self.current_page + 1);
        }
    }

    pub fn first_page(&mut self) {
        let old_page = self.current_page;
        self.current_page = 0;
        debug!("Ir al inicio: {} -> 1", old_page + 1);
    }

    pub fn last_page(&mut self) {
        if self.total_pages > 0 {
            let old_page = self.current_page;
            self.current_page = self.total_pages - 1;
            debug!("Ir al final: {} -> {}", old_page + 1, self.total_pages);
        }
    }
}

pub type SharedState = Arc<Mutex<AppState>>;
