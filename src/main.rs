pub mod terminal;
pub mod grammar;
pub mod derivation;
mod input;

use std::collections::HashMap;
use bevy::time::FixedTimestep;
use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use terminal::TerminalPlugin;
use terminal::TerminalEvent;
use crate::derivation::Derivation;
use crate::grammar::Grammar2D;
use crate::input::KeyCodeExt;
use crate::terminal::{Terminal, TerminalNew, TerminalReady};
use std::env;
use bevy::audio::{Audio, AudioPlugin, AudioSource};
use bevy::asset::LoadState;
use bevy::window::WindowMode;

extern crate rand;

const FAST_STEP: f64 = 0.002;
const SLOW_STEP: f64 = 0.25;
const MIN_CHAR_WIDTH: u16 = 80;
const MIN_CHAR_HEIGHT: u16 = 35;
const NUM_DERIVATIONS_PER_TICK: u8 = 1;
const PROGRAM_FILE: &str = "programs/highnoon.cfg";

#[derive(Clone, Eq, Debug, Hash, PartialEq, Copy)]
enum AppState {
    Paused,
    Running,
}

pub struct AudioState {
    pub audio_loaded: bool,
    pub audio_destroy: bool,
    sound_handles: HashMap<char, Handle<AudioSource>>,
}

struct KeyRepeatTiming(HashMap<KeyCode, f64>);

struct ProgramFile(String);

