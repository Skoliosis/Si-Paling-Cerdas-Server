pub mod database;
pub mod game;
pub mod player;
pub mod protocol;
pub mod state;

use bson::Document;
use enet::{
    Address, BandwidthLimit, ChannelLimit, Enet, EventKind, Host, Packet, PacketMode, Peer,
};
use player::Player;
use rand::Rng;
use state::State;
use std::{
    cell::RefCell,
    net::Ipv4Addr,
    rc::Rc,
    time::{Duration, Instant},
};

fn host_service(host: &mut Host<Rc<RefCell<Player>>>, state: &mut State) {
    let Ok(event) = host.service(Duration::from_millis(10)) else {
        return;
    };

    let Some(mut event) = event else {
        return;
    };

    match event.kind() {
        EventKind::Connect => {
            println!("Player connected!");

            state.add_player(event.peer_id());
        }

        EventKind::Disconnect { data: _ } => {
            println!("Player disconnected!");

            state.remove_player(event.peer_id());
        }

        EventKind::Receive {
            channel_id: _,
            packet,
        } => {
            let Some(player) = state.get_player(event.peer_id()) else {
                return;
            };

            let Ok(bson) = bson::from_slice::<Document>(packet.data()) else {
                println!("Invalid bson");
                event.peer_mut().disconnect_later(0);
                return;
            };

            handle_incoming_packet(state, player, event.peer_mut(), bson);
        }
    }
}

