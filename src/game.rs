use crate::{player::Player, state::GameQuestion};
use std::{cell::RefCell, collections::HashSet, rc::Rc, time::Instant};

pub struct Game {
    pub p1: Rc<RefCell<Player>>,
    pub p2: Rc<RefCell<Player>>,

    pub question: GameQuestion,

    pub stage: i32,
    pub started: bool,
    pub competitive: bool,

    pub start_timer: Instant,
    pub stage_timer: Instant,

    pub previous_questions: HashSet<usize>,
}

impl Game {
    pub fn new(p1: Rc<RefCell<Player>>, p2: Rc<RefCell<Player>>, competitive: bool) -> Self {
        Self {
            p1,
            p2,
            competitive,

            stage: 0,
            started: false,
            question: GameQuestion::default(),

            start_timer: Instant::now(),
            stage_timer: Instant::now(),

            previous_questions: HashSet::new(),
        }
    }
}
