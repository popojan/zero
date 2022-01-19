use bevy::prelude::*;
use bevy::window::WindowResized;

#[derive(Component)]
pub struct Terminal {
    pub rows: usize,
    pub cols: usize,
    pub font_size: f32,
    pub font_path: String,
    pub font_scale: (f32, f32),
    pub color_pair: (Color, Color),
}

//const FONT_PATH: &str = "fonts/DejaVuSansMono-Bold.ttf";
//const FONT_PATH: &str = "fonts/FreeMonoBold.otf";
const FONT_PATH: &str = "fonts/iosevka-term-regular.ttf";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TerminalState {
    New,
    Resized,
    Ready,
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(UiCameraBundle::default());
}

pub struct TerminalEvent {
    pub row: usize,
    pub col: usize,
    pub s: String,
    pub attr: (Color, Color)
}

impl TerminalEvent {
    pub fn attron(attr: (Color, Color)) -> Self {
        Self { row: 0, col: 0, s: " ".to_string(), attr }
    }

    pub fn mvaddch(row: usize, col: usize, c: char) -> Self {
        Self { row, col, s: c.to_string(), attr: (Color::WHITE, Color::BLACK) }
    }
}

pub struct TerminalReady;
pub struct TerminalNew;

pub struct TerminalOperation(pub Vec<TerminalEvent>);

#[derive(Component)]
struct Foreground;

#[derive(Component)]
struct Background;

fn window_resized_system(
    mut event_resized: EventReader<WindowResized>,
    mut state: ResMut<State<TerminalState>>,
) {
    for _event in event_resized.iter() {
        if state.current() == &TerminalState::Ready {
            state.set(TerminalState::New).unwrap();
        }
        break;
    }
}

fn scale_terminal_system(
    mut commands: Commands,
    query: Query<Entity, Or<(With<Foreground>, With<Background>)>>,
    asset_server: Res<AssetServer>,
    mut terminal: Query<(Entity, &Terminal)>,
    text: Query<&CalculatedSize, With<Foreground>>,
    mut state: ResMut<State<TerminalState>>,
    windows: Res<Windows>,
    mut is_new: EventWriter<TerminalNew>,
    mut is_ready: EventWriter<TerminalReady>,
) {
    let mut new_state = state.current().clone();
    let font_scale: (f32, f32) = match state.current() {
        TerminalState::New => {
            new_state = TerminalState::Resized;
            (1.0, 1.0)
        }
        TerminalState::Resized => {
            is_new.send(TerminalNew);
            let size = text.single();
            let terminal = terminal.single_mut().1;
            if size.size.width > 0.0 && size.size.height > 0.0 {
                new_state = TerminalState::Ready;
                (
                    size.size.width / terminal.cols as f32 / terminal.font_size,
                    size.size.height / terminal.rows as f32 / terminal.font_size,
                )
            } else {
                (1.0, 1.0)
            }
        }
        TerminalState::Ready => terminal.single_mut().1.font_scale
    };

    if state.current() != &new_state {
        query.iter().for_each(|id| commands.entity(id).despawn());
        let window = windows.get_primary().unwrap();
        let width = window.width();
        let height = window.height();

        let resized_terminal = Terminal::new(
            width, height, FONT_PATH.to_string(), font_scale
        );
        if let Some(old_terminal) = terminal.iter_mut().next() {
            commands.entity(old_terminal.0).despawn();
        }
        {
            let back = resized_terminal.create_layer(width, height, &asset_server);
            let fore = resized_terminal.create_layer(width, height, &asset_server);
            commands.spawn_bundle(back).insert(Background);
            commands.spawn_bundle(fore).insert(Foreground);
            commands.spawn().insert(resized_terminal);
            state.set(new_state).unwrap();
        }
    } else if state.current() == &TerminalState::Ready{
        is_ready.send(TerminalReady);
    }
}

pub struct TerminalPlugin;

impl TerminalPlugin {
    pub(crate) fn new() -> Self {
        TerminalPlugin{ }
    }
}
impl Plugin for TerminalPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ClearColor(Color::BLACK))
            .add_startup_system(setup)
            .add_state(TerminalState::New)
            .add_event::<TerminalEvent>()
            .add_event::<TerminalReady>()
            .add_event::<TerminalNew>()
            .add_system(window_resized_system)
            .add_system(scale_terminal_system)
            .add_system_set(SystemSet::on_update(TerminalState::Ready)
                .with_system(terminal_update_system));
    }
}

fn terminal_update_system(mut q0: Query<&mut Terminal>, mut q1: Query<&mut Text, (With<Foreground>, Without<Background>)>,
        mut q2: Query<&mut Text, (With<Background>, Without<Foreground>)>, mut events: EventReader<TerminalEvent>) {
    for e in events.iter() {
        let mut terminal = q0.single_mut();
        let mut fore = q1.single_mut();
        let mut back = q2.single_mut();
        terminal.attron(e.attr);
        terminal.mvprintw(fore.as_mut(), back.as_mut(), e.row, e.col, &e.s);
    }
}
impl Terminal {

    pub fn getmaxxy(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }

    pub fn command(&mut self, fun: fn(terminal: &mut Self, fore: &mut Text, back: &mut Text) -> (),
                   fore: &mut Text, back: &mut Text) {
        fun(self, fore, back);
    }
    pub fn new(width: f32, height: f32, font_path: String, font_scale: (f32, f32)) -> Self {
        const Y: usize = 35;
        const X: usize = 80;

        let _y = height / Y as f32;
        let _x = width / X as f32;
        let font_size = if _x >= _y { _x / font_scale.0 } else { _y / font_scale.1 };
        let rows = std::cmp::max(1, (height / font_size / font_scale.1).floor() as usize);
        let cols = std::cmp::max(1, (width / font_size / font_scale.0).floor() as usize);

        Self {rows, cols, font_size, font_scale, font_path, color_pair: (Color::WHITE, Color::BLACK)}
    }

    fn create_layer(&self, width: f32, height: f32, asset_server: &AssetServer) -> TextBundle {
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

    pub fn attron(&mut self, color_pair: (Color, Color)) {
        self.color_pair = color_pair;
    }

    pub fn mvaddch(&self, fore: &mut Text, back: &mut Text, row: usize, col: usize, c: char) {
        let idx = (self.cols + 1) * row + col;
        fn addch(section: &mut TextSection, c: char, col: Color) {
            section.value = String::from(c);
            section.style.color = col;
        }
        if (idx < back.sections.len()) & (idx < fore.sections.len()) {
            addch(&mut back.sections[idx], '█', self.color_pair.1);
            addch(&mut fore.sections[idx], c, self.color_pair.0);
        }
    }

    pub fn mvprintw(&mut self, fore: &mut Text, back: &mut Text, row: usize, col: usize, s: &str) {
        let mut cur: (usize, usize) = (row, col);
        for c in s.chars() {
            self.mvaddch(fore, back, cur.0, cur.1, c);
            cur.1 += 1;
            if cur.1 >= self.cols {
                cur.0 += 1;
                cur.1 = 0;
            }
        }
    }

}