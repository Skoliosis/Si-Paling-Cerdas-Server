use crate::{player::Player, state::GameQuestion};
use anyhow::anyhow;
use mysql::{prelude::Queryable, Conn, OptsBuilder, Row};
use std::{path::PathBuf, time::Duration};

pub struct Database {
    pub con: Conn,
}

pub struct FriendInfo {
    pub id: i32,
    pub name: String,
    pub online: bool,
    pub pfp_ext: String,
    pub pfp: Vec<u8>,
}

pub struct FriendRequestInfo {
    pub id: i32,
    pub name: String,
    pub pfp_ext: String,
    pub pfp: Vec<u8>,
}

pub struct LeaderboardInfo {
    pub win: i32,
    pub lose: i32,
    pub rating: i32,

    pub name: String,
    pub pfp_ext: String,

    pub pfp: Vec<u8>,
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}

impl Database {
    pub fn new() -> Self {
        let db_opts = OptsBuilder::new()
            .user(Some("root"))
            .pass(Some("root"))
            .ip_or_hostname(Some("localhost"))
            .tcp_port(3306)
            .db_name(Some("si_paling_cerdas"))
            .tcp_connect_timeout(Some(Duration::from_secs(3)));

        let mut con = Conn::new(db_opts).unwrap();
        Self::run_migrations(&mut con);

        Self { con }
    }

    fn run_migrations(con: &mut Conn) {
        let mut files = std::fs::read_dir("runtime/migrations")
            .expect("Cannot read directory runtime/migrations")
            .map(|x| x.expect("Cannot read DirEntry"))
            .map(|x| x.path())
            .collect::<Vec<PathBuf>>();
        files.sort();

        println!("Running {} migration(s)...", files.len());

        for file in files {
            let path = file.to_str().expect("Cannot convert file to absolute path");
            let query = std::fs::read_to_string(path)
                .unwrap_or_else(|_| panic!("Cannot read file {}", path));

            println!("Migrations: running file {}", path);

            con.query_drop(&query)
                .unwrap_or_else(|e| panic!("Failed to run query: {}, error: {}", query, e));
        }
    }

    pub fn save_player_name(&mut self, id: i32, name: &str) -> anyhow::Result<()> {
        self.con
            .exec_drop("UPDATE Players SET Name = ? WHERE ID = ?;", (name, id))?;

        Ok(())
    }

    pub fn insert_new_player(&mut self, rid: &str) -> anyhow::Result<()> {
        const QUERY: &str = "
            INSERT INTO Players (
                RID,
                ProfilePicture
            ) VALUES (
                ?,
                ?
            ) 
            RETURNING ID;
        ";

        let data = std::fs::read("runtime/EmptyProfilePicture.png")?;
        let id: Option<i32> = self.con.exec_first(QUERY, (rid, data))?;

        if let Some(id) = id {
            self.save_player_name(id, &format!("GUEST_{}", id))?;

            Ok(())
        } else {
            Err(anyhow!("Insert player returns none"))
        }
    }

    pub fn is_player_exist_by_rid(&mut self, rid: &str) -> anyhow::Result<bool> {
        const QUERY: &str = "SELECT EXISTS(SELECT 1 FROM Players WHERE RID = ? LIMIT 1);";

        if let Some(row) = self.con.exec_first(QUERY, (rid,))? {
            Ok(row)
        } else {
            Err(anyhow!("Cannot find player with RID of {}", rid))
        }
    }

    pub fn is_player_exist_by_name(&mut self, name: &str) -> anyhow::Result<bool> {
        const QUERY: &str = "SELECT EXISTS(SELECT 1 FROM Players WHERE Name = ? LIMIT 1);";

        if let Some(row) = self.con.exec_first(QUERY, (name,))? {
            Ok(row)
        } else {
            Err(anyhow!("Cannot find player with Name of {}", name))
        }
    }

    pub fn load_player_by_rid(&mut self, rid: &str, player: &mut Player) -> anyhow::Result<()> {
        const QUERY: &str = "SELECT * FROM Players WHERE RID = ? LIMIT 1;";

        let row: Option<Row> = self.con.exec_first(QUERY, (rid,))?;

        if let Some(row) = row {
            self.load_player_from_row(row, player)
                .ok_or_else(|| anyhow!("Cannot load player"))
        } else {
            Err(anyhow!("Cannot find player with RID of {}", rid))
        }
    }

    pub fn save_rank(&mut self, player: &Player) {
        const QUERY: &str =
            "UPDATE Players SET Rating = ?, WinCount = ?, LoseCount = ? WHERE ID = ?";

        self.con
            .exec_drop(
                QUERY,
                (
                    player.rating,
                    player.win_count,
                    player.lose_count,
                    player.id,
                ),
            )
            .unwrap();
    }

    pub fn load_player_from_row(&self, row: Row, player: &mut Player) -> Option<()> {
        player.id = row.get(0)?;
        player.rid = row.get(1)?;
        player.name = row.get(2)?;
        player.rating = row.get(3)?;
        player.win_count = row.get(4)?;
        player.lose_count = row.get(5)?;
        player.pfp_blob = row.get(6)?;
        player.pfp_ext = row.get(7)?;

        Some(())
    }

    pub fn get_all_questions(&mut self) -> anyhow::Result<Vec<GameQuestion>> {
        let rows: Vec<Row> = self.con.query("SELECT * FROM QuestionLists;")?;
        let mut questions = Vec::new();

        for row in rows {
            questions.push(
                Self::load_question_from_row(row)
                    .ok_or_else(|| anyhow!("Failed to load question from row"))?,
            );
        }

        Ok(questions)
    }

