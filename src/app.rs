use std::sync::Arc;
use parking_lot::Mutex;
use tracing::{info, debug};
use crate::render::Renderer;

pub struct AppState {
    pub current_page: usize,
    pub total_pages: usize,
    pub dark_mode: bool,
    pub file_path: Option<String>,
    pub renderer: Renderer,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            current_page: 0,
            total_pages: 0,
            dark_mode: false,
            file_path: None,
            renderer: Renderer::new(),
        }
    }

    pub fn load_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.renderer.load_file(path)?;
        self.total_pages = self.renderer.page_count();
        self.current_page = 0;
        self.file_path = Some(path.to_string());
        info!("Typst compilado con éxito. Total páginas: {}", self.total_pages);
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
