//! Manejo de teclado compartido por las ventanas de la UI GPUI.
//!
//! Se gestiona con `on_key_down` en la raíz de cada ventana (en vez de
//! `KeyBinding`), porque el editor de notas necesita recibir caracteres que
//! colisionan con los atajos de navegación (p. ej. `h`/`j`/`k`/`l`, espacio).
//! GPUI despacha los keybindings *antes* que `on_key_down`, así que un atajo
//! global secuestraría la escritura; aquí decidimos según el modo.

use gpui::{App, AppContext, Entity, KeyDownEvent};

use super::deck::Deck;

/// Enum de acciones de edición puras, independientes del contexto GPUI.
#[derive(Debug, PartialEq, Eq)]
pub enum EditAction {
    /// Cancelar edición (Esc)
    Cancel,
    /// Borrar carácter anterior (Backspace)
    Backspace,
    /// Borrar carácter siguiente (Delete)
    Delete,
    /// Añadir salto de línea (Enter en notas)
    Newline,
    /// Guardar título (Enter en heading)
    SaveHeading,
    /// Mover el cursor a la izquierda
    MoveLeft,
    /// Mover el cursor a la derecha
    MoveRight,
    /// Mover el cursor al inicio
    MoveHome,
    /// Mover el cursor al final
    MoveEnd,
    /// Insertar carácter (tecla con key_char)
    Insert(String),
    /// Tecla no consumible (sin key_char)
    Ignore,
}

/// Acción pura del editor de notas para una tecla y su `key_char`.
pub fn note_edit_action(key: &str, key_char: Option<&str>) -> EditAction {
    match key {
        "escape" => EditAction::Cancel,
        "backspace" => EditAction::Backspace,
        "delete" => EditAction::Delete,
        "enter" => EditAction::Newline,
        "left" => EditAction::MoveLeft,
        "right" => EditAction::MoveRight,
        "home" => EditAction::MoveHome,
        "end" => EditAction::MoveEnd,
        _ => match key_char {
            Some(ch) if !ch.is_empty() => EditAction::Insert(ch.to_string()),
            _ => EditAction::Ignore,
        },
    }
}

/// Acción pura del editor del título (heading) para una tecla y su `key_char`.
pub fn heading_edit_action(key: &str, key_char: Option<&str>) -> EditAction {
    match key {
        "escape" => EditAction::Cancel,
        "backspace" => EditAction::Backspace,
        "delete" => EditAction::Delete,
        "enter" => EditAction::SaveHeading,
        "left" => EditAction::MoveLeft,
        "right" => EditAction::MoveRight,
        "home" => EditAction::MoveHome,
        "end" => EditAction::MoveEnd,
        _ => match key_char {
            Some(ch) if !ch.is_empty() => EditAction::Insert(ch.to_string()),
            _ => EditAction::Ignore,
        },
    }
}

// ---------------------------------------------------------------------------
// Operaciones puras sobre (texto, cursor). Offsets en bytes, en frontera de char.
// ---------------------------------------------------------------------------

/// Frontera de carácter anterior a `i`.
pub fn prev_boundary(s: &str, i: usize) -> usize {
    if i == 0 {
        return 0;
    }
    let mut j = (i - 1).min(s.len());
    while j > 0 && !s.is_char_boundary(j) {
        j -= 1;
    }
    j
}

/// Frontera de carácter siguiente a `i`.
pub fn next_boundary(s: &str, i: usize) -> usize {
    let n = s.len();
    if i >= n {
        return n;
    }
    let mut j = i + 1;
    while j < n && !s.is_char_boundary(j) {
        j += 1;
    }
    j
}

/// Normaliza `i` a una frontera de carácter válida dentro de `s`.
fn clamp_boundary(s: &str, i: usize) -> usize {
    let mut j = i.min(s.len());
    while j > 0 && !s.is_char_boundary(j) {
        j -= 1;
    }
    j
}

/// Inserta `text` en `buf` en la posición del cursor y avanza este.
pub fn insert_at(buf: &mut String, cursor: &mut usize, text: &str) {
    let c = clamp_boundary(buf, *cursor);
    buf.insert_str(c, text);
    *cursor = c + text.len();
}

/// Borra el carácter anterior al cursor.
pub fn backspace_at(buf: &mut String, cursor: &mut usize) {
    if *cursor == 0 {
        return;
    }
    let prev = prev_boundary(buf, *cursor);
    buf.replace_range(prev..*cursor, "");
    *cursor = prev;
}

