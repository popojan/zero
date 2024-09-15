use bevy::prelude::*;
use bevy::window::WindowResized;
use bevy::window::PrimaryWindow;
use crate::{MIN_CHAR_HEIGHT, MIN_CHAR_WIDTH};

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

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum TerminalState {
    #[default]
    New,
    Resized,
    Ready,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(Event)]
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

    pub fn clear() -> Self {
        Self { row: usize::MAX, col: usize::MAX, s: String::from(" "), attr: (Color::WHITE, Color::BLACK) }
    }
}

#[derive(Event)]
pub struct TerminalReady;

#[derive(Event)]
pub struct TerminalNew;

pub struct TerminalOperation(pub Vec<TerminalEvent>);

#[derive(Component)]
struct Foreground;

#[derive(Component)]
struct Background;

fn window_resized_system(
    mut event_resized: EventReader<WindowResized>,
    mut state: ResMut<NextState<TerminalState>>,
) {
    for _event in event_resized.read() {
        state.set(TerminalState::New);
        break;
    }
}

fn scale_terminal_system(
    mut commands: Commands,
    query: Query<Entity, Or<(With<Foreground>, With<Background>)>>,
    asset_server: Res<AssetServer>,
    mut terminal: Query<(Entity, &Terminal)>,
    mut next_state: ResMut<NextState<TerminalState>>,
    state: Res<State<TerminalState>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut is_new: EventWriter<TerminalNew>,
    mut is_ready: EventWriter<TerminalReady>,
    mut resize_events: EventReader<WindowResized>,
    text: Query<(&Text, &Node), With<Foreground>>
) {
    let mut new_state = state.clone();
    let window = windows.get_single().unwrap();
    let mut width = window.width();
    let mut height = window.height();

    for e in resize_events.read() {
        height = e.height;
        width = e.width;
    }
    let font_scale: (f32, f32) = match state.get() {
        TerminalState::New => {
            new_state = TerminalState::Resized;
            (1.0, 1.0)
        }
        TerminalState::Resized => {
            is_new.send(TerminalNew);
            let terminal = terminal.single_mut().1;
            let (_text, node) = text.single();
            let size = node.size();
            let width = size.x;
            let height = size.y;
            if width > 0.0 && height > 0.0 {
                new_state = TerminalState::Ready;
                (
                    width / terminal.cols as f32 / terminal.font_size,
                    height / terminal.rows as f32 / terminal.font_size,
                )
            } else {
                (1.0, 1.0)
            }
        }
        TerminalState::Ready => terminal.single_mut().1.font_scale
    };
    if state.get() != & new_state {
        query.iter().for_each( | id| commands.entity(id).despawn());
        let resized_terminal = Terminal::new(
            MIN_CHAR_WIDTH, MIN_CHAR_HEIGHT,
            width, height, FONT_PATH.to_string(), font_scale
        );
        if let Some(old_terminal) = terminal.iter_mut().next() {
            commands.entity(old_terminal.0).despawn();
        }

        {
            let back = resized_terminal.create_layer(width, height, &asset_server);
            let fore = resized_terminal.create_layer(width, height, &asset_server);
            commands.spawn(back).insert(Background);
            commands.spawn(fore).insert(Foreground);
            commands.spawn(resized_terminal);
            next_state.set(new_state);
        }
    } else if state.get() == & TerminalState::Ready{
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
            .add_systems(Startup, setup)
            .init_state::<TerminalState>()
            .add_event::<TerminalEvent>()
            .add_event::<TerminalReady>()
            .add_event::<TerminalNew>()
            .add_systems(Update, window_resized_system)
            .add_systems(Update, scale_terminal_system)
            .add_systems(Update, (terminal_update_system).run_if(in_state(TerminalState::Ready)));
    }
}

fn terminal_update_system(mut q0: Query<&mut Terminal>, mut q1: Query<&mut Text, (With<Foreground>, Without<Background>)>,
        mut q2: Query<&mut Text, (With<Background>, Without<Foreground>)>, mut events: EventReader<TerminalEvent>,
    state: Res<State<TerminalState>>,
    mut next_state: ResMut<NextState<TerminalState>>
) {
    for e in events.read() {
        if e.row == usize::MAX && e.col == usize::MAX {
            if state.get() == &TerminalState::Ready {
                next_state.set(TerminalState::New);
                break;
            }
        } else {
            let mut terminal = q0.single_mut();
            let mut fore = q1.single_mut();
            let mut back = q2.single_mut();
            terminal.attron(e.attr);
            terminal.mvprintw(fore.as_mut(), back.as_mut(), e.row, e.col, &e.s);
        }
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
    pub fn new(min_w: u16, min_h: u16, width: f32, height: f32, font_path: String, font_scale: (f32, f32)) -> Self {

        let _y = height / min_h as f32;
        let _x = width / min_w as f32;
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
                        color: Color::Srgba(Srgba::RED),
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
                position_type: PositionType::Absolute,
                align_self: AlignSelf::FlexStart,
                left: Val::Px(off_x),
                bottom: Val::Px(off_y),
                ..Default::default()
            },
            text: Text::from_sections(
                layer.into_iter().flatten().collect::<Vec<_>>()
            ),
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