fn handle_incoming_packet(
    state: &mut State,
    player: Rc<RefCell<Player>>,
    peer: &mut Peer<Rc<RefCell<Player>>>,
    bson: Document,
) {
    let rc_player = player.clone();
    let player = &mut player.borrow_mut();

    let Ok(id) = bson.get_i32("PacketID") else {
        println!("PacketID not found");
        peer.disconnect_later(0);
        return;
    };

    match id as u32 {
        protocol::PACKET_ID_AUTHENTICATION => {
            let Ok(rid) = bson.get_str("RID") else {
                println!("Auth RID not found");
                peer.disconnect_later(0);
                return;
            };

            let Ok(exist) = state.database.is_player_exist_by_rid(rid) else {
                println!("is_player_exist_by_rid fails");
                player.send_auth_response("", &[], "", true);
                peer.disconnect_later(0);
                return;
            };

            if !exist {
                if state.database.insert_new_player(rid).is_err() {
                    println!("insert_new_player fails");
                    player.send_auth_response("", &[], "", true);
                    peer.disconnect_later(0);
                    return;
                };
            }

            if state.database.load_player_by_rid(rid, player).is_err() {
                println!("load_player_by_rid fails");
                player.send_auth_response("", &[], "", true);
                peer.disconnect_later(0);
                return;
            }

            player.send_auth_response(&player.name, &player.pfp_blob, &player.pfp_ext, false);
        }

        protocol::PACKET_ID_ADD_QUEUE => {
            let Ok(competitive) = bson.get_bool("Competitive") else {
                println!("Competitive option not found");
                peer.disconnect_later(0);
                return;
            };

            if let Some(queue) = *state.last_queue.get(competitive as usize).unwrap() {
                if queue == player.peer_id {
                    println!("already in queue");
                    peer.disconnect_later(0);
                    *state.last_queue.get_mut(competitive as usize).unwrap() = None;
                    return;
                }

                let Some(other) = state.get_player(queue) else {
                    println!("other player left");
                    peer.disconnect_later(0);
                    *state.last_queue.get_mut(competitive as usize).unwrap() = None;
                    return;
                };

                let other_rc = other.clone();
                let mut other = other.borrow_mut();
                other.send_match_notify(&player.name, &player.pfp_blob, &player.pfp_ext);
                player.send_match_notify(&other.name, &other.pfp_blob, &other.pfp_ext);

                let game_index = state.add_game(rc_player, other_rc, competitive);
                other.game_index = game_index;
                player.game_index = game_index;

                *state.last_queue.get_mut(competitive as usize).unwrap() = None;
            } else {
                *state.last_queue.get_mut(competitive as usize).unwrap() = Some(player.peer_id);
            }
        }

        protocol::PACKET_ID_QUESTION_ANSWER => {
            if player.answered {
                println!("Player already answered");
                peer.disconnect_later(0);
                return;
            }

            let Ok(answer_index) = bson.get_i32("AnswerIndex") else {
                println!("AnswerIndex option not found");
                peer.disconnect_later(0);
                return;
            };

            if player.game_index == usize::MAX {
                println!("Player not in game");
                peer.disconnect_later(0);
                return;
            }

            let game = state.games.get_mut(player.game_index).unwrap();
            if game.question.answer_index == answer_index {
                player.points += 15 - game.stage_timer.elapsed().as_secs() as i32;
            }

            player.answered = true;
            player.send_answer(game.question.answer_index);

            let other = if game.p1.try_borrow().is_err() {
                game.p2.clone()
            } else {
                game.p1.clone()
            };

            let other = other.borrow();
            if other.answered && game.stage_timer.elapsed().as_secs() < 12 {
                game.stage_timer = Instant::now() - Duration::from_secs(12);
            }
        }

        protocol::PACKET_ID_FETCH_LEADERBOARD => {
            let Ok(leaderboard) = state.database.get_leaderboard() else {
                println!("get leaderboard fails");
                peer.disconnect_later(0);
                return;
            };

            player.send_leaderboard(leaderboard);
        }

        protocol::PACKET_ID_CHANGE_PROFILE_PICTURE => {
            let Ok(extension) = bson.get_str("ProfilePictureExtension") else {
                println!("ProfilePictureExtension option not found");
                peer.disconnect_later(0);
                return;
            };

            let Ok(pfp_blob) = bson.get_binary_generic("ProfilePicture") else {
                println!("ProfilePicture option not found");
                peer.disconnect_later(0);
                return;
            };

            if pfp_blob.len() > 10 * 1024 * 1024 {
                println!("File too big");
                peer.disconnect_later(0);
                return;
            }

            state
                .database
                .update_profile_picture(player.id, &pfp_blob, extension)
                .unwrap();

            player.pfp_blob = pfp_blob.to_vec();
            player.pfp_ext = extension.to_string();
            player.send_change_profile_picture(&pfp_blob, extension);
        }

        protocol::PACKET_ID_UPDATE_NAME => {
            let Ok(name) = bson.get_str("Name") else {
                println!("Name option not found");
                peer.disconnect_later(0);
                return;
            };

            let Ok(result) = state.database.is_player_exist_by_name(name) else {
                println!("is_player_exist_by_name fails");
                peer.disconnect_later(0);
                return;
            };

            if !result {
                state.database.save_player_name(player.id, name).unwrap();
                player.name = name.to_string();
            }

            player.send_update_name(name, result);
        }

        protocol::PACKET_ID_FETCH_FRIENDS => {
            let Ok(mut friends) = state.database.get_friends(player.id) else {
                println!("get friends fails");
                peer.disconnect_later(0);
                return;
            };

            for friend in friends.iter_mut() {
                let mut found = false;
                for ply in state.players.values() {
                    if let Ok(ply) = ply.try_borrow() {
                        if ply.id == friend.id {
                            friend.online = true;
                            friend.pfp = ply.pfp_blob.clone();
                            friend.pfp_ext = ply.pfp_ext.clone();
                            found = true;
                        }
                    }
                }

                if !found {
                    let (pfp, pfp_ext) = state.database.get_pfp(player.id).unwrap();
                    friend.pfp = pfp;
                    friend.pfp_ext = pfp_ext;
                }
            }

            player.send_friends(friends);
        }

        protocol::PACKET_ID_FETCH_FRIEND_REQUESTS => {
            let Ok(mut friend_requests) = state.database.get_friend_requests(player.id) else {
                println!("get friend requests fails");
                peer.disconnect_later(0);
                return;
            };

            for friend in friend_requests.iter_mut() {
                let mut found = false;
                for ply in state.players.values() {
                    if let Ok(ply) = ply.try_borrow() {
                        if ply.id == friend.id {
                            friend.pfp = ply.pfp_blob.clone();
                            friend.pfp_ext = ply.pfp_ext.clone();
                            found = true;
                        }
                    }
                }

                if !found {
                    let (pfp, pfp_ext) = state.database.get_pfp(player.id).unwrap();
                    friend.pfp = pfp;
                    friend.pfp_ext = pfp_ext;
                }
            }

            player.send_friend_requests(friend_requests);
        }

        protocol::PACKET_ID_ACCEPT_FRIEND_REQUEST => {
            let Ok(id) = bson.get_i32("ID") else {
                println!("ID option not found");
                peer.disconnect_later(0);
                return;
            };

            if let Err(error) = state.database.remove_friend_request(id, player.id) {
                println!("Remove friend request error: {}", error);
                peer.disconnect_later(0);
                return;
            }

            if let Err(error) = state.database.add_friend(player.id, id) {
                println!("Add friend #1 error: {}", error);
                peer.disconnect_later(0);
                return;
            }

            if let Err(error) = state.database.add_friend(id, player.id) {
                println!("Add friend #2 error: {}", error);
                peer.disconnect_later(0);
                return;
            }
        }

        protocol::PACKET_ID_DECLINE_FRIEND_REQUEST => {
            let Ok(id) = bson.get_i32("ID") else {
                println!("ID option not found");
                peer.disconnect_later(0);
                return;
            };

            if let Err(error) = state.database.remove_friend_request(id, player.id) {
                println!("Remove friend request error: {}", error);
                peer.disconnect_later(0);
                return;
            }
        }

        protocol::PACKET_ID_SEARCH_NAME => {
            let Ok(name) = bson.get_str("Name") else {
                println!("Name option not found");
                peer.disconnect_later(0);
                return;
            };

            let Ok(exist) = state.database.is_player_exist_by_name(name) else {
                player.send_search_name(false, "", &[], "");
                return;
            };

            if !exist || player.name.eq_ignore_ascii_case(name) {
                player.send_search_name(false, "", &[], "");
            } else {
                let Ok((pfp_blob, pfp_ext)) = state.database.get_pfp_by_name(name) else {
                    player.send_search_name(false, "", &[], "");
                    return;
                };

                player.send_search_name(true, name, &pfp_blob, &pfp_ext)
            }
        }

        protocol::PACKET_ID_ADD_FRIEND_REQUEST => {
            let Ok(id) = bson.get_i32("ID") else {
                println!("ID option not found");
                peer.disconnect_later(0);
                return;
            };

            if id == player.id {
                println!("Illegal");
                peer.disconnect_later(0);
                return;
            }

            let _ = state.database.add_friend_request(id, player.id);
        }

        _ => peer.disconnect_later(0),
    }
}

