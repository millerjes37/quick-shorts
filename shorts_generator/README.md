# Video Shorts Generator

## Overview

`shorts_wizard` is a Rust application designed to quickly create video shorts from longer videos, with the option to automatically generate and burn subtitles using FFmpeg and the Whisper V3 CLI tool.

It allows users to:
- Specify input video, output path, and short duration.
- Optionally generate and burn subtitles onto the video.
- Configure subtitle appearance (font, size, color, position).
- Manage configurations via direct CLI arguments or JSON files.

## Prerequisites

Before building and running this project, ensure you have the following installed:

1.  **Rust**: Latest stable version, installed via `rustup` is recommended.
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

2.  **FFmpeg Libraries (Development Headers)**: Required by the `ffmpeg-next` Rust crate.
    *   On Debian/Ubuntu-based systems, install with:
        ```bash
        sudo apt-get update && sudo apt-get install -y \
            libavformat-dev libavcodec-dev libavutil-dev \
            libavfilter-dev libswscale-dev libavdevice-dev \
            libclang-dev
        ```
    *   For other systems, refer to FFmpeg installation guides and ensure development headers are included.

3.  **Whisper V3 CLI Tool**: This project uses the command-line interface of OpenAI's Whisper for subtitle generation. It must be installed and accessible in your system's `PATH`.
    *   Refer to the [official Whisper GitHub repository](https://github.com/openai/whisper) for installation instructions.
    *   *Note*: The project was initially specified to potentially run in an environment (like Nix) where Whisper V3 is available. Ensure it's correctly set up.

4.  **Whisper V3 Model File**: Download a Whisper V3 model file (e.g., `large-v3.bin`, `medium.bin`, `base.en.bin`). The path to this model will be needed for subtitle generation.

5.  **Font File**: A TrueType (`.ttf`) or OpenType (`.otf`) font file is required if you plan to burn subtitles.

## Building the Project

1.  **Clone the Repository** (if applicable, or ensure you are in the project's root directory where `Cargo.toml` is located).

2.  **Build the project**:
    *   For a debug build:
        ```bash
        cargo build
        ```
        The executable will be located at `target/debug/shorts_wizard`.
    *   For a release build (recommended for performance):
        ```bash
        cargo build --release
        ```
        The executable will be located at `target/release/shorts_wizard`.

## Usage (`shorts_wizard` CLI)

The `shorts_wizard` executable provides several commands to manage configurations and generate videos.

### Main Commands:

1.  **`generate`**: Generate a short video directly by providing all configuration options as command-line arguments.
    ```bash
    ./target/debug/shorts_wizard generate --input-path <INPUT.MP4> --output-path <OUTPUT.MP4> [OTHER_OPTIONS...]
    ```

2.  **`configure`**: Create and save a JSON configuration file with the specified options.
    ```bash
    ./target/debug/shorts_wizard configure --output-config-path config.json --input-path <INPUT.MP4> [OTHER_OPTIONS...]
    ```

3.  **`run-from-file`**: Generate a short video using a previously saved JSON configuration file.
    ```bash
    ./target/debug/shorts_wizard run-from-file --config-path config.json
    ```

### Getting Help:

-   For an overview of commands:
    ```bash
    ./target/debug/shorts_wizard --help
    ```
-   For help with a specific command and its options:
    ```bash
    ./target/debug/shorts_wizard generate --help
    ./target/debug/shorts_wizard configure --help
    ```

### Key Configuration Options (Flags):

These options can be used with the `generate` and `configure` commands.

*   `--input-path <PATH>`: Path to the input video file.
*   `--output-path <PATH>`: Path to save the output video short.
*   `--short-duration-secs <SECONDS>`: Duration of the short video in seconds (default: 60).
*   `--output-width <PIXELS>`: (Optional) Width of the output video.
*   `--output-height <PIXELS>`: (Optional) Height of the output video.
*   `--use-subtitles <true|false>`: Enable or disable subtitle generation and burning (default: true).
*   `--whisper-model-path <PATH>`: Path to the Whisper model file or directory.
*   `--font-path <PATH>`: Path to the font file for subtitles.
*   `--font-size <SIZE>`: Font size for subtitles (default: 24).
*   `--font-color <COLOR>`: Font color (e.g., 'white', '#FFFFFF') (default: "white").
*   `--subtitle-position-vertical-alignment <ALIGN>`: Vertical alignment (top, center, bottom) (default: "bottom").
*   `--subtitle-position-horizontal-alignment <ALIGN>`: Horizontal alignment (left, center, right) (default: "center").

## Configuration File

The JSON configuration file used by the `configure` and `run-from-file` commands mirrors the structure of the command-line flags.

**Example `config.json`:**
```json
{
  "video": {
    "input_path": "path/to/your/video.mp4",
    "output_path": "path/to/your/short.mp4",
    "short_duration_secs": 60,
    "output_width": null,
    "output_height": null
  },
  "subtitles": {
    "use_subtitles": true,
    "whisper_model_path": "path/to/your/whisper-large-v3.bin",
    "font_path": "path/to/your/font.ttf",
    "font_size": 24,
    "font_color": "white",
    "subtitle_position_vertical_alignment": "bottom",
    "subtitle_position_horizontal_alignment": "center"
  }
}
```

## Logging

The application uses `env_logger` for logging. The log level can be controlled using the `RUST_LOG` environment variable.

Examples:
-   Run with info-level logging:
    ```bash
    RUST_LOG=info ./target/debug/shorts_wizard generate ...
    ```
-   Run with debug-level logging (more verbose):
    ```bash
    RUST_LOG=debug ./target/debug/shorts_wizard generate ...
    ```
-   For even more detailed logs from specific modules (like `ffmpeg_next`):
    ```bash
    RUST_LOG=shorts_generator=debug,ffmpeg_next=trace ./target/debug/shorts_wizard generate ...
    ```
