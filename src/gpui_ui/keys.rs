//! Manejo de teclado compartido por las ventanas de la UI GPUI.
//!
//! Se gestiona con `on_key_down` en la raíz de cada ventana (en vez de
//! `KeyBinding`), porque el editor de notas necesita recibir caracteres que
//! colisionan con los atajos de navegación (p. ej. `h`/`j`/`k`/`l`, espacio).
//! GPUI despacha los keybindings *antes* que `on_key_down`, así que un atajo
//! global secuestraría la escritura; aquí decidimos según el modo.
//!
//! Contenido: mapeo de teclas a acciones de **navegación** sobre `Deck`, el
//! predicado de **salida de la app** (`Ctrl+Q`) y los helpers puros de
//! **frontera de carácter** (offsets en bytes) que consume el editor de
//! `edit.rs`.

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

/// `true` si la tecla corresponde al atajo de salida de la app (`Ctrl+Q`).
///
/// Es un predicado puro para poder testearlo sin GPUI. En GPUI `keystroke.key`
/// llega en minúsculas para letras (`"q"`).
///
/// Solo mira `control`: **ignora Shift y el resto de modificadores**, de modo
/// que `Ctrl+Shift+Q` (o `Ctrl+Alt+Q`) también sale. Es deliberado: es un atajo
/// de escape que no colisiona con nada (en modo edición `editor_command` trata
/// todo `Ctrl+<letra>` que no sea a/c/x/v como `Ignore`), y exigir `!shift`
/// obligaría a decidir además qué hacer con Alt/Super para no abrir la puerta a
/// combinaciones que el usuario deba memorizar. `!control` sigue siendo
/// obligatorio: `q` a secas es texto para la nota.
pub fn quit_shortcut(key: &str, control: bool) -> bool {
    control && key == "q"
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
    use gpui::Keystroke;

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
    fn quit_shortcut_matches_ctrl_q() {
        assert!(quit_shortcut("q", true), "Ctrl+Q debe salir");
        assert!(!quit_shortcut("q", false), "q sin Ctrl no sale");
        assert!(!quit_shortcut("w", true), "Ctrl+W no es el atajo");
        assert!(!quit_shortcut("t", true), "Ctrl+T no es el atajo");
    }

    #[test]
    fn quit_shortcut_ignores_shift_on_purpose() {
        // Keystroke real de `Ctrl+Shift+Q`: el predicado no recibe `shift`, así
        // que esa combinación sale igual que `Ctrl+Q`. La decisión (ignorar
        // Shift y demás modificadores) está documentada en `quit_shortcut`; el
        // aserto fija que es intencionada y no un olvido.
        let keystroke = Keystroke::parse("ctrl-shift-q").expect("keystroke válido");
        assert!(keystroke.modifiers.control, "el keystroke trae Control");
        assert!(keystroke.modifiers.shift, "el keystroke trae Shift");
        assert!(
            quit_shortcut(&keystroke.key, keystroke.modifiers.control),
            "Ctrl+Shift+Q también sale: Shift se ignora a propósito"
        );
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
