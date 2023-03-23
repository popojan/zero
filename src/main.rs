pub mod terminal;
pub mod grammar;
pub mod derivation;
mod input;

use std::collections::HashMap;
use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use terminal::TerminalPlugin;
use terminal::TerminalEvent;
use crate::derivation::Derivation;
use crate::grammar::Grammar2D;
use crate::input::KeyCodeExt;
use crate::terminal::{Terminal, TerminalNew, TerminalReady};
use std::env;
use bevy::audio::{Audio, AudioSource};
use std::path::PathBuf;
use std::time::Duration;
use bevy::app::AppExit;
use bevy::time::common_conditions::on_timer;
use bevy::window::WindowMode;
use bevy::asset::LoadState;

extern crate rand;

const FAST_STEP: f64 = 0.002;
const SLOW_STEP: f64 = 0.25;
const MIN_CHAR_WIDTH: u16 = 80;
const MIN_CHAR_HEIGHT: u16 = 35;
const NUM_DERIVATIONS_PER_TICK: u8 = 1;
const PROGRAM_FILE: &str = "assets/programs/menu.cfg";

#[derive(States, Clone, Eq, Debug, Hash, PartialEq, Copy, Default)]
enum AppState {
    #[default]
    Paused,
    Running,
}

#[derive(Resource)]
pub struct AudioState {
    pub audio_loaded: bool,
    pub audio_destroy: bool,
    sound_handles: HashMap<char, Handle<AudioSource>>,
}

#[derive(Resource)]
struct KeyRepeatTiming(HashMap<KeyCode, f64>);

#[derive(Resource)]
struct ProgramFile(String);

#[derive(Resource)]
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
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (640., 480.).into(),
                mode: WindowMode::BorderlessFullscreen,
                //present_mode: PresentMode::AutoVsync,
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugin(TerminalPlugin::new())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(ProgramFile(program_file.clone()))
        .insert_resource(KeyRepeatTiming(Default::default()))
        .insert_resource(RewardAccumulator{
            score: 0,
            time: 0,
            errors: 0
        })
        .insert_resource(FixedTime::new_from_secs(fast_step as f32))
        .add_startup_system(prepare_audio)
        .add_state::<AppState>()
        //.add_system(display_fps_system)
        .add_system(bevy::window::close_on_esc)
        //.add_system(bevy::window::exit_on_all_closed)
        .add_system(clear_grammar_system)
        .add_system(start_grammar_system)
        .add_system(check_audio_loading)
        .add_system(grammar_derivation_system_t.in_schedule(CoreSchedule::FixedUpdate))
        .add_system(grammar_derivation_system_b.run_if(on_timer(Duration::from_secs_f64(slow_step))))
        .add_system(grammar_derivation_system_m.run_if(on_timer(Duration::from_secs_f64(0.1*slow_step))))
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

