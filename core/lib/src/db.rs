use std::path::Path;

use crate::audio_clip::AudioClip;
use crate::internal_encoding::{decode_v0, decode_v1, encode_v1};
use chrono::prelude::*;
use color_eyre::eyre::{eyre, Result};
use directories::ProjectDirs;
use rusqlite::{params, types::Type, Connection};

pub struct Db(Connection);

pub struct ClipMeta {
    pub id: usize,
    pub name: String,
    pub date: DateTime<Utc>,
}
impl Db {
    pub fn open() -> Result<Db> {
        let proj_dirs = ProjectDirs::from("ca", "nettek", "oxygen").ok_or_else(|| {
            eyre!("Could not find project directories (home directory could not be retreived)")
        })?;
        let data_dir = proj_dirs.data_dir();

        std::fs::create_dir_all(data_dir)?;
        let db_file_path = data_dir.join("oxygen.sqlite");

        if Path::new("oxygen.sqlite").exists() && !db_file_path.exists() {
            eprintln!("Migration: moving oxygen.sqlite to {:?}", db_file_path);
            std::fs::copy("oxygen.sqlite", &db_file_path)?;
            std::fs::remove_file("oxygen.sqlite")?;
        }

        let connection = Connection::open(db_file_path)?;
        Self::from_connection(connection)
    }

    pub fn in_memory() -> Result<Db> {
        let connection = Connection::open_in_memory()?;
        Self::from_connection(connection)
    }

    fn from_connection(connection: Connection) -> Result<Db> {
        let user_version: u32 =
            connection.query_row("SELECT user_version FROM pragma_user_version", [], |r| {
                r.get(0)
            })?;
        connection.pragma_update(None, "page_size", 8192)?;
        connection.pragma_update(None, "user_version", 2)?;

        if user_version < 1 {
            eprintln!("Migration: init schema...");
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
            eprintln!("Migration: updating schema to version 2...");
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
            let sample_rate: u32 = row.get(3)?;
            let samples = decode_v1(sample_rate, &bytes).map_err(|_| {
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

    pub fn load_by_id(&self, id: usize) -> Result<Option<AudioClip>> {
        let mut stmt = self
            .0
            .prepare("SELECT id, name, date, sample_rate, opus FROM clips WHERE id = ?1")?;
        let mut clip_iter = stmt.query_map([id], |row| {
            let date: String = row.get(2)?;
            let bytes: Vec<u8> = row.get(4)?;
            let sample_rate: u32 = row.get(3)?;
            let samples = decode_v1(sample_rate, &bytes).map_err(|_| {
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

    pub fn delete_by_id(&self, id: usize) -> Result<()> {
        self.0.execute("DELETE FROM clips WHERE id = ?1", [id])?;

        Ok(())
    }

    pub fn rename(&self, old_name: &str, new_name: &str) -> Result<()> {
        let rows_changed = self.0.execute(
            "UPDATE clips SET name = ?2 WHERE name = ?1",
            [old_name, new_name],
        )?;

        if rows_changed == 0 {
            return Err(eyre!("There is no clip named \"{}\"", old_name));
        }

        Ok(())
    }

    pub fn rename_by_id(&self, id: usize, new_name: &str) -> Result<()> {
        let rows_changed = self.0.execute(
            "UPDATE clips SET name = ?2 WHERE id = ?1",
            params![id, new_name],
        )?;

        if rows_changed == 0 {
            return Err(eyre!("There is no clip with ID {}", id));
        }

        Ok(())
    }
}
