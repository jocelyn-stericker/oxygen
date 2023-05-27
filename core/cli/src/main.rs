use chrono::prelude::*;
use clap::{Parser, Subcommand};
use color_eyre::eyre::{eyre, Result};
use oxygen_core::analyzer::Analyzer;
use oxygen_core::audio_clip::{AudioBackend, AudioClip};
use oxygen_core::db::Db;
use std::{ffi::OsStr, path::Path, sync::mpsc::channel};

#[derive(Parser, Debug)]
#[clap(name = "oxygen")]
#[clap(
    about = "A voice journal and audio analysis toolkit for people who want to change the way their voice comes across."
)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,

    #[cfg(feature = "jack")]
    #[clap(global = true, long)]
    /// On Linux, use the jack backend instead of the alsa backend.
    ///
    /// Note that this requires that the app was compiled with the "jack" feature
    /// (e.g., `cargo run --features=jack -- --jack`)
    jack: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Record an audio clip using the default input device until ctrl+c is pressed.
    Record {
        /// The name of the clip to record. If not specified, the current date and time will be
        /// used.
        name: Option<String>,
    },
    /// List all clips.
    List {},
    /// Play the clip with the given name.
    #[clap(arg_required_else_help = true)]
    Play {
        /// The name of the clip to play.
        name: String,
    },
    /// Prints a transcript of the clip.
    #[clap(arg_required_else_help = true)]
    Transcribe {
        /// The name of the clip to transcribe.
        name: String,
    },
    /// Rename a clip with the given name.
    #[clap(arg_required_else_help = true)]
    Rename {
        /// The old name.
        old_name: String,

        /// The new name.
        new_name: String,
    },
    /// Delete the clip with the given name.
    #[clap(arg_required_else_help = true)]
    Delete {
        /// The name of the clip to delete.
        name: String,
    },
    /// Import the clip at the given path. If a name is not specified, the clip will be
    /// named after the path.
    #[clap(arg_required_else_help = true)]
    Import {
        /// The path to import.
        path: String,
        /// The name of the clip to import.
        name: Option<String>,
    },
    /// Export the clip with the given name to the given path, as a wav file.
    #[clap(arg_required_else_help = true)]
    Export {
        /// The name of the clip to export.
        name: String,
        /// The path to export to, ending in ".wav".
        path: String,
    },
    #[clap(arg_required_else_help = true)]
    /// Export all clips to the given folder.
    ExportAll { folder: String },
}

fn main() -> Result<()> {
    env_logger::init();
    color_eyre::install()?;
    let args = Cli::parse();
    let db = Db::open()?;

    #[cfg(feature = "jack")]
    let host = match args.jack {
        true => AudioBackend::Jack,
        false => AudioBackend::Default,
    };

    #[cfg(not(feature = "jack"))]
    let host = AudioBackend::Default;

    match args.command {
        Commands::Record { name } => {
            let name = name.unwrap_or_else(|| Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
            if db.load(&name)?.is_some() {
                return Err(eyre!("There is already a clip named {}", name));
            }
            let handle = AudioClip::record(host, name)?;

            let (tx, rx) = channel();
            ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))?;

            println!("Waiting for Ctrl-C...");
            rx.recv()?;
            println!("Got it! Exiting...");

            let mut clip = handle.stop();

            eprintln!("Recorded {} samples", clip.samples.len());
            db.save(&mut clip)?;
        }
        Commands::List {} => {
            println!("{:5} {:30} {:30}", "id", "name", "date");
            for entry in db.list()? {
                println!(
                    "{:5} {:30} {:30}",
                    entry.id,
                    entry.name,
                    entry.date.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S")
                )
            }
        }
        Commands::Play { name } => {
            if let Some(clip) = db.load(&name)? {
                let handle = clip.play(host)?;
                let (done_tx, done_rx) = channel::<()>();
                handle.connect_done(move || {
                    done_tx.send(()).unwrap();
                });
                done_rx.recv()?;
            } else {
                return Err(eyre!("No such clip."));
            }
        }
        Commands::Transcribe { name } => {
            let analyzer = Analyzer::new()?;
            if let Some(clip) = db.load(&name)? {
                for segment in &analyzer.transcribe(&clip)? {
                    println!(
                        "{:10.3} - {:10.3} {:30}",
                        (segment.0).0,
                        (segment.0).1,
                        segment.1
                    )
                }
            } else {
                return Err(eyre!("No such clip."));
            }
        }
        Commands::Rename { old_name, new_name } => {
            db.rename(&old_name, &new_name)?;
        }
        Commands::Delete { name } => {
            db.delete(&name)?;
        }
        Commands::Import { name, path } => {
            let name = match name {
                Some(name) => name,
                None => Path::new(&path)
                    .file_stem()
                    .ok_or_else(|| eyre!("Invalid path: {}", path))?
                    .to_str()
                    .ok_or_else(|| eyre!("Path is not utf8"))?
                    .to_string(),
            };
            if db.load(&name)?.is_some() {
                return Err(eyre!("There is already a clip named {}", name));
            }
            let mut clip = AudioClip::import(name, path)?;
            db.save(&mut clip)?;
        }
        Commands::Export { name, path } => {
            if let Some(clip) = db.load(&name)? {
                clip.export(&path)?
            } else {
                return Err(eyre!("No such clip."));
            }
        }
        Commands::ExportAll { folder } => {
            let path = Path::new(&folder);
            if !path.exists() {
                std::fs::create_dir(path)?;
            }
            let mut children = path.read_dir()?;
            if children.next().is_some() {
                return Err(eyre!("Expected {} to be empty.", folder));
            }

            for entry in db.list()? {
                if let Some(clip) = db.load(&entry.name)? {
                    let safe_name = Path::new(&entry.name)
                        .file_name()
                        .unwrap_or_else(|| OsStr::new("invalid"))
                        .to_str()
                        .ok_or_else(|| eyre!("Path is not valid utf8"))?
                        .to_string();
                    let export_path =
                        path.join(Path::new(&format!("{}_{}.wav", entry.id, safe_name)));
                    let export_path = export_path
                        .to_str()
                        .ok_or_else(|| eyre!("Path is not utf8"))?;
                    clip.export(export_path)?;
                } else {
                    return Err(eyre!("{} was removed during export.", entry.name));
                }
            }

            eprintln!("Exported to {}", folder);
        }
    }

    Ok(())
}
