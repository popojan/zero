use bevy::prelude::*;

pub struct Terminal {
    pub rows: usize,
    pub cols: usize,
    pub font_size: f32,
    pub font_path: String,
    pub font_scale: (f32, f32),
}

pub struct Ncurses<'a> {
    terminal: &'a Terminal,
    foreground: &'a mut Text,
    background: &'a mut Text,
}

impl<'a> Ncurses<'a> {
    pub fn mvaddch(&'a mut self, row: usize, col: usize, c: char, fg: &Color, bg: &Color) {
        let idx = (self.terminal.cols + 1) * row + col;
        let section = &mut self.foreground.sections[idx];
        section.value = String::from(c);
        section.style.color = *fg;

        let section = &mut self.background.sections[idx];
        section.value = String::from('█');
        section.style.color = *bg;

    }
}
impl Terminal {

    pub fn new(width: f32, height: f32, font_path: String, font_scale: (f32, f32)) -> Self {
        const Y: usize = 25;
        const X: usize = 80;

        let _y = height / Y as f32;
        let _x = width / X as f32;
        let font_size = if _x/(X as f32)>= _y/(Y as f32) { _x / font_scale.0 } else { _y / font_scale.1 };
        let rows = std::cmp::max(1, (height / font_size / font_scale.1).floor() as usize);
        let cols = std::cmp::max(1, (width / font_size / font_scale.0).floor() as usize);
        Self {rows, cols, font_size, font_scale, font_path}
    }

    pub fn create_layer(&self, width: f32, height: f32, asset_server: &AssetServer) -> TextBundle {
        let off_x = (width - self.cols as f32 * self.font_size * self.font_scale.0) / 2.;
        let off_y = (height - self.rows as f32 * self.font_size * self.font_scale.1) / 2.;

        let mut layer = vec![
            vec![
                TextSection {
                    value: " ".to_string(),
                    style: TextStyle {
                        font: asset_server.load(&self.font_path),
                        font_size: self.font_size,
                        color: Color::BLACK,
                    },
                }; self.cols + 1]; self.rows];

        layer.iter_mut().for_each(|row| {
            row.iter_mut().for_each(|text| {
                text.value = " ".to_string();
            });
            if let Some(end) = row.last_mut() {
                end.value = "\n".to_string();
            }
        });
        TextBundle {
            style: Style {
                position: Rect {
                    left: Val::Px(off_x),
                    bottom: Val::Px(off_y),
                    ..Default::default()
                },
                position_type: PositionType::Absolute,
                align_self: AlignSelf::FlexStart,
                ..Default::default()
            },
            text: Text {
                sections: layer.into_iter().flatten().collect(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn with<'a>(&'a self, foreground: &'a mut Text, background: &'a mut Text) -> Ncurses<'a> {
        Ncurses { terminal: self, foreground, background }
    }
}