/// Borra el carácter siguiente al cursor.
pub fn delete_at(buf: &mut String, cursor: &mut usize) {
    if *cursor >= buf.len() {
        return;
    }
    let next = next_boundary(buf, *cursor);
    buf.replace_range(*cursor..next, "");
}

/// Texto con un marcador de cursor (`|`) insertado en la posición `cursor`.
pub fn display_with_caret(buf: &str, cursor: usize) -> String {
    let c = clamp_boundary(buf, cursor);
    let mut out = String::with_capacity(buf.len() + 1);
    out.push_str(&buf[..c]);
    out.push('|');
    out.push_str(&buf[c..]);
    out
}

/// Acción de navegación asociada a una tecla (y si Shift está pulsado), si la hay.
pub fn nav_action_for(key: &str, shift: bool) -> Option<fn(&mut Deck)> {
    match key {
        // Flechas / navegación estándar
        "right" | "space" | "pagedown" => Some(Deck::next),
        "left" | "pageup" => Some(Deck::prev),
        "home" => Some(Deck::first),
        "end" => Some(Deck::last),
        // Estilo vim: h/k atrás, l/j adelante, g inicio, G final
        "l" | "j" => Some(Deck::next),
        "h" | "k" => Some(Deck::prev),
        "g" => Some(if shift { Deck::last } else { Deck::first }),
        // Tema
        "t" => Some(Deck::toggle_theme),
        _ => None,
    }
}

/// Acción de navegación asociada a un evento de tecla, si la hay.
fn nav_action(event: &KeyDownEvent) -> Option<fn(&mut Deck)> {
    nav_action_for(&event.keystroke.key, event.keystroke.modifiers.shift)
}

/// Aplica la navegación si la tecla corresponde. Devuelve `true` si se gestionó.
pub fn apply_nav<C: AppContext>(deck: &Entity<Deck>, event: &KeyDownEvent, cx: &mut C) -> bool {
    let Some(action) = nav_action(event) else {
        return false;
    };
    deck.update(cx, |deck, cx| {
        action(deck);
        cx.notify();
    });
    true
}

/// Aplica una pulsación al editor de notas. Devuelve `true` si se gestionó.
pub fn apply_edit(deck: &Entity<Deck>, event: &KeyDownEvent, cx: &mut App) -> bool {
    // Pegar con Ctrl+V: inserta el portapapeles en la posición del cursor.
    if event.keystroke.modifiers.control && event.keystroke.key.eq_ignore_ascii_case("v") {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            deck.update(cx, |deck, cx| {
                insert_at(&mut deck.notes.edit_buffer, &mut deck.notes.cursor, &text);
                deck.notes.dirty = true;
                cx.notify();
            });
        }
        return true;
    }

    let action = note_edit_action(&event.keystroke.key, event.keystroke.key_char.as_deref());
    match action {
        EditAction::Cancel => {
            deck.update(cx, |deck, cx| {
                deck.notes.cancel_edit();
                cx.notify();
            });
        }
        EditAction::Backspace => {
            deck.update(cx, |deck, cx| {
                backspace_at(&mut deck.notes.edit_buffer, &mut deck.notes.cursor);
                deck.notes.dirty = true;
                cx.notify();
            });
        }
        EditAction::Delete => {
            deck.update(cx, |deck, cx| {
                delete_at(&mut deck.notes.edit_buffer, &mut deck.notes.cursor);
                deck.notes.dirty = true;
                cx.notify();
            });
        }
        EditAction::Newline => {
            deck.update(cx, |deck, cx| {
                insert_at(&mut deck.notes.edit_buffer, &mut deck.notes.cursor, "\n");
                deck.notes.dirty = true;
                cx.notify();
            });
        }
        EditAction::MoveLeft => {
            deck.update(cx, |deck, cx| {
                let c = prev_boundary(&deck.notes.edit_buffer, deck.notes.cursor);
                deck.notes.cursor = c;
                cx.notify();
            });
        }
        EditAction::MoveRight => {
            deck.update(cx, |deck, cx| {
                let c = next_boundary(&deck.notes.edit_buffer, deck.notes.cursor);
                deck.notes.cursor = c;
                cx.notify();
            });
        }
        EditAction::MoveHome => {
            deck.update(cx, |deck, cx| {
                deck.notes.cursor = 0;
                cx.notify();
            });
        }
        EditAction::MoveEnd => {
            deck.update(cx, |deck, cx| {
                deck.notes.cursor = deck.notes.edit_buffer.len();
                cx.notify();
            });
        }
        EditAction::Insert(ch) => {
            deck.update(cx, |deck, cx| {
                insert_at(&mut deck.notes.edit_buffer, &mut deck.notes.cursor, &ch);
                deck.notes.dirty = true;
                cx.notify();
            });
        }
        EditAction::SaveHeading | EditAction::Ignore => {}
    }
    true
}