fn prepare_audio(mut commands: Commands, program_file: Res<ProgramFile>,
                 asset_server: ResMut<AssetServer>
) {

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
    mut next_state: ResMut<NextState<AppState>>,
    state: Res<State<AppState>>,
    asset_server: ResMut<AssetServer>,
) {
    if derivation.iter().count() <= 0  {
        if let Some(_ready) = is_ready.iter().next() {
            if let Some(terminal) = terminals.iter().next() {
                if state.0 != AppState::Paused {
                    next_state.set(AppState::Paused);
                }
                let mut grammar = Grammar2D::default();
                grammar.load(&program_file.0);
                let mut derivation = Derivation::new(
                    grammar, terminal.rows, terminal.cols);

                for e in derivation.start() {
                    term.send(e);
                }
                commands.spawn(derivation);

                prepare_audio(commands, program_file, asset_server);
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

fn grammar_derivation_system(time_step_code: KeyCode,
                             mut commands: Commands,
                             program_file: Res<ProgramFile>,
                             terminal: Query<&Terminal>,
                             time: Res<Time>,
                             audio: Res<Audio>,
                             mut accumulator: ResMut<RewardAccumulator>,
                             audio_state: Res<AudioState>,
                             state: Res<State<AppState>>,
                             mut next_state: ResMut<NextState<AppState>>,
                             mut key_repeat_times: ResMut<KeyRepeatTiming>,
                             mut keyboard_input: ResMut<Input<KeyCode>>,
                             mut derivation: Query<&mut Derivation>, mut events: EventWriter<TerminalEvent>,
                             mut exit: EventWriter<AppExit>
) {
    let new_state = if state.0 == AppState::Paused {AppState::Running} else {AppState::Paused};

    let current_time = time.elapsed_seconds_f64();
    keyboard_input
        .get_just_pressed().for_each(|&x| {
            key_repeat_times.0.insert(x.clone(), current_time);
    });

    if let Some(terminal) = terminal.iter().next() {
        if let Some(derive) = derivation.iter_mut().next().as_mut() {
            if state.0 == AppState::Paused {
                let msg_pad =
                    if terminal.cols > derive.grammar.help.chars().count() {
                        std::iter::repeat(" ")
                            .take(terminal.cols - 1 - derive.grammar.help.chars().count())
                            .collect::<String>()
                    } else {
                        String::from("")
                    };
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
            let time_lapse = if state.0 == AppState::Running { time_step } else { vec![] };

            let iter = keyboard_input
                .get_just_pressed().filter(|&x| {
                (x == &KeyCode::Space) || (time_step_code == KeyCode::T)
            })
                .chain(keyboard_input.get_pressed().filter(|&x| {
                    time_step_code == KeyCode::M && x != &KeyCode::Space
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
                        if state.0 != new_state {
                            next_state.set(new_state);
                        }
                        break;
                    }
                    let mut repeat_times = 1;
                    if c == 'B' {
                        accumulator.time += 1;
                    } else if c == 'T' {
                        repeat_times = NUM_DERIVATIONS_PER_TICK;
                    }
                    for _ in 1..(repeat_times + 1) {
                        let result = derive.step(c);
                        if result.sound_alias == '>' {
                            let mut new_program = PathBuf::from(program_file.0.clone());
                            new_program.pop();
                            let new_file = result.dbg_rule.split(" ").last().unwrap();
                            new_program.push(new_file);
                            if !new_program.exists() {
                                exit.send(AppExit);
                            }
                            else {
                                let new_program = new_program.to_str().unwrap().to_string();
                                next_state.set(AppState::Paused);
                                commands.insert_resource(ProgramFile(new_program));
                                events.send(TerminalEvent::clear());
                            }
                            break;
                        }
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
                                audio.play(sound_handle_ref.clone());
                            }
                        }
                    }
                }
                if time_step_code == KeyCode::T {
                    cleared.push(key_code.clone());
                }
            }
            cleared.iter().for_each(|input| {
                if input != &KeyCode::Escape {
                    keyboard_input.clear_just_pressed(*input);
                }
            });
        }
    }
}

fn grammar_derivation_system_t(commands: Commands,
                               terminal: Query<&Terminal>,
                               program_file: Res<ProgramFile>,
                               time: Res<Time>,
                               audio: Res<Audio>,
                               accumulator: ResMut<RewardAccumulator>,
                               audio_state: Res<AudioState>,
                               state: Res<State<AppState>>,
                               next_state: ResMut<NextState<AppState>>,
                               key_repeat_times: ResMut<KeyRepeatTiming>,
                               keyboard_input: ResMut<Input<KeyCode>>,
                               derivation: Query<&mut Derivation>, events: EventWriter<TerminalEvent>,
                               exit: EventWriter<AppExit>) {
        grammar_derivation_system(KeyCode::T, commands, program_file, terminal, time,
                                  audio, accumulator, audio_state, state, next_state, key_repeat_times,
                                  keyboard_input, derivation, events, exit);
}

fn grammar_derivation_system_b(commands: Commands,
                               terminal: Query<&Terminal>,
                               program_file: Res<ProgramFile>,
                               time: Res<Time>,
                               audio: Res<Audio>,
                               accumulator: ResMut<RewardAccumulator>,
                               audio_state: Res<AudioState>,
                               state: Res<State<AppState>>,
                               next_state: ResMut<NextState<AppState>>,
                               key_repeat_times: ResMut<KeyRepeatTiming>,
                               keyboard_input: ResMut<Input<KeyCode>>,
                               derivation: Query<&mut Derivation>, events: EventWriter<TerminalEvent>,
                               exit: EventWriter<AppExit>
) {
    grammar_derivation_system(KeyCode::B, commands, program_file, terminal, time,
                              audio, accumulator, audio_state, state, next_state,key_repeat_times,
                              keyboard_input, derivation, events, exit);
}

fn grammar_derivation_system_m(commands: Commands,
                               terminal: Query<&Terminal>,
                               program_file: Res<ProgramFile>,
                               time: Res<Time>,
                               audio: Res<Audio>,
                               accumulator: ResMut<RewardAccumulator>,
                               audio_state: Res<AudioState>,
                               state: Res<State<AppState>>,
                               next_state: ResMut<NextState<AppState>>,
                               key_repeat_times: ResMut<KeyRepeatTiming>,
                               keyboard_input: ResMut<Input<KeyCode>>,
                               derivation: Query<&mut Derivation>, events: EventWriter<TerminalEvent>,
                               exit: EventWriter<AppExit>
) {

    grammar_derivation_system(KeyCode::M, commands, program_file, terminal, time,
                              audio, accumulator, audio_state, state, next_state, key_repeat_times,
                              keyboard_input, derivation, events, exit);
}