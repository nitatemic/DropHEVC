# DropHEVC

DropHEVC is a fast, beautiful, and cross-platform desktop application (Windows, macOS, Linux) designed to easily convert videos to HEVC (H.265). It automatically optimizes the output for the Apple ecosystem (macOS/iOS natively supported).

## Features

* **Drag & Drop Interface**: Seamlessly drop multiple videos to queue them up.
* **Premium UI**: Custom "Glassmorphism Dark" aesthetic for a modern feel.
* **Smart Hardware Acceleration**: Automatically detects and uses the best available hardware encoder (`hevc_videotoolbox` on Mac, `hevc_nvenc` on NVIDIA, `hevc_amf` on AMD, `hevc_qsv` on Intel) for lightning-fast conversions. Falls back to CPU encoding (`libx265`) if no supported GPU is found.
* **Apple Compatibility**: Injects the `-tag:v hvc1` flag so hardware-encoded HEVC videos from Windows/Linux play natively on iOS and macOS.
* **Zero-Freeze Progress Tracking**: Parses FFmpeg carriage returns (`\r`) in real-time, delivering buttery-smooth progress bars, speed metrics, and current processing time without ever freezing the UI.
* **Auto FFmpeg Management**: Checks if FFmpeg is installed. If missing on Windows or macOS, a single click automatically downloads, extracts, and binds a local copy.

## Technical Stack

*   **Tauri 2**: Lightweight, secure, and highly performant framework.
*   **Backend (Rust)**: Orchestrates tasks, handles FFmpeg IPC, dynamic hardware detection, and asynchronous stream parsing via `tokio`.
*   **Frontend (Vanilla JS, HTML, CSS)**: Zero heavy dependencies. Pure Vanilla JavaScript interacting with `window.__TAURI__` APIs to ensure maximum speed and minimal footprint.
*   **CI/CD**: Fully automated release pipeline via GitHub Actions.

## Project Structure

```text
DropHEVC/
├── package.json             # Node dependencies (Tauri CLI, plugins)
├── src/                     # Frontend Vanilla JS
│   ├── index.html           # Layout structure
│   ├── main.js              # UI logic, queue management, Drag & Drop
│   └── styles.css           # Custom dark theme
└── src-tauri/               # Backend Rust
    ├── Cargo.toml           # Rust dependencies (tokio, reqwest, zip)
    ├── tauri.conf.json      # Tauri 2 configuration
    └── src/
        ├── main.rs          # Entry point
        ├── lib.rs           # Tauri builder and plugin init
        ├── commands.rs      # IPC commands invoked by frontend
        └── ffmpeg.rs        # Hardware detection and FFmpeg downloading logic
```

## Prerequisites

To develop or build this application, you will need:
1.  **Node.js** (v18+)
2.  **Rust** (via rustup) & Cargo.

## Development Setup

1. Clone the repository.
2. Install dependencies:
   ```bash
   npm install
   ```
3. Run the development environment:
   ```bash
   npm run tauri dev
   ```

## Build for Production

To create the final executables (exe, dmg, appimage/deb):

```bash
npm run tauri build
```
The output files will be located in `src-tauri/target/release/bundle/`. 
Alternatively, push a tag (e.g., `v1.0.0`) to trigger the GitHub Actions workflow, which will automatically build and publish the binaries for all platforms.
