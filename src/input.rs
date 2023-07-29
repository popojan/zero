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
            KeyCode::Key1 => '1',
            KeyCode::Key2 => '2',
            KeyCode::Key3 => '3',
            KeyCode::Key4 => '4',
            KeyCode::Key5 => '5',
            KeyCode::Key6 => '6',
            KeyCode::Key7 => '7',
            KeyCode::Key8 => '8',
            KeyCode::Key9 => '9',
            KeyCode::Key0 => '0',

            KeyCode::A => 'a',
            KeyCode::B => 'b',
            KeyCode::C => 'c',
            KeyCode::D => 'd',
            KeyCode::E => 'e',
            KeyCode::F => 'f',
            KeyCode::G => 'g',
            KeyCode::H => 'h',
            KeyCode::I => 'i',
            KeyCode::J => 'j',
            KeyCode::K => 'k',
            KeyCode::L => 'l',
            KeyCode::M => 'm',
            KeyCode::N => 'n',
            KeyCode::O => 'o',
            KeyCode::P => 'p',
            KeyCode::Q => 'q',
            KeyCode::R => 'r',
            KeyCode::S => 's',
            KeyCode::T => 't',
            KeyCode::U => 'u',
            KeyCode::V => 'v',
            KeyCode::W => 'w',
            KeyCode::X => 'x',
            KeyCode::Y => 'y',
            KeyCode::Z => 'z',

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
            KeyCode::Apostrophe => '\'',
            KeyCode::Backslash => '\\',
            KeyCode::Comma => ',',
            KeyCode::NumpadDecimal => '.',
            KeyCode::NumpadDivide => '/',
            KeyCode::Equals => '=',
            KeyCode::Grave => '`',
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
            KeyCode::Key1 => '!',
            KeyCode::Key2 => '@',
            KeyCode::Key3 => '#',
            KeyCode::Key4 => '$',
            KeyCode::Key5 => '%',
            KeyCode::Key6 => '^',
            KeyCode::Key7 => '&',
            KeyCode::Key8 => '*',
            KeyCode::Key9 => '(',
            KeyCode::Key0 => ')',

            KeyCode::A => 'A',
            KeyCode::B => 'B',
            KeyCode::C => 'C',
            KeyCode::D => 'D',
            KeyCode::E => 'E',
            KeyCode::F => 'F',
            KeyCode::G => 'G',
            KeyCode::H => 'H',
            KeyCode::I => 'I',
            KeyCode::J => 'J',
            KeyCode::K => 'K',
            KeyCode::L => 'L',
            KeyCode::M => 'M',
            KeyCode::N => 'N',
            KeyCode::O => 'O',
            KeyCode::P => 'P',
            KeyCode::Q => 'Q',
            KeyCode::R => 'R',
            KeyCode::S => 'S',
            KeyCode::T => 'T',
            KeyCode::U => 'U',
            KeyCode::V => 'V',
            KeyCode::W => 'W',
            KeyCode::X => 'X',
            KeyCode::Y => 'Y',
            KeyCode::Z => 'Z',

            KeyCode::NumpadAdd => '+',
            KeyCode::Apostrophe => '"',
            KeyCode::Backslash => '|',
            KeyCode::Comma => '<',
            KeyCode::NumpadDivide => '/',
            KeyCode::Equals => '+',
            KeyCode::Grave => '~',
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