/// Aplica una pulsación al editor del título (heading) de la nota.
/// Devuelve `true` si se gestionó.
pub fn apply_heading_edit(deck: &Entity<Deck>, event: &KeyDownEvent, cx: &mut App) -> bool {
    let action = heading_edit_action(&event.keystroke.key, event.keystroke.key_char.as_deref());
    match action {
        EditAction::Cancel => {
            deck.update(cx, |deck, cx| {
                deck.notes.cancel_edit_heading();
                cx.notify();
            });
        }
        EditAction::Backspace => {
            deck.update(cx, |deck, cx| {
                backspace_at(
                    &mut deck.notes.heading_edit_buffer,
                    &mut deck.notes.heading_cursor,
                );
                cx.notify();
            });
        }
        EditAction::Delete => {
            deck.update(cx, |deck, cx| {
                delete_at(
                    &mut deck.notes.heading_edit_buffer,
                    &mut deck.notes.heading_cursor,
                );
                cx.notify();
            });
        }
        EditAction::MoveLeft => {
            deck.update(cx, |deck, cx| {
                let c = prev_boundary(&deck.notes.heading_edit_buffer, deck.notes.heading_cursor);
                deck.notes.heading_cursor = c;
                cx.notify();
            });
        }
        EditAction::MoveRight => {
            deck.update(cx, |deck, cx| {
                let c = next_boundary(&deck.notes.heading_edit_buffer, deck.notes.heading_cursor);
                deck.notes.heading_cursor = c;
                cx.notify();
            });
        }
        EditAction::MoveHome => {
            deck.update(cx, |deck, cx| {
                deck.notes.heading_cursor = 0;
                cx.notify();
            });
        }
        EditAction::MoveEnd => {
            deck.update(cx, |deck, cx| {
                deck.notes.heading_cursor = deck.notes.heading_edit_buffer.len();
                cx.notify();
            });
        }
        EditAction::SaveHeading => {
            deck.update(cx, |deck, cx| {
                if let Err(e) = deck.notes.save_heading() {
                    deck.error = Some(e.to_string());
                }
                cx.notify();
            });
        }
        EditAction::Insert(ch) => {
            deck.update(cx, |deck, cx| {
                insert_at(
                    &mut deck.notes.heading_edit_buffer,
                    &mut deck.notes.heading_cursor,
                    &ch,
                );
                cx.notify();
            });
        }
        EditAction::Newline | EditAction::Ignore => {}
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigation_keys_map() {
        assert!(nav_action_for("right", false).is_some());
        assert!(nav_action_for("space", false).is_some());
        assert!(nav_action_for("h", false).is_some());
        assert!(nav_action_for("l", false).is_some());
        assert!(nav_action_for("t", false).is_some());
        assert!(nav_action_for("q", false).is_none(), "tecla no mapeada");
        assert!(nav_action_for("F5", false).is_none());
    }

    #[test]
    fn shift_g_goes_last() {
        let mut deck = Deck::empty();
        deck.total = 5;
        nav_action_for("g", false).unwrap()(&mut deck);
        assert_eq!(deck.page, 0, "g va a la primera");
        nav_action_for("g", true).unwrap()(&mut deck);
        assert_eq!(deck.page, 4, "Shift+G va a la ultima");
    }

    #[test]
    fn vim_h_l_move_one_page() {
        let mut deck = Deck::empty();
        deck.total = 3;
        nav_action_for("l", false).unwrap()(&mut deck);
        assert_eq!(deck.page, 1);
        nav_action_for("h", false).unwrap()(&mut deck);
        assert_eq!(deck.page, 0);
    }

    #[test]
    fn note_edit_action_cancels_on_escape() {
        assert_eq!(note_edit_action("escape", None), EditAction::Cancel);
    }

    #[test]
    fn note_edit_action_backspace() {
        assert_eq!(note_edit_action("backspace", None), EditAction::Backspace);
    }

    #[test]
    fn note_edit_action_newline() {
        assert_eq!(note_edit_action("enter", None), EditAction::Newline);
    }

    #[test]
    fn note_edit_action_insert_accented_chars() {
        assert_eq!(
            note_edit_action("aacute", Some("á")),
            EditAction::Insert("á".to_string())
        );
        assert_eq!(
            note_edit_action("ntilde", Some("ñ")),
            EditAction::Insert("ñ".to_string())
        );
        assert_eq!(
            note_edit_action("a", Some("a")),
            EditAction::Insert("a".to_string())
        );
        assert_eq!(
            note_edit_action("oacute", Some("ó")),
            EditAction::Insert("ó".to_string())
        );
    }

    #[test]
    fn note_edit_action_ignore_without_char() {
        assert_eq!(note_edit_action("shift", None), EditAction::Ignore);
    }

    #[test]
    fn heading_edit_action_cancels_on_escape() {
        assert_eq!(heading_edit_action("escape", None), EditAction::Cancel);
    }

    #[test]
    fn heading_edit_action_backspace() {
        assert_eq!(
            heading_edit_action("backspace", None),
            EditAction::Backspace
        );
    }

    #[test]
    fn heading_edit_action_save_on_enter() {
        assert_eq!(heading_edit_action("enter", None), EditAction::SaveHeading);
    }

    #[test]
    fn heading_edit_action_insert_accented_chars() {
        assert_eq!(
            heading_edit_action("oacute", Some("ó")),
            EditAction::Insert("ó".to_string())
        );
    }

    #[test]
    fn heading_edit_action_ignore_without_char() {
        assert_eq!(heading_edit_action("shift", None), EditAction::Ignore);
    }
    #[test]
    fn edit_actions_map_arrows_and_delete() {
        assert_eq!(note_edit_action("left", None), EditAction::MoveLeft);
        assert_eq!(note_edit_action("right", None), EditAction::MoveRight);
        assert_eq!(note_edit_action("home", None), EditAction::MoveHome);
        assert_eq!(note_edit_action("end", None), EditAction::MoveEnd);
        assert_eq!(note_edit_action("delete", None), EditAction::Delete);
        assert_eq!(heading_edit_action("delete", None), EditAction::Delete);
    }

    #[test]
    fn boundaries_respect_char_boundaries() {
        // 'a'(1 byte) 'á'(2) '€'(3)
        let s = "aá€";
        assert_eq!(s.len(), 6);
        assert_eq!(next_boundary(s, 0), 1);
        assert_eq!(next_boundary(s, 1), 3);
        assert_eq!(next_boundary(s, 3), 6);
        assert_eq!(next_boundary(s, 6), 6);
        assert_eq!(prev_boundary(s, 6), 3);
        assert_eq!(prev_boundary(s, 3), 1);
        assert_eq!(prev_boundary(s, 1), 0);
        assert_eq!(prev_boundary(s, 0), 0);
    }

    #[test]
    fn insert_at_advances_cursor() {
        let mut buf = String::from("ac");
        let mut cur = 1;
        insert_at(&mut buf, &mut cur, "b");
        assert_eq!(buf, "abc");
        assert_eq!(cur, 2);
        // multibyte
        insert_at(&mut buf, &mut cur, "á");
        assert_eq!(buf, "abác");
        assert_eq!(cur, 4);
    }

    #[test]
    fn backspace_removes_previous_char() {
        let mut buf = String::from("aá");
        let mut cur = buf.len(); // 3
        backspace_at(&mut buf, &mut cur);
        assert_eq!(buf, "a");
        assert_eq!(cur, 1);
        backspace_at(&mut buf, &mut cur);
        assert_eq!(buf, "");
        assert_eq!(cur, 0);
        // no-op al inicio
        backspace_at(&mut buf, &mut cur);
        assert_eq!(buf, "");
        assert_eq!(cur, 0);
    }

    #[test]
    fn delete_removes_next_char() {
        let mut buf = String::from("aáb");
        let mut cur = 1;
        delete_at(&mut buf, &mut cur);
        assert_eq!(buf, "ab");
        assert_eq!(cur, 1);
        // no-op al final
        let mut buf2 = String::from("ab");
        let mut cur2 = 2;
        delete_at(&mut buf2, &mut cur2);
        assert_eq!(buf2, "ab");
    }

    #[test]
    fn display_with_caret_marks_position() {
        assert_eq!(display_with_caret("abc", 1), "a|bc");
        assert_eq!(display_with_caret("", 0), "|");
        assert_eq!(display_with_caret("áé", 0), "|áé");
        assert_eq!(display_with_caret("áé", 2), "á|é");
        // offset dentro de un multibyte se normaliza hacia atrás
        assert_eq!(display_with_caret("áé", 1), "|áé");
    }
}
