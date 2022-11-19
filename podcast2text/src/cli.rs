use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)] // Read from `Cargo.toml`
#[command(propagate_version = true)]
pub(crate) struct Cli {
    /// Turn debugging information on
    #[arg(short, long, default_value_t = false)]
    pub(crate) debug: bool,

    /// Sets a path to a whisper model that should be used for transcription
    #[arg(short, long, env = "MODEL_PATH")]
    pub(crate) model_path: PathBuf,

    /// Sets a path to output directory. Defaults to working directory.
    #[arg(short, long, env = "OUTPUT_PATH")]
    pub(crate) output_path: Option<PathBuf>,

    /// Sets worker count
    #[arg(short, long, default_value_t = 6)]
    pub(crate) threads_per_worker: usize,

    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Downloads and transcribes all episodes at given url
    Rss {
        /// RSS url
        #[arg(value_name = "RSS_URL")]
        rss_url: String,
        /// Number of episodes to download
        #[arg(short, long)]
        num_of_episodes: Option<usize>,

        /// Sets worker count - will be set to sane default but can be fine tuned
        #[arg(short, long)]
        worker_count: Option<usize>,
    },
    /// Transcribes local audio file
    File {
        /// Sets a path to a audio file
        #[arg(value_name = "AUDIO_FILE")]
        auido_file: PathBuf,
    },
}
