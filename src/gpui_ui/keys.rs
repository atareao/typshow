//! Manejo de teclado compartido por las ventanas de la UI GPUI.
//!
//! Se gestiona con `on_key_down` en la raíz de cada ventana (en vez de
//! `KeyBinding`), porque el editor de notas necesita recibir caracteres que
//! colisionan con los atajos de navegación (p. ej. `h`/`j`/`k`/`l`, espacio).
//! GPUI despacha los keybindings *antes* que `on_key_down`, así que un atajo
//! global secuestraría la escritura; aquí decidimos según el modo.

use gpui::{AppContext, Entity, KeyDownEvent};

use super::deck::Deck;

/// Enum de acciones de edición puras, independientes del contexto GPUI.
#[derive(Debug, PartialEq, Eq)]
pub enum EditAction {
    /// Cancelar edición (Esc)
    Cancel,
    /// Borrar carácter (Backspace)
    Backspace,
    /// Añadir salto de línea (Enter en notas)
    Newline,
    /// Guardar título (Enter en heading)
    SaveHeading,
    /// Insertar carácter (tecla con key_char)
    Insert(String),
    /// Tecla no consumible (sin key_char)
    Ignore,
}

/// Acción de aplicación de teclado puras, sin dependencias de GPUI.
pub fn note_edit_action(key: &str, key_char: Option<&str>) -> EditAction {
    match key {
        "escape" => EditAction::Cancel,
        "backspace" => EditAction::Backspace,
        "enter" => EditAction::Newline,
        _ => {
            let Some(ch) = key_char else {
                return EditAction::Ignore;
            };
            EditAction::Insert(ch.to_string())
        }
    }
}

pub fn heading_edit_action(key: &str, key_char: Option<&str>) -> EditAction {
    match (key, key_char) {
        ("escape", _) => EditAction::Cancel,
        ("backspace", _) => EditAction::Backspace,
        ("enter", _) => EditAction::SaveHeading,
        (_, None) => EditAction::Ignore,
        (_key, Some(ch)) => EditAction::Insert(ch.to_string()),
    }
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
pub fn apply_edit<C: AppContext>(deck: &Entity<Deck>, event: &KeyDownEvent, cx: &mut C) -> bool {
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
                deck.notes.edit_buffer.pop();
                deck.notes.dirty = true;
                cx.notify();
            });
        }
        EditAction::Newline => {
            deck.update(cx, |deck, cx| {
                deck.notes.edit_buffer.push('\n');
                deck.notes.dirty = true;
                cx.notify();
            });
        }
        EditAction::Insert(ch) => {
            deck.update(cx, |deck, cx| {
                deck.notes.edit_buffer.push_str(&ch);
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
pub fn apply_heading_edit<C: AppContext>(
    deck: &Entity<Deck>,
    event: &KeyDownEvent,
    cx: &mut C,
) -> bool {
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
                deck.notes.heading_edit_buffer.pop();
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
                deck.notes.heading_edit_buffer.push_str(&ch);
                cx.notify();
            });
        }
        EditAction::Newline => {}
        EditAction::Ignore => {}
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
}
