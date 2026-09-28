use clap::Parser;
use insta::cli::Cli;
use insta::downloader::Downloader;
use insta::extractor::Extractor;
use insta::server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    if args.ui {
        server::run_server(args.port).await?;
        return Ok(());
    }

    if let Some(url) = args.download {
        println!("Fetching media for: {}", url);

        let extractor = Extractor::new();
        let post = match extractor.fetch_post(&url, args.cookie.as_deref()).await {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        };

        if let Some(owner) = &post.owner_username {
            println!("Post by: @{}", owner);
        }
        if let Some(caption) = &post.caption {
            let excerpt = if caption.len() > 80 {
                format!("{}...", &caption[..80])
            } else {
                caption.clone()
            };
            println!("Caption: {}", excerpt);
        }

        println!("Found {} media item(s). Starting download...", post.items.len());

        let downloader = Downloader::new();
        let results = downloader.download_post(&post, args.output.as_deref()).await;

        let mut success_count = 0;
        let mut fail_count = 0;

        for res in results {
            if res.success {
                println!("Saved: {}", res.path);
                success_count += 1;
            } else {
                eprintln!("Failed to save {}: {}", res.filename, res.error.unwrap_or_default());
                fail_count += 1;
            }
        }

        println!("Summary: {} downloaded, {} failed", success_count, fail_count);
        return Ok(());
    }

    println!("No action specified. Run with --ui to open the web interface, or --download <URL> to download media.");
    println!("Run with --help for full usage information.");
    Ok(())
}
