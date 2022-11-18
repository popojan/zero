use std::default::Default;
use std::collections::HashMap;
use bevy::prelude::Color;
use bevy::prelude::Component;
use bevy::utils::HashSet;

use crate::grammar::Rule;
use crate::grammar::Grammar2D;
use crate::terminal::TerminalEvent;

#[derive(Clone, Copy)]
struct G {
    c: char,
    fore: u8,
    back: u8,
    z_ord: u8,
}

#[derive(Component)]
pub struct Derivation {
    pub grammar: Grammar2D,
    rows: i32, cols: i32,
    current: Vec<Vec<G>>,
    memory: Vec<Vec<G>>,
    x: HashMap<(usize, usize), char>,
    colours: Vec<Color>,
}

pub struct DerivationResult {
    pub terminal_events: Vec<TerminalEvent>,
    pub score_delta: i32,
    pub errors_delta: i32,
    pub dbg_rule: String,
    pub sound_alias: char,
}
impl Default for DerivationResult {
    fn default() -> Self {
        DerivationResult {
            terminal_events: Default::default(),
            score_delta: 0,
            errors_delta: 0,
            dbg_rule: "".to_string(),
            sound_alias: ' ',
        }
    }
}
impl Derivation {
    pub fn new(grammar: Grammar2D, rows: usize, cols: usize) -> Self {
        Derivation {
            grammar,
            rows: rows as i32,
            cols: cols as i32,
            current: vec![vec![G { c: ' ', fore: 7, back: 0, z_ord: 'a' as u8 }; cols]; rows],
            memory:  vec![vec![G { c: ' ', fore: 7, back: 0, z_ord: 'a' as u8 }; cols]; rows],
            x: Default::default(),
            colours: vec![
                Color::BLACK,
                Color::RED,
                Color::GREEN,
                Color::YELLOW,
                Color::BLUE,
                Color::FUCHSIA,
                Color::CYAN,
                Color::WHITE,
            ]
        }
    }

    //TODO refactor bevy deps out of Derivation
    fn get_color(&self, fore: u8, back: u8) -> (Color, Color) {
        (self.colours[fore.try_into().unwrap_or(7)], self.colours[back.try_into().unwrap_or(0)])
    }
    pub fn start(&mut self) -> Vec<TerminalEvent> {
        let colour = self.get_color(7, 0);
        self.grammar.seeds.iter().map(
            |seed| {
                let col = match seed.lr {
                    'l' => 0,
                    'r' => (self.cols - 1) as usize,
                    'c' => (self.cols / 2) as usize,
                    'R' => 2*((self.cols - 1)/2) as usize,
                    'C' => 2*((self.cols / 2)/2) as usize,
                    'X' => 2*((rand::random::<usize>() % (self.cols as usize))/2),
                    _ => rand::random::<usize>() % (self.cols as usize),
                };
                let row = match seed.ul {
                    'u' => 1,
                    'l' => (self.rows - 1) as usize,
                    'c' => (self.rows / 2) as usize,
                    'L' => 2*((self.rows - 2)/2) as usize,
                    'C' => 2*((self.rows / 2 - 1)/2) as usize,
                    'X' => 2*((rand::random::<usize>() % ((self.rows - 1) as usize))/2),
                    _ => rand::random::<usize>() % ((self.rows - 1) as usize)  + 1,
                };
                self.x.insert((row, col), seed.c);

                let cursor = self.current
                    .get_mut(row).unwrap()
                    .get_mut(col).unwrap();
                cursor.c = seed.c;
                cursor.fore = 7;
                cursor.back = 0;
                cursor.z_ord = 'a' as u8;

                TerminalEvent {
                    row,
                    col,
                    s: seed.c.to_string(),
                    attr: colour,
                }
            }
        ).collect()
    }
    fn is_rule_applicable(&self, ro: i32, co: i32, rule: &Rule) -> bool {
        let mut r = ro;
        let mut c = co;

        let horizontal = rule.cq > rule.co;
        for p in rule.rhs_all.chars() {
            if p == '\n' {
                r += 1;
                c = co;
                continue;
            }
            if p == ' ' {
                c += 1;
                continue;
            }
            if horizontal {
                // @ LHS @ >>RHS<<
                if c - co >= rule.cm {
                    c += 1;
                    continue;
                }
            } else {
                if r - ro >= rule.rm {
                    break;
                }
            }
            let mut req = p;
            let mut ctx = '#';
            if r > 0 && r < self.rows && c >= 0 && c < self.cols {
                ctx = self.current[r as usize][c as usize].c;
                if ctx == ' ' {
                    ctx = '~';
                }
            }
            if req == '@' {
                req = rule.lhs;
            }
            if p == '&' {
                req = rule.ctx;
            }
            if req == ' ' {
                req = '~';
            }
            if (
                    (req != '!') && (req != '%') && (req != ctx))
                || ((req == '!') && (ctx == rule.ctx))
                || ((p == '%')  && (ctx != rule.ctx_rep) && (ctx != rule.ctx)
            ) {
                return false;
            }
            c += 1;
        }
        return true;
    }