fn send_packets(host: &mut Host<Rc<RefCell<Player>>>, state: &State) {
    let mut packets = state.packets.borrow_mut();
    for _ in 0..packets.len() {
        let (peer_id, data) = packets.remove(0);
        let Some(peer)= host.peer_mut(peer_id) else {
            return;
        };

        let packet = Packet::new(data, PacketMode::ReliableSequenced).unwrap();
        peer.send_packet(packet, 0).unwrap();
    }
}

fn poll_game(state: &mut State) {
    let mut to_remove = Vec::new();

    for (i, game) in state.games.iter_mut().enumerate().rev() {
        let send_question_update = (!game.started && game.start_timer.elapsed().as_secs() >= 3)
            || (game.started && game.stage_timer.elapsed().as_secs() >= 15);

        if send_question_update {
            let mut rng = rand::thread_rng();
            let mut index;
            loop {
                index = rng.gen_range(0..state.questions.len());
                if game.previous_questions.contains(&index) {
                    continue;
                }

                game.previous_questions.insert(index);
                break;
            }

            let question = state.questions.get(index).unwrap();

            let mut p1 = game.p1.borrow_mut();
            let mut p2 = game.p2.borrow_mut();

            game.stage += 1;
            game.question = question.clone();
            game.stage_timer = Instant::now();

            if game.stage > 2 {
                let winner = if p1.points == p2.points {
                    String::from("-")
                } else if p1.points > p2.points {
                    p1.name.clone()
                } else {
                    p2.name.clone()
                };

                p1.send_game_ended(&winner);
                p2.send_game_ended(&winner);
                p1.send_question_update(p1.points, p2.points, question);
                p2.send_question_update(p2.points, p1.points, question);

                if p1.points > p2.points {
                    p1.win_count += 1;
                    p2.lose_count += 1;
                } else {
                    p2.win_count += 1;
                    p1.lose_count += 1;
                }

                if game.competitive && p1.points != p2.points {
                    if p1.points > p2.points {
                        p1.rating += 10;
                    } else {
                        p2.rating += 10;
                    }
                }

                state.database.save_rank(&p1);
                state.database.save_rank(&p2);

                p1.points = 0;
                p2.points = 0;
                p1.answered = false;
                p2.answered = false;
                p1.game_index = usize::MAX;
                p2.game_index = usize::MAX;

                to_remove.push(i);
            } else {
                if !game.started {
                    game.started = true;
                } else {
                    p1.answered = false;
                    p2.answered = false;
                }

                p1.send_question_update(p1.points, p2.points, question);
                p2.send_question_update(p2.points, p1.points, question);
            }
        }
    }

    for index in to_remove {
        state.games.remove(index);
    }
}

fn main() -> ! {
    println!("Si Paling Cerdas Server!");

    let enet = Enet::new().unwrap();
    let address = Address::new(Ipv4Addr::UNSPECIFIED, 17091);

    let mut state = State::new();
    let mut host = enet
        .create_host::<Rc<RefCell<Player>>>(
            Some(&address),
            1024,
            ChannelLimit::Maximum,
            BandwidthLimit::Unlimited,
            BandwidthLimit::Unlimited,
        )
        .unwrap();

    loop {
        host_service(&mut host, &mut state);
        send_packets(&mut host, &state);
        poll_game(&mut state);
    }
}
