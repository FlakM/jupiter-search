use anyhow::{anyhow, Result};
use clap::Parser;
use cli::Cli;
use jupiter_downloader::{DownloadParams, Downloader};

mod cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    let downloader = Downloader::default();

    match &cli.command {
        cli::Commands::Rss {
            rss_url,
            num_of_episodes,
            worker_count,
        } => {
            let (threads_per_worker, workers) = validate_worker_params(&cli, worker_count)?;
            let wd = match cli.output_path {
                None => std::env::current_dir()?,
                Some(dir) => dir,
            };

            let params = DownloadParams {
                rss_url,
                worker_count: workers,
                model_file_path: &cli.model_path,
                output_dir: &wd,
                n_elements: *num_of_episodes,
                debug: cli.debug,
                threads_per_worker,
            };
            let results = downloader.download_rss(params).await?;

            println!("{:?}", results);
        }

        cli::Commands::File { auido_file: _ } => todo!(),
    }

    Ok(())
}

fn validate_worker_params(params: &Cli, worker_count: &Option<usize>) -> Result<(usize, usize)> {
    let num_cpus: usize = std::thread::available_parallelism()
        .map_err(|e| anyhow!("unable to obtain paralelism {}", e))?
        .get();
    let threads_per_worker = params.threads_per_worker;

    let workers = match *worker_count {
        None => num_cpus / threads_per_worker,
        Some(n) => n,
    };
    if workers * threads_per_worker > num_cpus {
        return Err(anyhow!("Provided parameter --worker_count ({}) and --threads_per_worker ({}) multiplied should be less then total avaialable parallelism ({})",
            workers, threads_per_worker, num_cpus
                ));
    }

    eprintln!(
        "Picked number of workers {}, each with {} threads",
        workers, threads_per_worker
    );

    Ok((threads_per_worker, workers))
}
