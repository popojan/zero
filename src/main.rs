pub mod terminal;
pub mod grammar;
pub mod derivation;
mod input;

use std::collections::HashMap;
use std::default::Default;
use bevy::core::FixedTimestep;
use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use terminal::TerminalPlugin;
use terminal::TerminalEvent;
use crate::derivation::Derivation;
use crate::grammar::Grammar2D;
use crate::input::KeyCodeExt;
use crate::terminal::{Terminal, TerminalNew, TerminalReady};
use std::env;
use bevy_kira_audio::{Audio, AudioChannel, AudioPlugin, AudioSource};
use bevy::asset::LoadState;

extern crate rand;

const FAST_STEP: f64 = 0.001;
const SLOW_STEP: f64 = 0.25;
const PROGRAM_FILE: &str = "programs/highnoon.cfg";

#[derive(Clone, Eq, Debug, Hash, PartialEq, Copy)]
enum AppState {
    Paused,
    Running,
}

struct AudioState {
    audio_loaded: bool,
    sound_handle: Handle<AudioSource>,
    channels: HashMap<AudioChannel, ChannelAudioState>,
}
struct ChannelAudioState {
    stopped: bool,
    paused: bool,
}

impl Default for ChannelAudioState {
    fn default() -> Self {
        ChannelAudioState {
            stopped: true,
            paused: false,
        }
    }
}

struct KeyRepeatTiming(HashMap<KeyCode, f64>);

struct ProgramFile(String);

fn main() {
    let args: Vec<String> = env::args().collect();
    let (program_file, fast_step, slow_step) = match args.len() {
        1 => (PROGRAM_FILE.to_string(), FAST_STEP, SLOW_STEP),
        2 => (args[1].clone(), FAST_STEP, SLOW_STEP),
        3 => (args[1].clone(), args[2].parse::<f64>().unwrap(), SLOW_STEP),
        _ => (args[1].clone() , args[2].parse::<f64>().unwrap(), args[3].parse::<f64>().unwrap()),
    };
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(TerminalPlugin::new())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(AudioPlugin)
        .insert_resource(ProgramFile(program_file.clone()))
        .insert_resource(KeyRepeatTiming(Default::default()))
        .add_startup_system(prepare_audio.system())
        .add_state(AppState::Paused)
        .add_system(display_fps_system)
        .add_system(clear_grammar_system)
        .add_system(start_grammar_system)
        .add_system(check_audio_loading.system())
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
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}

fn check_audio_loading(mut audio_state: ResMut<AudioState>, asset_server: ResMut<AssetServer>) {
    if audio_state.audio_loaded
        || LoadState::Loaded != asset_server.get_load_state(&audio_state.sound_handle)
    {
        return;
    }
    audio_state.audio_loaded = true;
}

fn prepare_audio(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    let mut channels = HashMap::new();
    channels.insert(
        AudioChannel::new("first".to_owned()),
        ChannelAudioState::default(),
    );
    channels.insert(
        AudioChannel::new("second".to_owned()),
        ChannelAudioState::default(),
    );
    let sound_handle = asset_server.load("sounds/beep.wav");
    let audio_state = AudioState {
        channels,
        audio_loaded: false,
        sound_handle,
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

fn display_fps_system(diagnostics: Res<Diagnostics>, mut events: EventWriter<TerminalEvent>) {
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
                             mut audio_state: ResMut<AudioState>,
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
            let mut score = 0;
            let mut errs = 0;
            let mut dbg_rule = String::from("");
            let mut cleared = Vec::<KeyCode>::default();
            let time_step = vec![time_step_code];
            let time_lapse = if state.current() == &AppState::Running { time_step } else {vec![]};

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
                    let result = derive.step(c, &mut score, &mut dbg_rule, &mut errs);
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
                    if time_step_code == KeyCode::T {
                        cleared.push(key_code.clone());
                    } else if time_step_code == KeyCode::B {
                        if audio_state.audio_loaded {
                            let audio_channel = &audio_state.channels.iter().next().unwrap().0.clone();
                            let channel_audio_state = audio_state.channels.get_mut(audio_channel).unwrap();
                            channel_audio_state.paused = false;
                            channel_audio_state.stopped = false;
                            audio.play_in_channel(audio_state.sound_handle.clone(), &audio_channel);
                        }
                    }
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
                               audio_state: ResMut<AudioState>,
                               state: ResMut<State<AppState>>,
                               key_repeat_times: ResMut<KeyRepeatTiming>,
                               keyboard_input: ResMut<Input<KeyCode>>,
                               derivation: Query<&mut Derivation>, events: EventWriter<TerminalEvent>) {
    grammar_derivation_system(KeyCode::T, terminal, time,  audio, audio_state, state, key_repeat_times, keyboard_input, derivation, events);
}

fn grammar_derivation_system_b(terminal: Query<&Terminal>,
                               time: Res<Time>,
                               audio: Res<Audio>,
                               audio_state: ResMut<AudioState>,
                               state: ResMut<State<AppState>>,
                               key_repeat_times: ResMut<KeyRepeatTiming>,
                               keyboard_input: ResMut<Input<KeyCode>>,
                               derivation: Query<&mut Derivation>, events: EventWriter<TerminalEvent>) {
    grammar_derivation_system(KeyCode::B, terminal, time,  audio, audio_state, state, key_repeat_times, keyboard_input, derivation, events);
}

fn grammar_derivation_system_m(terminal: Query<&Terminal>,
                               time: Res<Time>,
                               audio: Res<Audio>,
                               audio_state: ResMut<AudioState>,
                               state: ResMut<State<AppState>>,
                               key_repeat_times: ResMut<KeyRepeatTiming>,
                               keyboard_input: ResMut<Input<KeyCode>>,
                               derivation: Query<&mut Derivation>, events: EventWriter<TerminalEvent>) {
    grammar_derivation_system(KeyCode::M, terminal, time, audio, audio_state, state, key_repeat_times, keyboard_input, derivation, events);
}