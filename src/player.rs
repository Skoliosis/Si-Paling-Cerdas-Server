use crate::{
    database::{FriendInfo, FriendRequestInfo, LeaderboardInfo},
    protocol,
    state::{GameQuestion, PacketSent},
};
use bson::{doc, spec::BinarySubtype, Array, Binary, Bson, Document};
use enet::PeerID;
use std::{cell::RefCell, rc::Rc};

pub struct Player {
    pub peer_id: PeerID,

    pub id: i32,
    pub rating: i32,
    pub points: i32,
    pub win_count: i32,
    pub lose_count: i32,
    pub game_index: usize,

    pub answered: bool,

    pub rid: String,
    pub name: String,
    pub pfp_ext: String,

    pub pfp_blob: Vec<u8>,

    pub packets: Rc<RefCell<Vec<PacketSent>>>,
}

impl Player {
    pub fn new(peer_id: PeerID, packets: Rc<RefCell<Vec<PacketSent>>>) -> Self {
        Self {
            id: 0,
            rating: 0,
            points: 0,
            win_count: 0,
            lose_count: 0,
            game_index: usize::MAX,

            answered: false,

            rid: String::new(),
            name: String::new(),
            pfp_ext: String::new(),

            pfp_blob: Vec::new(),

            peer_id,
            packets,
        }
    }

    pub fn send_packet(&self, bson: Document) {
        let mut packets = self.packets.borrow_mut();
        packets.push((self.peer_id, bson::to_vec(&bson).unwrap()));
    }

    pub fn send_auth_response(&self, name: &str, pfp_blob: &[u8], pfp_ext: &str, error: bool) {
        self.send_packet(doc! {
            "PacketID": protocol::PACKET_ID_AUTHENTICATION,
            "Name": name,
            "Error": error,
            "ProfilePicture": Binary { subtype: BinarySubtype::Generic, bytes: pfp_blob.to_vec() },
            "ProfilePictureExtension": pfp_ext
        })
    }

    pub fn send_match_notify(&self, name: &str, pfp_blob: &[u8], pfp_ext: &str) {
        self.send_packet(doc! {
            "PacketID": protocol::PACKET_ID_ADD_QUEUE,
            "Name": name,
            "ProfilePicture": Binary { subtype: BinarySubtype::Generic, bytes: pfp_blob.to_vec() },
            "ProfilePictureExtension": pfp_ext
        })
    }

    pub fn send_question_update(&self, points: i32, enemy_points: i32, question: &GameQuestion) {
        self.send_packet(doc! {
            "PacketID": protocol::PACKET_ID_UPDATE_QUESTION,
            "Points": points,
            "Question": question.question.as_str(),
            "EnemyPoints": enemy_points,
            "AnswerOption1": question.answer_option_1.as_str(),
            "AnswerOption2": question.answer_option_2.as_str(),
            "AnswerOption3": question.answer_option_3.as_str(),
            "AnswerOption4": question.answer_option_4.as_str(),
        })
    }

    pub fn send_answer(&self, answer_index: i32) {
        self.send_packet(doc! {
            "PacketID": protocol::PACKET_ID_QUESTION_ANSWER,
            "AnswerIndex": answer_index
        })
    }

    pub fn send_update_name(&self, name: &str, error: bool) {
        self.send_packet(doc! {
            "PacketID": protocol::PACKET_ID_UPDATE_NAME,
            "Name": name,
            "Error": error
        })
    }

    pub fn send_game_ended(&self, winner: &str) {
        self.send_packet(doc! {
            "PacketID": protocol::PACKET_ID_GAME_ENDED,
            "Winner": winner
        })
    }

    pub fn send_friends(&self, friends: Vec<FriendInfo>) {
        let mut array = Array::new();

        for info in friends {
            let pfp = Binary {
                subtype: BinarySubtype::Generic,
                bytes: info.pfp,
            };

            let mut value = Document::new();
            value.insert("ID", info.id);
            value.insert("Name", info.name);
            value.insert("ProfilePicture", pfp);
            value.insert("ProfilePictureExtension", info.pfp_ext);

            array.push(Bson::Document(value));
        }

        self.send_packet(doc! {
            "PacketID": protocol::PACKET_ID_FETCH_FRIENDS,
            "Friends": array
        })
    }

    pub fn send_friend_requests(&self, friend_requests: Vec<FriendRequestInfo>) {
        let mut array = Array::new();

        for info in friend_requests {
            let pfp = Binary {
                subtype: BinarySubtype::Generic,
                bytes: info.pfp,
            };

            let mut value = Document::new();
            value.insert("ID", info.id);
            value.insert("Name", info.name);
            value.insert("ProfilePicture", pfp);
            value.insert("ProfilePictureExtension", info.pfp_ext);

            array.push(Bson::Document(value));
        }

        self.send_packet(doc! {
            "PacketID": protocol::PACKET_ID_FETCH_FRIEND_REQUESTS,
            "FriendRequests": array
        })
    }

    pub fn send_search_name(&self, found: bool, name: &str, pfp_blob: &[u8], pfp_ext: &str) {
        self.send_packet(doc! {
            "PacketID": protocol::PACKET_ID_SEARCH_NAME,
            "Name": name,
            "Found": found,
            "ProfilePicture": Binary { subtype: BinarySubtype::Generic, bytes: pfp_blob.to_vec() },
            "ProfilePictureExtension": pfp_ext
        })
    }

    pub fn send_leaderboard(&self, leaderboard: Vec<LeaderboardInfo>) {
        let mut array = Array::new();

        for info in leaderboard {
            let pfp = Binary {
                subtype: BinarySubtype::Generic,
                bytes: info.pfp,
            };

            let mut value = Document::new();
            value.insert("Win", info.win);
            value.insert("Lose", info.lose);
            value.insert("Name", info.name);
            value.insert("Rating", info.rating);
            value.insert("ProfilePicture", pfp);
            value.insert("ProfilePictureExtension", info.pfp_ext);

            array.push(Bson::Document(value));
        }

        self.send_packet(doc! {
            "PacketID": protocol::PACKET_ID_FETCH_LEADERBOARD,
            "Leaderboard": array
        });
    }

    pub fn send_change_profile_picture(&self, pfp_blob: &[u8], pfp_ext: &str) {
        self.send_packet(doc! {
            "PacketID": protocol::PACKET_ID_CHANGE_PROFILE_PICTURE,
            "ProfilePicture": Binary { subtype: BinarySubtype::Generic, bytes: pfp_blob.to_vec() },
            "ProfilePictureExtension": pfp_ext
        })
    }
}
