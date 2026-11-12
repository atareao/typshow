//! Manejo de teclado compartido por las ventanas de la UI GPUI.
//!
//! Se gestiona con `on_key_down` en la raíz de cada ventana (en vez de
//! `KeyBinding`), porque el editor de notas necesita recibir caracteres que
//! colisionan con los atajos de navegación (p. ej. `h`/`j`/`k`/`l`, espacio).
//! GPUI despacha los keybindings *antes* que `on_key_down`, así que un atajo
//! global secuestraría la escritura; aquí decidimos según el modo.
//!
//! Contenido: mapeo de teclas a acciones de **navegación** sobre `Deck` y los
//! helpers puros de **frontera de carácter** (offsets en bytes) que consume el
//! editor de `edit.rs`.

use gpui::{AppContext, Entity, KeyDownEvent};

use super::deck::Deck;

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
pub fn clamp_boundary(s: &str, i: usize) -> usize {
    let mut j = i.min(s.len());
    while j > 0 && !s.is_char_boundary(j) {
        j -= 1;
    }
    j
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
}
