//! Modelo de edición de texto puro para los editores de la UI GPUI.
//!
//! Mantiene el texto, el cursor, la selección (con dirección) y el rango marcado
//! de IME. Los offsets son **en bytes** y siempre en frontera de carácter; la
//! conversión a UTF-16 se hace solo en la frontera con GPUI
//! (`EntityInputHandler`), siguiendo la plantilla de
//! `gpui/examples/input.rs`.
//!
//! Tarea 3.11 de `openspec/changes/migracion-gpui/tasks.md`. El modelo es
//! independiente de GPUI: la ventana de presentador solo lo envuelve en un
//! `EntityInputHandler` (tarea 3.12).

use crate::gpui_ui::keys::{clamp_boundary, next_boundary, prev_boundary};

/// Estado de edición de una línea de texto (cuerpo de nota o título).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TextEdit {
    /// Texto editado.
    pub text: String,
    /// Rango seleccionado, en bytes y en frontera de carácter.
    pub selection: std::ops::Range<usize>,
    /// `true` cuando el cursor está en el extremo inicial de `selection`.
    pub reversed: bool,
    /// Rango marcado por el IME (pre-edit), en bytes.
    pub marked: Option<std::ops::Range<usize>>,
}

impl TextEdit {
    /// Crea un editor con `text` y el cursor al final.
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let end = text.len();
        Self {
            text,
            selection: end..end,
            reversed: false,
            marked: None,
        }
    }

    /// Posición del cursor en bytes: el extremo final de la selección, o el
    /// inicial cuando esta está invertida.
    pub fn cursor(&self) -> usize {
        if self.reversed {
            self.selection.start
        } else {
            self.selection.end
        }
    }

    /// Coloca el cursor en `byte`, normalizado a frontera de carácter, y
    /// colapsa la selección.
    pub fn set_cursor(&mut self, byte: usize) {
        let byte = clamp_boundary(&self.text, byte);
        self.selection = byte..byte;
        self.reversed = false;
    }

    /// Mueve el cursor al principio: colapsa la selección a su extremo inicial
    /// o retrocede un carácter.
    pub fn move_left(&mut self) {
        if self.selection.is_empty() {
            let cursor = prev_boundary(&self.text, self.cursor());
            self.set_cursor(cursor);
        } else {
            self.set_cursor(self.selection.start);
        }
    }

    /// Mueve el cursor al final: colapsa la selección a su extremo final o
    /// avanza un carácter.
    pub fn move_right(&mut self) {
        if self.selection.is_empty() {
            let cursor = next_boundary(&self.text, self.cursor());
            self.set_cursor(cursor);
        } else {
            self.set_cursor(self.selection.end);
        }
    }

    /// Mueve el cursor al inicio del texto.
    pub fn move_home(&mut self) {
        self.set_cursor(0);
    }

    /// Mueve el cursor al final del texto.
    pub fn move_end(&mut self) {
        let end = self.text.len();
        self.set_cursor(end);
    }

    /// Extiende la selección un carácter hacia la izquierda.
    pub fn select_left(&mut self) {
        let cursor = prev_boundary(&self.text, self.cursor());
        self.select_to(cursor);
    }

    /// Extiende la selección un carácter hacia la derecha.
    pub fn select_right(&mut self) {
        let cursor = next_boundary(&self.text, self.cursor());
        self.select_to(cursor);
    }

    /// Selecciona todo el texto.
    pub fn select_all(&mut self) {
        self.selection = 0..self.text.len();
        self.reversed = false;
    }

    /// Sustituye `range` (bytes) por `text`, deja el cursor al final de lo
    /// insertado y descarta el rango marcado.
    pub fn replace_range(&mut self, range: std::ops::Range<usize>, text: &str) {
        let start = clamp_boundary(&self.text, range.start);
        let end = clamp_boundary(&self.text, range.end).max(start);
        self.text.replace_range(start..end, text);
        let cursor = start + text.len();
        self.selection = cursor..cursor;
        self.reversed = false;
        self.marked = None;
    }

    /// Sustituye la selección (o el rango marcado, si hay composición IME en
    /// curso) por `text`; con `text` vacío equivale a borrarla. Réplica de
    /// `replace_text_in_range(None, ..)` de la plantilla de GPUI.
    pub fn replace_selection(&mut self, text: &str) {
        let range = self
            .marked
            .clone()
            .unwrap_or_else(|| self.selection.clone());
        self.replace_range(range, text);
    }

    /// Borra el carácter anterior al cursor, o la selección si la hay.
    pub fn backspace(&mut self) {
        if !self.selection.is_empty() {
            self.replace_selection("");
            return;
        }
        let cursor = clamp_boundary(&self.text, self.cursor());
        let prev = prev_boundary(&self.text, cursor);
        if prev != cursor {
            self.replace_range(prev..cursor, "");
        }
    }

    /// Borra el carácter siguiente al cursor, o la selección si la hay.
    pub fn delete(&mut self) {
        if !self.selection.is_empty() {
            self.replace_selection("");
            return;
        }
        let cursor = clamp_boundary(&self.text, self.cursor());
        let next = next_boundary(&self.text, cursor);
        if next != cursor {
            self.replace_range(cursor..next, "");
        }
    }

    /// Sustituye `range` (bytes) por `text`, marca lo insertado como pre-edit
    /// de IME y fija la selección provisional en UTF-16. Réplica de
    /// `replace_and_mark_text_in_range` de la plantilla de GPUI.
    pub fn replace_and_mark(
        &mut self,
        range: Option<std::ops::Range<usize>>,
        text: &str,
        selected_utf16: Option<std::ops::Range<usize>>,
    ) {
        let range = range
            .or_else(|| self.marked.clone())
            .unwrap_or_else(|| self.selection.clone());
        let start = clamp_boundary(&self.text, range.start);
        let end = clamp_boundary(&self.text, range.end).max(start);

        self.text.replace_range(start..end, text);
        self.marked = (!text.is_empty()).then_some(start..start + text.len());
        self.selection = match selected_utf16 {
            Some(selected) => {
                let insertion_start = self.offset_to_utf16(start);
                let inserted_len = text.encode_utf16().count();
                let from = insertion_start + selected.start.min(inserted_len);
                let to = insertion_start + selected.end.min(inserted_len);
                self.range_from_utf16(&(from..to))
            }
            None => {
                let cursor = start + text.len();
                cursor..cursor
            }
        };
        self.reversed = false;
    }

    /// Descarta el rango marcado de IME sin tocar el texto ni la selección.
    pub fn unmark(&mut self) {
        self.marked = None;
    }

    /// Convierte un offset en bytes a unidades UTF-16, redondeando hacia el
    /// extremo del carácter (`offset` debe estar en frontera).
    pub fn offset_to_utf16(&self, byte: usize) -> usize {
        let mut utf8 = 0;
        let mut utf16 = 0;
        for ch in self.text.chars() {
            if utf8 >= byte {
                break;
            }
            utf8 += ch.len_utf8();
            utf16 += ch.len_utf16();
        }
        utf16
    }

    /// Convierte un offset en unidades UTF-16 a bytes, redondeando hacia el
    /// extremo del carácter.
    pub fn offset_from_utf16(&self, utf16: usize) -> usize {
        let mut utf16_count = 0;
        let mut utf8 = 0;
        for ch in self.text.chars() {
            if utf16_count >= utf16 {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8 += ch.len_utf8();
        }
        utf8
    }

    /// Convierte un rango de bytes en un rango UTF-16.
    pub fn range_to_utf16(&self, range: &std::ops::Range<usize>) -> std::ops::Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    /// Convierte un rango UTF-16 en un rango de bytes.
    pub fn range_from_utf16(&self, range: &std::ops::Range<usize>) -> std::ops::Range<usize> {
        self.offset_from_utf16(range.start)..self.offset_from_utf16(range.end)
    }

    /// Longitud del texto en unidades UTF-16 (lo que espera GPUI).
    pub fn len_utf16(&self) -> usize {
        self.text.encode_utf16().count()
    }

    /// Texto seleccionado actualmente.
    pub fn selected_text(&self) -> &str {
        let start = clamp_boundary(&self.text, self.selection.start);
        let end = clamp_boundary(&self.text, self.selection.end);
        &self.text[start..end.max(start)]
    }

    /// Extiende la selección hasta `byte` manteniendo el ancla en el extremo
    /// opuesto. Misma semántica que `select_to` de la plantilla de GPUI.
    ///
    /// Público porque la ventana de presentador lo usa para el arrastre con el
    /// ratón y para `set_selected_text_range` (tarea 3.12).
    pub fn select_to(&mut self, byte: usize) {
        let cursor = clamp_boundary(&self.text, byte);
        if self.reversed {
            self.selection.start = cursor;
        } else {
            self.selection.end = cursor;
        }
        if self.selection.end < self.selection.start {
            self.reversed = !self.reversed;
            self.selection = self.selection.end..self.selection.start;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TextEdit;

    #[test]
    fn cursor_at_end_on_new() {
        let edit = TextEdit::new("hola");

        assert_eq!(edit.text, "hola");
        assert_eq!(edit.selection, 4..4);
        assert_eq!(edit.cursor(), 4);
        assert!(!edit.reversed);
        assert_eq!(edit.marked, None);
    }

    #[test]
    fn move_left_right_collapses_selection() {
        let mut edit = TextEdit::new("abcdef");

        edit.selection = 2..5;
        edit.move_left();
        assert_eq!(edit.selection, 2..2);
        assert_eq!(edit.cursor(), 2);

        edit.selection = 2..5;
        edit.move_right();
        assert_eq!(edit.selection, 5..5);
        assert_eq!(edit.cursor(), 5);

        // Con la selección colapsada, las flechas avanzan un carácter.
        edit.move_left();
        assert_eq!(edit.cursor(), 4);
        edit.move_end();
        assert_eq!(edit.cursor(), 6);
    }

    #[test]
    fn select_left_extends_and_reverses() {
        let mut edit = TextEdit::new("abcd");
        edit.set_cursor(3);

        edit.select_left();
        assert_eq!(edit.selection, 2..3);
        assert!(edit.reversed);
        assert_eq!(edit.cursor(), 2);

        edit.select_left();
        assert_eq!(edit.selection, 1..3);
        assert!(edit.reversed);
        assert_eq!(edit.cursor(), 1);
    }

    #[test]
    fn select_right_extends_forward() {
        let mut edit = TextEdit::new("abcd");
        edit.set_cursor(1);

        edit.select_right();
        assert_eq!(edit.selection, 1..2);
        assert!(!edit.reversed);
        assert_eq!(edit.cursor(), 2);

        edit.select_right();
        assert_eq!(edit.selection, 1..3);
        assert!(!edit.reversed);
        assert_eq!(edit.cursor(), 3);
    }

    #[test]
    fn select_all_covers_text() {
        let mut edit = TextEdit::new("ñ🙂z");

        edit.select_all();
        assert_eq!(edit.selection, 0..edit.text.len());
        assert_eq!(edit.selected_text(), "ñ🙂z");
        assert!(!edit.reversed);
        assert_eq!(edit.cursor(), edit.text.len());

        edit.move_home();
        assert_eq!(edit.selection, 0..0);
        assert_eq!(edit.cursor(), 0);
    }

    #[test]
    fn insert_replaces_selection() {
        let mut edit = TextEdit::new("hola mundo");

        edit.selection = 5..10; // "mundo"
        edit.replace_selection("Rust");
        assert_eq!(edit.text, "hola Rust");
        assert_eq!(edit.selection, 9..9);
        assert_eq!(edit.cursor(), 9);

        edit.replace_range(4..5, "-");
        assert_eq!(edit.text, "hola-Rust");
        assert_eq!(edit.selection, 5..5);
    }

    #[test]
    fn backspace_deletes_selection() {
        let mut edit = TextEdit::new("hola mundo");

        edit.selection = 4..10;
        edit.backspace();
        assert_eq!(edit.text, "hola");
        assert_eq!(edit.selection, 4..4);
        assert_eq!(edit.cursor(), 4);
    }

    #[test]
    fn backspace_deletes_one_char() {
        let mut edit = TextEdit::new("añb"); // bytes: 'a' 1, 'ñ' 2, 'b' 1

        edit.move_end();
        edit.backspace();
        assert_eq!(edit.text, "añ");
        assert_eq!(edit.cursor(), 3);

        edit.backspace();
        assert_eq!(edit.text, "a");

        edit.move_home();
        edit.backspace();
        assert_eq!(edit.text, "a");
        assert_eq!(edit.selection, 0..0);
    }

    #[test]
    fn delete_deletes_next_char() {
        let mut edit = TextEdit::new("añb");

        edit.move_home();
        edit.delete();
        assert_eq!(edit.text, "ñb");
        assert_eq!(edit.cursor(), 0);

        edit.delete();
        assert_eq!(edit.text, "b");
        edit.delete();
        assert_eq!(edit.text, "");

        // Sin nada que borrar no cambia el estado.
        edit.delete();
        edit.backspace();
        assert_eq!(edit.text, "");
        assert_eq!(edit.selection, 0..0);

        // Delete también elimina la selección.
        let mut edit = TextEdit::new("abc");
        edit.selection = 0..2;
        edit.delete();
        assert_eq!(edit.text, "c");
        assert_eq!(edit.cursor(), 0);
    }

    #[test]
    fn utf16_roundtrip_with_emoji() {
        let edit = TextEdit::new("a🙂b"); // bytes: 0, 1, 5, 6; UTF-16: 0, 1, 3, 4

        assert_eq!(edit.len_utf16(), 4);
        assert_eq!(edit.offset_to_utf16(1), 1);
        assert_eq!(edit.offset_to_utf16(5), 3);
        assert_eq!(edit.offset_from_utf16(3), 5);
        assert_eq!(edit.offset_from_utf16(4), 6);

        for byte in [0usize, 1, 5, 6] {
            assert_eq!(edit.offset_from_utf16(edit.offset_to_utf16(byte)), byte);
        }

        // Offsets en medio de un sustituto: se redondea al extremo del carácter,
        // igual que en la plantilla de GPUI (no hay fronteras intermedias).
        assert_eq!(edit.offset_from_utf16(2), 5);
        assert_eq!(edit.offset_to_utf16(2), 3);
    }

    #[test]
    fn utf16_roundtrip_with_accents() {
        let edit = TextEdit::new("café"); // bytes: 0, 1, 2, 3, 5; UTF-16: 0, 1, 2, 3, 4

        assert_eq!(edit.text.len(), 5);
        assert_eq!(edit.len_utf16(), 4);
        assert_eq!(edit.offset_to_utf16(3), 3);
        assert_eq!(edit.offset_to_utf16(5), 4);
        assert_eq!(edit.offset_from_utf16(4), 5);

        for byte in [0usize, 1, 2, 3, 5] {
            assert_eq!(edit.offset_from_utf16(edit.offset_to_utf16(byte)), byte);
        }
    }

    #[test]
    fn range_utf16_roundtrip() {
        let edit = TextEdit::new("A🙂éZ"); // bytes: 0, 1, 5, 7, 8; UTF-16: 0, 1, 3, 4, 5

        assert_eq!(edit.range_to_utf16(&(1..7)), 1..4);
        assert_eq!(edit.range_from_utf16(&(1..4)), 1..7);

        let whole = 0..edit.text.len();
        assert_eq!(edit.range_from_utf16(&edit.range_to_utf16(&whole)), whole);
    }

    #[test]
    fn replace_and_mark_sets_marked_range() {
        let mut edit = TextEdit::new("A🙂Z");

        // Pre-edit: sustituye bytes 1..5 por "é🙂" y marca lo insertado.
        edit.replace_and_mark(Some(1..5), "é🙂", Some(1..3));
        assert_eq!(edit.text, "Aé🙂Z");
        assert_eq!(edit.marked, Some(1..7));
        assert_eq!(edit.selection, 3..7);

        // Siguiente paso del IME: reemplaza lo marcado por "漢".
        edit.replace_and_mark(None, "漢", Some(1..1));
        assert_eq!(edit.text, "A漢Z");
        assert_eq!(edit.marked, Some(1..4));
        assert_eq!(edit.selection, 4..4);

        // Cancelación: el IME vacía el texto provisional.
        edit.replace_and_mark(None, "", Some(0..0));
        assert_eq!(edit.text, "AZ");
        assert_eq!(edit.marked, None);
        assert_eq!(edit.selection, 1..1);
    }

    #[test]
    fn unmark_clears_marked() {
        let mut edit = TextEdit::new("abc");
        edit.selection = 1..3;
        edit.marked = Some(1..2);

        edit.unmark();
        assert_eq!(edit.marked, None);
        assert_eq!(edit.text, "abc");
        assert_eq!(edit.selection, 1..3);
    }

    #[test]
    fn len_utf16_counts_surrogates() {
        let edit = TextEdit::new("e🙂漢");

        assert_eq!(edit.text.len(), 1 + 4 + 3);
        assert_eq!(edit.len_utf16(), 1 + 2 + 1);

        assert_eq!(TextEdit::new("").len_utf16(), 0);
        assert_eq!(TextEdit::new("café").len_utf16(), 4);
    }
}
