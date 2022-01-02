use crate::audio_clip::AudioClip;
use crate::internal_encoding::{decode_v0, decode_v1, encode_v1};
use chrono::prelude::*;
use color_eyre::Result;
use rusqlite::{params, types::Type, Connection};

pub struct Db(Connection);

pub struct ClipMeta {
    pub id: usize,
    pub name: String,
    pub date: DateTime<Utc>,
}
impl Db {
    pub fn open() -> Result<Db> {
        let connection = Connection::open("oxygen.sqlite")?;
        let user_version: u32 =
            connection.query_row("SELECT user_version FROM pragma_user_version", [], |r| {
                r.get(0)
            })?;
        connection.pragma_update(None, "page_size", 8192)?;
        connection.pragma_update(None, "user_version", 2)?;

        if user_version < 1 {
            eprintln!("Init schema...");
            connection.execute(
                "
                CREATE TABLE IF NOT EXISTS clips (
                  id INTEGER PRIMARY KEY,
                  name TEXT NOT NULL UNIQUE,
                  date TEXT NOT NULL,
                  sample_rate INTEGER NOT NULL,
                  samples BLOB NOT NULL
                );
                ",
                [],
            )?;
        }

        if user_version < 2 {
            eprintln!("Updating schema...");
            let mut stmt =
                connection.prepare("SELECT id, name, date, sample_rate, samples FROM clips")?;
            let clip_iter = stmt.query_map([], |row| {
                let date: String = row.get(2)?;
                let samples: Vec<u8> = row.get(4)?;

                Ok(AudioClip {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                    date: date.parse().map_err(|_| {
                        rusqlite::Error::InvalidColumnType(2, "date".to_string(), Type::Text)
                    })?,
                    sample_rate: row.get(3)?,
                    samples: decode_v0(&samples),
                })
            })?;

            let clips: Vec<_> = clip_iter.collect::<Result<_, rusqlite::Error>>()?;
            for clip in &clips {
                let (sr, bytes) = encode_v1(clip)?;
                connection.execute(
                    "INSERT OR REPLACE INTO clips (id, name, date, sample_rate, samples) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        clip.id,
                        clip.name,
                        clip.date.to_string(),
                        sr,
                        bytes,
                    ],
                )?;
            }

            connection.execute("ALTER TABLE clips RENAME COLUMN samples TO opus", [])?;
        }

        Ok(Db(connection))
    }

    pub fn save(&self, clip: &mut AudioClip) -> Result<()> {
        let (sr, bytes) = encode_v1(clip)?;

        self.0.execute(
            "INSERT OR REPLACE INTO clips (id, name, date, sample_rate, opus) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                clip.id,
                clip.name,
                clip.date.to_string(),
                sr,
                bytes,
            ],
        )?;

        if clip.id.is_none() {
            clip.id = Some(self.0.last_insert_rowid().try_into()?);
        }

        Ok(())
    }

    pub fn load(&self, name: &str) -> Result<Option<AudioClip>> {
        let mut stmt = self
            .0
            .prepare("SELECT id, name, date, sample_rate, opus FROM clips WHERE name = ?1")?;
        let mut clip_iter = stmt.query_map([name], |row| {
            let date: String = row.get(2)?;
            let bytes: Vec<u8> = row.get(4)?;
            let sample_rate: u32 =  row.get(3)?;
            let samples = decode_v1(sample_rate, &bytes)
                    .map_err(|_| {
                    rusqlite::Error::InvalidColumnType(3, "opus".to_string(), Type::Blob)
                })?;

            Ok(AudioClip {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                date: date.parse().map_err(|_| {
                    rusqlite::Error::InvalidColumnType(2, "date".to_string(), Type::Text)
                })?,
                sample_rate,
                samples,
            })
        })?;

        Ok(if let Some(clip) = clip_iter.next() {
            Some(clip?)
        } else {
            None
        })
    }

    pub fn list(&self) -> Result<Vec<ClipMeta>> {
        let mut stmt = self
            .0
            .prepare("SELECT id, name, date FROM clips ORDER BY date")?;
        let clip_iter = stmt.query_map([], |row| {
            let date: String = row.get(2)?;

            Ok(ClipMeta {
                id: row.get(0)?,
                name: row.get(1)?,
                date: date.parse().map_err(|_| {
                    rusqlite::Error::InvalidColumnType(2, "date".to_string(), Type::Text)
                })?,
            })
        })?;

        Ok(clip_iter.collect::<Result<_, rusqlite::Error>>()?)
    }

    pub fn delete(&self, name: &str) -> Result<()> {
        self.0
            .execute("DELETE FROM clips WHERE name = ?1", [name])?;

        Ok(())
    }
}
