use bevy::prelude::KeyCode;

pub struct KeyCodeExt(pub KeyCode);

// thanks to https://github.com/Jerald/bevy

impl KeyCodeExt {
    pub fn to_qwerty_char(self, shift_down: bool) -> Option<char> {
        if shift_down {
            self.to_qwerty_char_with_shift()
        } else {
            self.to_qwerty_char_without_shift()
        }
    }

    pub fn to_qwerty_char_without_shift(self) -> Option<char> {
        let out = match self.0 {
            KeyCode::Digit1 => '1',
            KeyCode::Digit2 => '2',
            KeyCode::Digit3 => '3',
            KeyCode::Digit4 => '4',
            KeyCode::Digit5 => '5',
            KeyCode::Digit6 => '6',
            KeyCode::Digit7 => '7',
            KeyCode::Digit8 => '8',
            KeyCode::Digit9 => '9',
            KeyCode::Digit0 => '0',

            KeyCode::KeyA => 'a',
            KeyCode::KeyB => 'b',
            KeyCode::KeyC => 'c',
            KeyCode::KeyD => 'd',
            KeyCode::KeyE => 'e',
            KeyCode::KeyF => 'f',
            KeyCode::KeyG => 'g',
            KeyCode::KeyH => 'h',
            KeyCode::KeyI => 'i',
            KeyCode::KeyJ => 'j',
            KeyCode::KeyK => 'k',
            KeyCode::KeyL => 'l',
            KeyCode::KeyM => 'm',
            KeyCode::KeyN => 'n',
            KeyCode::KeyO => 'o',
            KeyCode::KeyP => 'p',
            KeyCode::KeyQ => 'q',
            KeyCode::KeyR => 'r',
            KeyCode::KeyS => 's',
            KeyCode::KeyT => 't',
            KeyCode::KeyU => 'u',
            KeyCode::KeyV => 'v',
            KeyCode::KeyW => 'w',
            KeyCode::KeyX => 'x',
            KeyCode::KeyY => 'y',
            KeyCode::KeyZ => 'z',

            KeyCode::Numpad0 => '0',
            KeyCode::Numpad1 => '1',
            KeyCode::Numpad2 => '2',
            KeyCode::Numpad3 => '3',
            KeyCode::Numpad4 => '4',
            KeyCode::Numpad5 => '5',
            KeyCode::Numpad6 => '6',
            KeyCode::Numpad7 => '7',
            KeyCode::Numpad8 => '8',
            KeyCode::Numpad9 => '9',

            KeyCode::NumpadAdd => '+',
            KeyCode::Quote => '\'',
            KeyCode::Backslash => '\\',
            KeyCode::Comma => ',',
            KeyCode::NumpadDecimal => '.',
            KeyCode::NumpadDivide => '/',
            KeyCode::Equal => '=',
            KeyCode::Backquote => '`',
            KeyCode::BracketLeft => '[',
            KeyCode::Minus => '-',
            KeyCode::Period => '.',
            KeyCode::BracketRight => ']',
            KeyCode::Semicolon => ';',
            KeyCode::Slash => '/',
            KeyCode::Tab => '\t',
            KeyCode::Space => ' ',
            _ => return None,
        };

        Some(out)
    }

    pub fn to_qwerty_char_with_shift(self) -> Option<char> {
        let out = match self.0 {
            KeyCode::Digit1 => '!',
            KeyCode::Digit2 => '@',
            KeyCode::Digit3 => '#',
            KeyCode::Digit4 => '$',
            KeyCode::Digit5 => '%',
            KeyCode::Digit6 => '^',
            KeyCode::Digit7 => '&',
            KeyCode::Digit8 => '*',
            KeyCode::Digit9 => '(',
            KeyCode::Digit0 => ')',

            KeyCode::KeyA => 'A',
            KeyCode::KeyB => 'B',
            KeyCode::KeyC => 'C',
            KeyCode::KeyD => 'D',
            KeyCode::KeyE => 'E',
            KeyCode::KeyF => 'F',
            KeyCode::KeyG => 'G',
            KeyCode::KeyH => 'H',
            KeyCode::KeyI => 'I',
            KeyCode::KeyJ => 'J',
            KeyCode::KeyK => 'K',
            KeyCode::KeyL => 'L',
            KeyCode::KeyM => 'M',
            KeyCode::KeyN => 'N',
            KeyCode::KeyO => 'O',
            KeyCode::KeyP => 'P',
            KeyCode::KeyQ => 'Q',
            KeyCode::KeyR => 'R',
            KeyCode::KeyS => 'S',
            KeyCode::KeyT => 'T',
            KeyCode::KeyU => 'U',
            KeyCode::KeyV => 'V',
            KeyCode::KeyW => 'W',
            KeyCode::KeyX => 'X',
            KeyCode::KeyY => 'Y',
            KeyCode::KeyZ => 'Z',

            KeyCode::NumpadAdd => '+',
            KeyCode::Quote => '"',
            KeyCode::Backslash => '|',
            KeyCode::Comma => '<',
            KeyCode::NumpadDivide => '/',
            KeyCode::Equal => '+',
            KeyCode::Backquote => '~',
            KeyCode::BracketLeft => '{',
            KeyCode::Minus => '_',
            KeyCode::Period => '>',
            KeyCode::BracketRight => '}',
            KeyCode::Semicolon => ':',
            KeyCode::Slash => '?',
            KeyCode::Tab => '\t',
            KeyCode::Space => ' ',
            _ => return None,
        };

        Some(out)
    }
}