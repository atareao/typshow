//! Manejo de teclado compartido por las ventanas de la UI GPUI.
//!
//! Se gestiona con `on_key_down` en la raíz de cada ventana (en vez de
//! `KeyBinding`), porque el editor de notas necesita recibir caracteres que
//! colisionan con los atajos de navegación (p. ej. `h`/`j`/`k`/`l`, espacio).
//! GPUI despacha los keybindings *antes* que `on_key_down`, así que un atajo
//! global secuestraría la escritura; aquí decidimos según el modo.

use gpui::{AppContext, Entity, KeyDownEvent};

use super::deck::Deck;

/// Acción de navegación asociada a una tecla, si la hay.
fn nav_action(event: &KeyDownEvent) -> Option<fn(&mut Deck)> {
    let shift = event.keystroke.modifiers.shift;
    match event.keystroke.key.as_str() {
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
    match event.keystroke.key.as_str() {
        "escape" => {
            deck.update(cx, |deck, cx| {
                deck.notes.cancel_edit();
                cx.notify();
            });
            true
        }
        "backspace" => {
            deck.update(cx, |deck, cx| {
                deck.notes.edit_buffer.pop();
                deck.notes.dirty = true;
                cx.notify();
            });
            true
        }
        "enter" => {
            deck.update(cx, |deck, cx| {
                deck.notes.edit_buffer.push('\n');
                deck.notes.dirty = true;
                cx.notify();
            });
            true
        }
        _ => {
            let Some(ch) = event.keystroke.key_char.clone() else {
                return false;
            };
            deck.update(cx, |deck, cx| {
                deck.notes.edit_buffer.push_str(&ch);
                deck.notes.dirty = true;
                cx.notify();
            });
            true
        }
    }
}
