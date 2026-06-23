use std::sync::Arc;
use parking_lot::Mutex;
use tracing::{info, debug};
use crate::notes::Notes;
use crate::render::Renderer;

pub struct AppState {
    pub current_page: usize,
    pub total_pages: usize,
    pub dark_mode: bool,
    pub file_path: Option<String>,
    pub renderer: Renderer,
    pub notes: Notes,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            current_page: 0,
            total_pages: 0,
            dark_mode: false,
            file_path: None,
            renderer: Renderer::new(),
            notes: Notes::new(),
        }
    }

    pub fn load_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let typ_content = std::fs::read_to_string(path)?;

        self.renderer.load_file(path)?;
        self.total_pages = self.renderer.page_count();
        self.current_page = 0;
        self.file_path = Some(path.to_string());
        self.notes = Notes::load(path, &typ_content);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_state() {
        let s = AppState::new();
        assert_eq!(s.current_page, 0);
        assert_eq!(s.total_pages, 0);
        assert!(s.file_path.is_none());
        assert!(!s.dark_mode);
    }

    #[test]
    fn test_next_page_basic() {
        let mut s = AppState::new();
        s.total_pages = 5;
        s.current_page = 0;
        s.next_page();
        assert_eq!(s.current_page, 1);
    }

    #[test]
    fn test_next_page_boundary() {
        let mut s = AppState::new();
        s.total_pages = 3;
        s.current_page = 2;
        s.next_page();
        assert_eq!(s.current_page, 2);
    }

    #[test]
    fn test_next_page_empty() {
        let mut s = AppState::new();
        s.total_pages = 0;
        s.next_page();
        assert_eq!(s.current_page, 0);
    }

    #[test]
    fn test_prev_page_basic() {
        let mut s = AppState::new();
        s.total_pages = 5;
        s.current_page = 3;
        s.prev_page();
        assert_eq!(s.current_page, 2);
    }

    #[test]
    fn test_prev_page_boundary() {
        let mut s = AppState::new();
        s.total_pages = 5;
        s.current_page = 0;
        s.prev_page();
        assert_eq!(s.current_page, 0);
    }

    #[test]
    fn test_first_page() {
        let mut s = AppState::new();
        s.total_pages = 10;
        s.current_page = 7;
        s.first_page();
        assert_eq!(s.current_page, 0);
    }

    #[test]
    fn test_last_page() {
        let mut s = AppState::new();
        s.total_pages = 10;
        s.current_page = 0;
        s.last_page();
        assert_eq!(s.current_page, 9);
    }

    #[test]
    fn test_last_page_empty() {
        let mut s = AppState::new();
        s.total_pages = 0;
        s.current_page = 0;
        s.last_page();
        assert_eq!(s.current_page, 0);
    }
}
