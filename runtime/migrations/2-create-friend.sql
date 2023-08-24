CREATE TABLE IF NOT EXISTS FriendLists (
    PlayerID INT NOT NULL,
    FriendID INT NOT NULL,

    FOREIGN KEY (PlayerID) REFERENCES Players (ID),
    FOREIGN KEY (FriendID) REFERENCES Players (ID),
    PRIMARY KEY (PlayerID, FriendID)
);

CREATE TABLE IF NOT EXISTS FriendRequests (
    PlayerID INT NOT NULL,
    FriendID INT NOT NULL,
    DateRequested DATE NOT NULL,

    FOREIGN KEY (PlayerID) REFERENCES Players (ID),
    FOREIGN KEY (FriendID) REFERENCES Players (ID),
    PRIMARY KEY (PlayerID, FriendID)
)
