use std::path::Path;

use rusqlite::Connection;

pub struct Archive {
    db: Connection
}

impl Archive {
    pub fn new(db_path: &str) -> Result<Self, rusqlite::Error> {
        Self::connect(db_path).map(|db| Self { db })
    }

    fn connect(path: &str) -> rusqlite::Result<Connection> {
        let db_exists = Path::exists(&Path::new(path));
        let db = Connection::open(path)?;

        if !(db_exists) {
            db.execute(
                "CREATE TABLE channel (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    description TEXT,
                    avatar_path TEXT
                )",
                ()
            )?;

            db.execute(
                "CREATE TABLE video (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    description TEXT,
                    path TEXT NOT NULL,
                    thumbnail_path TEXT NOT NULL,
                    author TEXT,
                    FOREIGN KEY(author) REFERENCES channel(id)
                )",
                ()
            )?;

            db.execute(
                "CREATE TABLE playlist (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    author TEXT,
                    FOREIGN KEY(author) REFERENCES channel(id)
                )",
                ()
            )?;

            db.execute(
                "CREATE TABLE playlist_video (
                    index INTEGER NOT NULL,
                    playlist_id TEXT NOT NULL,
                    video_id TEXT NOT NULL,
                    FOREIGN KEY(playlist_id) REFERENCES playlist(id),
                    FOREIGN KEY(video_id) REFERENCES video(id)
                    PRIMARY KEY (playlist_id, video_id)
                )",
                ()
            )?;
        }

        Ok(db)
    }
}