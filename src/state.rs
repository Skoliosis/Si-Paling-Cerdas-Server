use crate::{database::Database, game::Game, player::Player};
use enet::PeerID;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

type PlayersMap = HashMap<PeerID, Rc<RefCell<Player>>>;
pub type PacketSent = (PeerID, Vec<u8>);

#[derive(Default, Debug, Clone)]
pub struct GameQuestion {
    pub question: String,
    pub answer_option_1: String,
    pub answer_option_2: String,
    pub answer_option_3: String,
    pub answer_option_4: String,
    pub answer_index: i32,
}

pub struct State {
    pub games: Vec<Game>,
    pub players: PlayersMap,
    pub packets: Rc<RefCell<Vec<PacketSent>>>,
    pub database: Database,
    pub questions: Vec<GameQuestion>,
    pub last_queue: [Option<PeerID>; 2],
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    pub fn new() -> Self {
        let mut database = Database::new();
        let questions = database.get_all_questions().unwrap();

        Self {
            database,
            questions,

            games: Vec::new(),
            players: HashMap::new(),
            packets: Rc::new(RefCell::new(Vec::new())),
            last_queue: [None, None],
        }
    }

    pub fn add_player(&mut self, peer_id: PeerID) {
        self.players.insert(
            peer_id,
            Rc::new(RefCell::new(Player::new(peer_id, self.packets.clone()))),
        );
    }

    pub fn get_player(&self, peer_id: PeerID) -> Option<Rc<RefCell<Player>>> {
        self.players.get(&peer_id).cloned()
    }

    pub fn remove_player(&mut self, peer_id: PeerID) {
        self.players.remove(&peer_id);

        for last_queue in self.last_queue.iter_mut() {
            if let Some(queue) = last_queue.as_ref() {
                if *queue == peer_id {
                    *last_queue = None;
                }
            }
        }

        self.games.retain(|game| {
            let p1 = game.p1.borrow();
            let p2 = game.p2.borrow();
            p1.peer_id != peer_id && p2.peer_id != peer_id
        });
    }

    pub fn add_game(
        &mut self,
        p1: Rc<RefCell<Player>>,
        p2: Rc<RefCell<Player>>,
        competitive: bool,
    ) -> usize {
        self.games.push(Game::new(p1, p2, competitive));
        self.games.len() - 1
    }
}