    pub fn load_question_from_row(row: Row) -> Option<GameQuestion> {
        Some(GameQuestion {
            question: row.get(1)?,
            answer_option_1: row.get(2)?,
            answer_option_2: row.get(3)?,
            answer_option_3: row.get(4)?,
            answer_option_4: row.get(5)?,
            answer_index: row.get(6)?,
        })
    }

    pub fn get_leaderboard(&mut self) -> anyhow::Result<Vec<LeaderboardInfo>> {
        let rows: Vec<Row> = self
            .con
            .query("SELECT * FROM Players ORDER BY Rating DESC LIMIT 10;")?;

        let mut leaderboard = Vec::new();
        for row in rows {
            leaderboard.push(
                Self::load_leaderboard_from_row(row)
                    .ok_or_else(|| anyhow!("Load leaderboard fails"))?,
            );
        }

        Ok(leaderboard)
    }

    pub fn get_pfp(&mut self, id: i32) -> anyhow::Result<(Vec<u8>, String)> {
        const QUERY: &str = "
            SELECT ProfilePicture, ProfilePictureExtension
            FROM Players
            WHERE ID = ?
            LIMIT 1;
        ";

        let row: Option<Row> = self.con.exec_first(QUERY, (id,))?;
        if let Some(row) = row {
            Ok((row.get(0).unwrap(), row.get(1).unwrap()))
        } else {
            Err(anyhow!("Cannot find pfp with ID of {}", id))
        }
    }

    pub fn get_pfp_by_name(&mut self, name: &str) -> anyhow::Result<(Vec<u8>, String)> {
        const QUERY: &str = "
            SELECT ProfilePicture, ProfilePictureExtension
            FROM Players
            WHERE Name = ?
            LIMIT 1;
        ";

        let row: Option<Row> = self.con.exec_first(QUERY, (name,))?;
        if let Some(row) = row {
            Ok((row.get(0).unwrap(), row.get(1).unwrap()))
        } else {
            Err(anyhow!("Cannot find pfp with Name of {}", name))
        }
    }

    pub fn get_friends(&mut self, id: i32) -> anyhow::Result<Vec<FriendInfo>> {
        const QUERY: &str = "
            SELECT Players.ID, Players.Name
            FROM FriendLists
            INNER JOIN Players ON FriendLists.FriendID = Players.ID
            WHERE FriendLists.PlayerID = ?
        ";

        let rows: Vec<Row> = self.con.exec(QUERY, (id,))?;

        let mut friends = Vec::new();
        for row in rows {
            let friend = FriendInfo {
                id: row.get(0).unwrap(),
                name: row.get(1).unwrap(),
                pfp_ext: String::new(),
                pfp: Vec::new(),
                online: false,
            };
            friends.push(friend);
        }

        Ok(friends)
    }

    pub fn add_friend(&mut self, id: i32, target: i32) -> anyhow::Result<()> {
        const QUERY: &str = "
            INSERT INTO FriendLists (PlayerID, FriendID)
            VALUES (?, ?)
        ";

        self.con.exec_drop(QUERY, (id, target))?;

        Ok(())
    }

    pub fn get_friend_requests(&mut self, id: i32) -> anyhow::Result<Vec<FriendRequestInfo>> {
        const QUERY: &str = "
            SELECT Players.ID, Players.Name
            FROM FriendRequests
            INNER JOIN Players ON FriendRequests.FriendID = Players.ID
            WHERE FriendRequests.PlayerID = ?
        ";

        let rows: Vec<Row> = self.con.exec(QUERY, (id,))?;

        let mut friend_requests = Vec::new();
        for row in rows {
            let friend = FriendRequestInfo {
                id: row.get(0).unwrap(),
                name: row.get(1).unwrap(),
                pfp_ext: String::new(),
                pfp: Vec::new(),
            };
            friend_requests.push(friend);
        }

        Ok(friend_requests)
    }

    pub fn add_friend_request(&mut self, id: i32, target: i32) -> anyhow::Result<()> {
        const QUERY: &str = "
            INSERT INTO FriendRequests (PlayerID, FriendID, DateRequested)
            VALUES (?, ?, CURDATE())
        ";

        self.con.exec_drop(QUERY, (id, target))?;

        Ok(())
    }

    pub fn remove_friend_request(&mut self, id: i32, target: i32) -> anyhow::Result<()> {
        const QUERY: &str = "
            DELETE FROM FriendRequests
            WHERE PlayerID = ? AND FriendID = ?
        ";

        self.con.exec_drop(QUERY, (id, target))?;

        Ok(())
    }

    pub fn load_leaderboard_from_row(row: Row) -> Option<LeaderboardInfo> {
        Some(LeaderboardInfo {
            name: row.get(2)?,
            rating: row.get(3)?,
            win: row.get(4)?,
            lose: row.get(5)?,
            pfp: row.get(6)?,
            pfp_ext: row.get(7)?,
        })
    }

    pub fn update_profile_picture(
        &mut self,
        id: i32,
        pfp_blob: &[u8],
        extension: &str,
    ) -> anyhow::Result<()> {
        self.con.exec_drop(
            "UPDATE Players SET ProfilePicture = ?, ProfilePictureExtension = ? WHERE ID = ?;",
            (pfp_blob, extension, id),
        )?;

        Ok(())
    }
}
