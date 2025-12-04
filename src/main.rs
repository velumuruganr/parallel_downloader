use anyhow::{Result, anyhow};
use clap::Parser;
use reqwest::header::{CONTENT_LENGTH, RANGE};

#[derive(Debug, Clone, Copy)]
struct Chunk {
    start: u64,
    end: u64,
}

/// A fast, concurrent file downloader built in Rust.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The URL of the file to download
    #[arg(short, long)]
    url: String,

    /// The output file name
    #[arg(short, long)]
    output: Option<String>,

    /// Number of threads to use
    #[arg(short = 't', long, default_value_t = 4)]
    threads: u8,
}

fn calculate_chunks(total_size: u64, num_threads: u64) -> Vec<Chunk> {
    let mut chunks = Vec::new();
    let chunk_size = total_size / num_threads;

    for i in 0..num_threads {
        let start = i * chunk_size;

        let end = if i == num_threads - 1 {
            total_size - 1
        } else {
            (start + chunk_size) - 1
        };

        chunks.push(Chunk { start, end })
    }

    chunks
}

async fn get_file_size(url: &str) -> Result<u64> {
    let client = reqwest::Client::new();

    let response = client.head(url).send().await?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Request failed. Status Code: {}",
            response.status()
        ));
    }

    let headers = response.headers();
    let content_length = headers
        .get(CONTENT_LENGTH)
        .ok_or(anyhow!("Content Length not found in response header."))?
        .to_str()?
        .parse::<u64>()?;

    Ok(content_length)
}

async fn download_chunk(url: String, chunk: Chunk, output_file: String) -> Result<()> {
    println!("  -> Starting chunk: {}-{}", chunk.start, chunk.end);

    let client = reqwest::Client::new();

    let range_header = format!("bytes={}-{}", chunk.start, chunk.end);
    
    let mut response = client.get(&url).header(RANGE, range_header).send().await?;

    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .open(&output_file)
        .await
        .context("Failed to open file")?;


    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    println!("Starting download for: {}", args.url);

    let file_size = get_file_size(&args.url).await?;
    println!("File Size: {}", file_size);

    let chunks = calculate_chunks(file_size, args.threads as u64);

    for (i, chunk) in chunks.iter().enumerate() {
        println!(
            "Thread: {}, Start: {}, End: {} (Size: {} bytes)",
            i + 1,
            chunk.start,
            chunk.end,
            chunk.end - chunk.start + 1
        );
    }
    Ok(())
}