struct RewardAccumulator {
    score: i64,
    time: i64,
    errors: i64,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let (program_file, fast_step, slow_step) = match args.len() {
        1 => (PROGRAM_FILE.to_string(), FAST_STEP, SLOW_STEP),
        2 => (args[1].clone(), FAST_STEP, SLOW_STEP),
        3 => (args[1].clone(), args[2].parse::<f64>().unwrap(), SLOW_STEP),
        _ => (args[1].clone() , args[2].parse::<f64>().unwrap(), args[3].parse::<f64>().unwrap()),
    };
    App::new()
        .insert_resource(WindowDescriptor {
            width: 640.0,
            height: 480.0,
            position: WindowPosition::Centered(MonitorSelection::Current),
            mode: WindowMode::BorderlessFullscreen,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TerminalPlugin::new())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(AudioPlugin)
        .insert_resource(ProgramFile(program_file.clone()))
        .insert_resource(KeyRepeatTiming(Default::default()))
        .insert_resource(RewardAccumulator{
            score: 0,
            time: 0,
            errors: 0
        })
        .add_startup_system(prepare_audio)
        .add_state(AppState::Paused)
        //.add_system(display_fps_system)
        .add_system(clear_grammar_system)
        .add_system(start_grammar_system)
        .add_system(check_audio_loading)
        .add_system_set(SystemSet::new()
                .with_run_criteria(FixedTimestep::step(fast_step))
                .with_system(grammar_derivation_system_t)
        )
        .add_system_set(SystemSet::new()
                            .with_run_criteria(FixedTimestep::step(slow_step))
                            .with_system(grammar_derivation_system_b)
        )
        .add_system_set(SystemSet::new()
            .with_run_criteria(FixedTimestep::step(0.1*slow_step))
            .with_system(grammar_derivation_system_m)
        )
        .add_system(bevy::window::close_on_esc)
        .run();
}

fn check_audio_loading(mut audio_state: ResMut<AudioState>, asset_server: ResMut<AssetServer>) {
    if audio_state.audio_loaded || audio_state.audio_destroy
    {
        return;
    }
    let mut loading = false;
    for (_sound_alias, sound_handle) in audio_state.sound_handles.iter() {
        loading |= LoadState::Loaded != asset_server.get_load_state(sound_handle);
    }
    if loading {
        return;
    }
    audio_state.audio_loaded = true;
}

fn prepare_audio(mut commands: Commands, program_file: Res<ProgramFile>, asset_server: ResMut<AssetServer>) {

    let mut grammar = Grammar2D::default();
    grammar.load(&program_file.0);

    let mut sound_handles =  HashMap::<char, Handle<AudioSource>>::new();
    for (sound_alias, sound_file) in grammar.sounds.iter() {
        let sound_handle = asset_server.load(sound_file);
        sound_handles.insert(*sound_alias, sound_handle);
    }
    let audio_state = AudioState {
        audio_loaded: false,
        audio_destroy: false,
        sound_handles,
    };

    commands.insert_resource(audio_state);
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
    program_file: Res<ProgramFile>,
    mut is_ready: EventReader<TerminalReady>,
    mut term: EventWriter<TerminalEvent>,
    mut state: ResMut<State<AppState>>,
) {
    if derivation.iter().count() <= 0  {
        if let Some(_ready) = is_ready.iter().next() {
            if let Some(terminal) = terminals.iter().next() {
                if state.current() != &AppState::Paused {
                    state.set(AppState::Paused).unwrap();
                }
                let mut grammar = Grammar2D::default();
                grammar.load(&program_file.0);
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

fn _display_fps_system(diagnostics: Res<Diagnostics>, mut events: EventWriter<TerminalEvent>) {
    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(average) = fps.average() {
            events.send(TerminalEvent {
                row: 0, col: 1, s: format!("{:.0} fps", average), attr: (Color::WHITE, Color::BLACK)
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

fn grammar_derivation_system(time_step_code: KeyCode, terminal: Query<&Terminal>,
                             time: Res<Time>,
                             audio: Res<Audio>,
                             mut accumulator: ResMut<RewardAccumulator>,
                             audio_state: ResMut<AudioState>,
                             mut state: ResMut<State<AppState>>,
                             mut key_repeat_times: ResMut<KeyRepeatTiming>,
                             mut keyboard_input: ResMut<Input<KeyCode>>,
                             mut derivation: Query<&mut Derivation>, mut events: EventWriter<TerminalEvent>) {
    let new_state = if state.current() == &AppState::Paused {AppState::Running} else {AppState::Paused};

    let current_time = time.seconds_since_startup();
    keyboard_input
        .get_just_pressed().for_each(|&x| {
            key_repeat_times.0.insert(x.clone(), current_time);
    });

    if let Some(terminal) = terminal.iter().next() {
        if let Some(derive) = derivation.iter_mut().next().as_mut() {
            if state.current() == &AppState::Paused {
                let msg_pad = std::iter::repeat(" ")
                    .take(terminal.cols - 1 - derive.grammar.help.chars().count())
                    .collect::<String>();
                events.send(TerminalEvent {
                    row: 0,
                    col: 0,
                    s: format!(" {}{}", derive.grammar.help.to_string(), msg_pad),
                    attr: (Color::WHITE, Color::BLACK)
                });
            }

            let mut _dbg_rule = String::from("");
            let mut cleared = Vec::<KeyCode>::default();
            let time_step = vec![time_step_code];
            let time_lapse = if state.current() == &AppState::Running { time_step } else { vec![] };

            let iter = keyboard_input
                .get_just_pressed().filter(|&x| {
                (x == &KeyCode::Space) || (time_step_code == KeyCode::T)
            })
                .chain(keyboard_input.get_pressed().filter(|&x| {
                    time_step_code == KeyCode::M
                        && ((current_time - key_repeat_times.0.get(x)
                        .unwrap_or(&current_time)) > 0.25)
                }))
                .chain(time_lapse.iter());
            for key_code in iter {
                let shift_down = (key_code == &KeyCode::T)
                    || (key_code == &KeyCode::M)
                    || (key_code == &KeyCode::B)
                    || keyboard_input.pressed(KeyCode::LShift)
                    || keyboard_input.pressed(KeyCode::RShift);
                if let Some(c) = KeyCodeExt(key_code.clone()).to_qwerty_char(shift_down) {
                    if c == ' ' {
                        if state.current() != &new_state {
                            state.set(new_state).unwrap_or_default();
                        }
                    }
                    let mut repeat_times = 1;
                    if c == 'B' {
                        accumulator.time += 1;
                    } else if c == 'T' {
                        repeat_times = NUM_DERIVATIONS_PER_TICK;
                    }
                    for _ in 1..(repeat_times + 1) {
                        let result = derive.step(c);
                        accumulator.score += result.score_delta as i64;
                        accumulator.errors += result.errors_delta as i64;
                        for e in result.terminal_events {
                            events.send(e);
                            let msg_left = format!("Score: {} Time: {} Errors: {}",
                                        accumulator.score,
                                        accumulator.time,
                                        accumulator.errors);
                            let msg_pad = std::iter::repeat(" ")
                                .take(terminal.cols - 2 - msg_left.chars().count() - result.dbg_rule.chars().count())
                                .collect::<String>();
                            let msg = format!(" {}{}{} ", msg_left, msg_pad, result.dbg_rule);

                            events.send(TerminalEvent {
                                row: 0,
                                col: 0,
                                s: msg,
                                attr: (Color::WHITE, Color::BLACK)
                            });
                        }
                        if let Some(sound_handle_ref) = audio_state.sound_handles.get(&result.sound_alias) {
                            if audio_state.audio_loaded {
                                let sound_handle = sound_handle_ref.clone();
                                audio.play(sound_handle);
                            }
                        }
                    }
                }
                if time_step_code == KeyCode::T {
                    cleared.push(key_code.clone());
                }
            }
            cleared.iter().for_each(|input| {
                keyboard_input.clear_just_pressed(*input);
            });
        }
    }

}

fn grammar_derivation_system_t(terminal: Query<&Terminal>,
                               time: Res<Time>,
                               audio: Res<Audio>,
                               accumulator: ResMut<RewardAccumulator>,
                               audio_state: ResMut<AudioState>,
                               state: ResMut<State<AppState>>,
                               key_repeat_times: ResMut<KeyRepeatTiming>,
                               keyboard_input: ResMut<Input<KeyCode>>,
                               derivation: Query<&mut Derivation>, events: EventWriter<TerminalEvent>) {
        grammar_derivation_system(KeyCode::T, terminal, time, audio, accumulator, audio_state, state, key_repeat_times, keyboard_input, derivation, events);
}

fn grammar_derivation_system_b(terminal: Query<&Terminal>,
                               time: Res<Time>,
                               audio: Res<Audio>,
                               accumulator: ResMut<RewardAccumulator>,
                               audio_state: ResMut<AudioState>,
                               state: ResMut<State<AppState>>,
                               key_repeat_times: ResMut<KeyRepeatTiming>,
                               keyboard_input: ResMut<Input<KeyCode>>,
                               derivation: Query<&mut Derivation>, events: EventWriter<TerminalEvent>) {
    grammar_derivation_system(KeyCode::B, terminal, time,  audio, accumulator, audio_state, state, key_repeat_times, keyboard_input, derivation, events);
}

fn grammar_derivation_system_m(terminal: Query<&Terminal>,
                               time: Res<Time>,
                               audio: Res<Audio>,
                               accumulator: ResMut<RewardAccumulator>,
                               audio_state: ResMut<AudioState>,
                               state: ResMut<State<AppState>>,
                               key_repeat_times: ResMut<KeyRepeatTiming>,
                               keyboard_input: ResMut<Input<KeyCode>>,
                               derivation: Query<&mut Derivation>, events: EventWriter<TerminalEvent>) {

    grammar_derivation_system(KeyCode::M, terminal, time, audio, accumulator, audio_state, state, key_repeat_times, keyboard_input, derivation, events);
}