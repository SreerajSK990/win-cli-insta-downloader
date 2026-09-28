# Windows Instagram Media Downloader

A lightweight, portable Rust app for Windows that lets you download photos, videos, reels, and multi-slide carousel posts from Instagram.

You can use it straight from the terminal or launch an edge-to-edge web dashboard where you can preview each video and photo before saving.

## Why this exists

Most online Instagram downloaders are loaded with ads, popups, and artificial rate limits. This tool compiles down into a single 6.8 MB executable that runs locally on your PC. It has no external runtime requirements (no Node.js, no Python). It connects directly to Instagram endpoints and saves files directly to your Windows Downloads folder.

## Features

- Single portable executable for Windows.
- CLI mode: Pass a link, fetch media, and save directly to your Downloads folder.
- Web UI mode: Runs a local web server on port 2022 and opens your default browser.
- Full-screen design: The interface spreads across your entire display instead of being confined to a narrow mobile column.
- Carousel support: If an Instagram post has 10 slides, you can preview all 10, select individual items, or download everything at once.
- Built-in video player: Watch reel and video previews directly inside the browser cards.
- Optional cookie support: Supply a session cookie if you need to fetch restricted or login-walled posts.

## How to build and run

### Requirements
- Rust and Cargo installed on Windows.

### Build release binary
```bash
cargo build --release
```
The compiled binary will be placed at `target/release/insta.exe`.

### CLI Usage

Download all media from a post directly to your default Downloads folder:
```bash
insta.exe --download https://www.instagram.com/p/SHORTCODE/
```

Specify a custom folder for saving files:
```bash
insta.exe --download https://www.instagram.com/p/SHORTCODE/ --output "C:\Media\Instagram"
```

Pass an optional session cookie:
```bash
insta.exe --download https://www.instagram.com/p/SHORTCODE/ --cookie "sessionid=YOUR_COOKIE"
```

### Web UI Usage

Launch the full-screen interface:
```bash
insta.exe --ui
```

Use a different port if needed:
```bash
insta.exe --ui --port 3000
```

Once running, it opens your browser to `http://localhost:2022`. Paste any Instagram link, click Fetch Media, and all preview cards will render with resolution tags, checkboxes, and download buttons.

## Architecture

- `src/cli.rs`: Command line parsing using clap.
- `src/extractor.rs`: Post shortcode extraction and Instagram GraphQL and embed endpoint fallbacks.
- `src/downloader.rs`: Streaming network chunks directly to disk in your Windows Downloads folder.
- `src/server.rs`: Embedded Axum web server hosting API routes and static frontend files.
- `assets/`: Full-width responsive frontend (HTML, CSS, JS) baked directly inside the executable.