    fn apply_rule(&mut self, ro: i32, co: i32, rule: &Rule) -> Vec<TerminalEvent> {
        let mut r: i32 = ro;
        let mut c: i32 = co;
        let mut ret = Vec::<TerminalEvent>::default();
        for p in rule.rhs_all.chars() {
            if p == '\n' {
                r += 1;
                c = co;
                continue;
            }
            // @ LHS @ >>RHS<<
            if (rule.cq > rule.co) && ((c - co) <= rule.cm) {
                c+= 1;
                continue;
            }
            if (rule.cq <= rule.co) && ((r - ro) <= rule.rm) {
                c += 1;
                continue;
            }
            let mut saved: G;
            let mut rep = p;

            if rep == '@' {
                rep = rule.rep;
            }
            if rep == '&' {
                rep = rule.ctx_rep;
            }

            let is_nonterminal = self.grammar.nonterminals.contains(&rep);

            if rep != ' ' && r > 0 && r < self.rows.try_into().unwrap() && c >= 0 && c < self.cols.try_into().unwrap(){
                if rep == '~' {
                    rep = ' ';
                }
                let mut back = rule.back;

                // transparent background; take background from memory
                if rule.back > 7 {
                    back = self.memory[r as usize][c as usize].back;
                }
                // to be saved in memory
                let mut d = G {c: rep, fore: rule.fore, back, z_ord: rule.z_ord as u8};

                // special char: restore from memory
                if rep == '$' {
                    d = self.memory[r as usize][c as usize];
                }
                // memory empty
                if d.c == 255 as char {
                    d = G { c: ' ', fore: rule.fore, back, z_ord: 'a' as u8 };
                }
                let cidx = self.get_color(d.fore, d.back);

                if rule.z_ord >= self.memory[r as usize][c as usize].z_ord {
                    ret.push(TerminalEvent{
                        row: r as usize,
                        col: c as usize,
                        s: d.c.to_string(),
                        attr: cidx,
                    });
                    if !is_nonterminal {
                        //terminal symbol: save all
                        saved = d;
                    } else {
                        //nonterminal symbol: replace bg color if any
                        saved = self.memory[r as usize][c as usize];
                        saved.back = d.back; //TODO reconsider
                    }
                    self.current[r as usize][c as usize] = d;
                    self.memory[r as usize][c as usize] = saved;
                }
                if is_nonterminal {
                    self.x.insert((r as usize, c as usize), rep);
                } else {
                    self.x.remove(&(r as usize, c as usize));
                }
            }
            c+= 1;
        }
        return ret;
    }

    pub fn step(&mut self, key: char) -> DerivationResult {
        // choose random nonterminal instance and apply a single random rule
        const MAGIC: char = '?';

        // non-terminals to choose from: LHS's of rules given by key
        let a = HashSet::from_iter(
            self.grammar.rules.iter().map(|(_lhs, rules)| {
                rules.iter()
                    .filter(|&rule| {
                        (rule.key == key) || (rule.key == MAGIC)
                    })
                    .map(|rule| rule.lhs)
            }).flatten()
        );

        // positions of applicable rules
        let xx = Vec::from_iter(
            self.x.iter()
                .filter(|(_position, nonterminal)| {
                    a.contains(nonterminal)
                })
                .map(|(position, _nonterminal)| {
                    position.clone()
                })
        );

        if xx.is_empty() {
            return Default::default();
        }

        let mut applicable_rules = Vec::<(&(usize, usize), &Rule)>::default();
        for position in xx.iter() {
            if let Some(symbol) = self.x.get(position) {
                self.grammar.rules.contains_key(symbol);
                let symbol = self.x.get(&position).unwrap();
                if let Some(rules) = self.grammar.rules.get(symbol) {
                    for rule in rules.iter() {
                        if (rule.key == key) || rule.key == MAGIC {
                            if self.is_rule_applicable(position.0 as i32 - rule.ro, position.1 as i32 - rule.co, rule) {
                                applicable_rules.push((position, rule));
                            }
                        }
                    }
                }
            }
        };

        let weight_sum: f32 = applicable_rules.iter()
            .map(|x| x.1.weight)
            .map(|x| x as f32).sum();
        let uniform_random = weight_sum * (rand::random::<u32>() as f32) / (u32::MAX as f32);

        let mut weight_sum = 0.0;
        let mut rule_chosen: Option<(&(usize, usize), usize)> = None;
        for (idx, (position, rule)) in applicable_rules.iter().enumerate() {
            weight_sum += rule.weight as f32;
            if weight_sum >= uniform_random {
                rule_chosen = Some((position, idx));
                break;
            }
        }
        if let Some(((row, col), idx)) = rule_chosen {
            let rule = applicable_rules[idx].1.clone();
            DerivationResult {
                terminal_events: self.apply_rule(*row as i32 - rule.rq, *col as i32 - rule.cq, &rule),
                score_delta: rule.reward,
                errors_delta: 0,
                dbg_rule: rule.lhs_all.clone(),
                sound_alias: rule.sound,
            }
        } else {
            Default::default()
        }
    }
}
