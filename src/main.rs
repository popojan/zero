use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

extern crate rand;

pub mod terminal;
use terminal::TerminalPlugin;
use terminal::TerminalEvent;
use crate::terminal::Terminal;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(TerminalPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_system(display_fps_system)
        .add_system(random_text_system)
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}

fn display_fps_system(diagnostics: Res<Diagnostics>, mut events: EventWriter<TerminalEvent>) {
    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(average) = fps.average() {
            events.send(TerminalEvent {
                row: 0, col: 0, s: format!("{:.0} fps", average), attr: (Color::WHITE, Color::BLACK)
            });
        }
    }
}

fn random_text_system(time: Res<Time>, terminal: Query<&Terminal>, mut events: EventWriter<TerminalEvent>) {

    if let Some(x) = terminal.iter().next() {
        let seconds = time.seconds_since_startup() as f32;

        let (rows, cols) = x.getmaxxy();

        let row = 1 + rand::random::<usize>() % (rows - 1);
        let col = rand::random::<usize>() % cols;

        const ALPHABET: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let k = rand::random::<usize>() % ALPHABET.chars().count();
        let c = ALPHABET.chars().nth(k).unwrap();

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
        events.send(TerminalEvent{ row, col, s: c.to_string(), attr: (fg, bg, ) });
    }
}