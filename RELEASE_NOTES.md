# Release Notes - v0.1.0

## What is in this release

This is the initial release of the Windows Instagram Media Downloader.

### Key additions

- CLI downloader: Run `insta --download <url>` to fetch and save public Instagram photos, videos, reels, and carousel posts directly into your Windows Downloads folder.
- Local web dashboard: Run `insta --ui` to start a local server on port 2022 and automatically open the full-screen browser interface.
- Full-screen layout: The web view expands edge-to-edge to take advantage of your whole display, avoiding cramped mobile containers.
- Multi-item previews: If a post has multiple slides, each item gets its own card with an embedded video player or photo preview, resolution specs, individual download buttons, and bulk selection controls.
- Single self-contained binary: All web assets (HTML, CSS, JS) are baked directly into `insta.exe`. The output binary is roughly 6.8 MB with no external dependencies required.
- Dynamic path resolution: Automatically finds and writes to the current Windows user's official Downloads folder.
- Robust extraction pipeline: Uses Instagram's internal GraphQL endpoints with fallback support for public embed structures and optional session cookies.
