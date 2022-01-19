#[derive(Clone)]
pub struct Rule {
    pub lhs: char,
    pub lhs_all: String,
    pub rhs_all: String,
    pub ro: i32,
    pub co: i32,
    pub  rm: i32,
    pub cm: i32,
    pub rq: i32,
    pub cq: i32,
    pub fore: u8,
    pub back: u8,
    pub reward: i32,
    pub key: char,
    pub ctx: char,
    pub rep: char,
    pub ctx_rep: char,
    pub  weight: i32,
    pub  z_ord: u8,
}

#[derive(Default)]
pub struct Grammar2D {
    pub seeds: Vec<Start>,
    pub rules: HashMap<char, Vec<Rule>>,
    pub nonterminals: HashSet<char>,
}

use std::collections::HashMap;
use std::fs;
use bevy::utils::HashSet;

pub struct Start {
    pub ul: char,
    pub lr: char,
    pub c: char,
}

impl Grammar2D {

    fn at_with_default(line: &str, i: usize, def: char) -> char {
        if let Some(c) = line.chars().nth(i) {
            c
        } else {
            def
        }
    }

    fn _process(&mut self, lhss: &Vec<&str>, rule: &Vec<&str>) -> bool {
        let rhs = rule.join("\n");
        lhss.iter().for_each(|&lhs| self.add_rule(&lhs, &rhs));
        true
    }

    fn origin(_s: char, rhs: &str, spec: char, mut ord: i32) -> (i32, i32) {
        let mut row_off = 0;
        let mut col_off = 0;

        for p in rhs.chars() {
            if p == '\n' {
                row_off += 1;
                col_off = -1;
            } else if p == spec {
                if ord == 0 {
                    return (row_off, col_off);
                }
                ord -= 1;
            }
            col_off += 1;
        }
        (-1, -1)
    }

    fn add_rule(&mut self, lhs: &str, rhs: &str) {
        let s = lhs.chars().nth(1).unwrap();
        if !self.rules.contains_key(&s) {
            self.rules.insert(s, vec![]);
            self.nonterminals.insert(s);
        }
        const ANCHOR_CHAR: char = '@';
        let (ro, co) = Self::origin(s, rhs, ANCHOR_CHAR, 0);
        let (rm, cm) = Self::origin(s, rhs, ANCHOR_CHAR, 1);
        let (rq, cq) = Self::origin(s, rhs, ANCHOR_CHAR, 2);


        let default = (0, 1);
        let (reward, weight) = if lhs.chars().count() > 10 {
            let mut it = lhs[10..].split(' ');
            if let (Some(a), Some(b)) = (it.next(), it.next()) {
                    (
                        a.parse::<i32>().or::<i32>(Ok(0)).unwrap(),
                        b.parse::<i32>().or::<i32>(Ok(1)).unwrap(),
                    )
            } else {
                default
            }
        } else {
            default
        };

        let ctx_0 = Self::at_with_default(lhs, 6, 255 as char);
        let ctx = match ctx_0 {
            '?' => 255 as char,
            _ => ctx_0
        };
        let key = lhs.chars().nth(2).unwrap();
        let rep = lhs.chars().nth(3).unwrap();
        let ctx_rep_0 = Self::at_with_default(lhs, 7, ' ');
        let ctx_rep  = match ctx_rep_0 {
            '*' => s,
            _ => ctx_rep_0
        };

        let lhs_all = String::from(lhs);
        let rhs_all = str::replace(rhs, "*", &s.to_string());

        let rule = Rule {
            lhs: s, lhs_all, rhs_all,
            ro, co, rm, cm, rq, cq,
            fore: (Self::at_with_default(lhs, 4, '7') as u8 - '0' as u8),
            back: (Self::at_with_default(lhs, 5, '8') as u8 - '0' as u8),
            reward, key, ctx,
            rep,
            ctx_rep,
            weight,
            z_ord: Self::at_with_default(lhs, 8, 'a')  as u8,
        };
        self.rules.get_mut(&s).unwrap().push(rule);
    }

    pub fn load(&mut self, filename: &str) {
        println!("{}",filename);
        let mut lhs: Vec<&str> = vec![];
        let mut rhs: Vec<&str> = vec![];
        let mut help = "".to_string();
        let g = fs::read_to_string(filename)
            .expect("Cannot read grammar file.");

        g.split("\n")
            .for_each(
                |line| {
                    if let Some(fc) = line.chars().next() {
                        if fc == '#' { //comment
                            if Self::at_with_default(line, 1, ' ') == '!' {
                                help = String::from_iter(line.chars().skip(2));
                            }
                        } else if fc == '^' {
                            let c = Self::at_with_default(&line, 1, 's');
                            let ul = Self::at_with_default(&line, 2, 'c');
                            let lr = Self::at_with_default(&line, 3, 'c');
                            self.seeds.push(Start { ul, lr, c });
                        } else if fc == '=' {
                            if !rhs.is_empty() && self._process(&lhs, &rhs) {
                                lhs.clear();
                                rhs.clear();
                            }
                            lhs.push(&line);
                        } else {
                            rhs.push(&line);
                        }
                    } else {
                        rhs.push(&line);
                    }
                }
            );
        if !rhs.is_empty() {
            self._process(&lhs, &rhs);
        }
        if self.seeds.is_empty() {
            self.seeds.push(Start {ul: 'c', lr: 'c', c: 'c'});
        }
    }
}