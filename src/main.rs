use std::default::Default;
use bevy::core::FixedTimestep;

use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

extern crate rand;

pub mod terminal;
pub mod grammar;
pub mod derivation;

use terminal::TerminalPlugin;
use terminal::TerminalEvent;
use crate::derivation::Derivation;
use crate::grammar::Grammar2D;
use crate::terminal::{Terminal, TerminalNew, TerminalReady};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(TerminalPlugin::new())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_system(display_fps_system)
        .add_system(clear_grammar_system)
        .add_system(start_grammar_system)
        .add_system_set(
            SystemSet::new()
               .with_run_criteria(FixedTimestep::step(0.001))
                .with_system(grammar_derivation_system)
        )
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}

fn clear_grammar_system(mut commands: Commands,
    mut is_new: EventReader<TerminalNew>,
    derivation: Query<Entity, With<Derivation>>,
) {
    for _event in is_new.iter() {
        for id in derivation.iter() {
            commands.entity(id).despawn();
        }
    }
}

fn start_grammar_system(mut commands: Commands,
    terminals: Query<&Terminal>,
    derivation: Query<Entity, With<Derivation>>,
    mut is_ready: EventReader<TerminalReady>,
    mut term: EventWriter<TerminalEvent>,
) {
    if derivation.iter().count() <= 0  {
        if let Some(_ready) = is_ready.iter().next() {
            if let Some(terminal) = terminals.iter().next() {

                let mut grammar = Grammar2D::default();
                grammar.load("programs/life.cfg");
                let mut derivation = Derivation::new(
                    grammar, terminal.rows, terminal.cols);

                for e in derivation.start() {
                    term.send(e);
                }
                commands.spawn().insert(derivation);
            }
        }
    }
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

fn _random_text_system(time: Res<Time>, terminal: Query<&Terminal>, mut events: EventWriter<TerminalEvent>) {

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

fn grammar_derivation_system(_time: Res<Time>, terminal: Query<&Terminal>,
                             mut derivation: Query<&mut Derivation>, mut events: EventWriter<TerminalEvent>) {

    if let Some(terminal) = terminal.iter().next() {
        if let Some(derive) = derivation.iter_mut().next().as_mut() {
            let mut score = 0;
            let mut errs = 0;
            let mut dbg_rule = String::from("");
            let result = derive.step('T', &mut score, &mut dbg_rule, &mut errs);
            for e in result {
                //for e in step.iter() {
                events.send(e);
                use std::ops::Add;
                let msg = "          ".to_string().add(&dbg_rule);
                events.send(TerminalEvent {
                    row: 0,
                    col: terminal.cols - msg.chars().count() - 1,
                    s: msg,
                    attr: (Color::WHITE, Color::BLACK)
                });
            }
        }
    }
}