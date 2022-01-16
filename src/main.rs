use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    window::WindowResized,
};

extern crate rand;

pub mod terminal;
use terminal::Terminal;

//const FONT_PATH: &str = "fonts/DejaVuSansMono-Bold.ttf";
const FONT_PATH: &str = "fonts/FreeMonoBold.otf";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum TerminalState {
    New,
    Resized,
    Ready,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(ClearColor(Color::BLACK))
        .add_startup_system(setup)
        .add_state(TerminalState::New)
        .add_system(window_resized_system)
        .add_system(scale_terminal_system)
        .add_system(text_update_system)
        .add_system(text_color_system)
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}

#[derive(Component)]
struct FpsText;

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
    terminal: Option<Res<Terminal>>,
    text: Query<&CalculatedSize, With<Foreground>>,
    mut state: ResMut<State<TerminalState>>,
    windows: Res<Windows>,
) {
    let mut new_state = state.current().clone();
    let font_scale: (f32, f32) = match state.current() {
        TerminalState::New => {
            new_state = TerminalState::Resized;
            (1.0, 1.0)
        }
        TerminalState::Resized => {
            let size = text.single();
            let terminal = terminal.unwrap();
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
        TerminalState::Ready => terminal.unwrap().font_scale
    };

     if state.current() != &new_state {
        query.iter().for_each(|id| commands.entity(id).despawn());
        let window = windows.get_primary().unwrap();
        let width = window.width();
        let height = window.height();

        let resized_terminal = Terminal::new(
            width, height, FONT_PATH.to_string(), font_scale
        );
        let fg = resized_terminal.create_layer(width, height, &asset_server);
        let bg = resized_terminal.create_layer(width, height, &asset_server);
        commands.insert_resource(resized_terminal);
        commands.spawn_bundle(bg).insert(Background);
        commands.spawn_bundle(fg).insert(Foreground);
        //println!("scale_terminal_system {:?} -> {:?} font_scale = {:?}, w = {}, h = {}", state.current(), new_state, font_scale, width, height);
        state.set(new_state).unwrap();
    }

}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(UiCameraBundle::default());
}

fn text_update_system(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text, With<FpsText>>) {
    for mut text in query.iter_mut() {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(average) = fps.average() {
                // Update the value of the second section
                text.sections[1].value = format!("{:.0}", average);
            }
        }
    }
}

fn text_color_system(time: Res<Time>,
     terminal: Option<Res<Terminal>>,
     mut foreground: Query<&mut Text, (With<Foreground>, Without<Background>)>,
     mut background: Query<&mut Text, (With<Background>, Without<Foreground>)>,
) {
    if let (Some(terminal) , Some(mut fore), Some(mut back)) =
        (terminal, foreground.iter_mut().next(), background.iter_mut().next()) {
        let k = rand::random::<usize>() % 52;
        let row = rand::random::<usize>() % terminal.rows;
        let col = rand::random::<usize>() % terminal.cols;
        const ALPHABET: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
        if let Some(c) = ALPHABET.chars().nth(k) {
            let seconds = time.seconds_since_startup() as f32;
            let fg = Color::Rgba {
                red: (1.25 * seconds).sin() / 2.0 + 0.5,
                green: (0.75 * seconds).sin() / 2.0 + 0.5,
                blue: (0.50 * seconds).sin() / 2.0 + 0.5,
                alpha: 1.0,
            };
            let bg = Color::Rgba {
                red: 1.0 - fg.r(),
                green: 1.0 - fg.g(),
                blue: 1.0 - fg.b(),
                alpha: 1.0,
            };
            terminal
                .with(fore.as_mut(), back.as_mut())
                .mvaddch(row, col, c, &fg, &bg);
        }
    }
}