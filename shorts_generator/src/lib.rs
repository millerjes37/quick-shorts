pub mod config;
pub mod video_processing;
pub mod subtitle_generation;
pub use config::{AppConfig, SubtitleConfig, VideoConfig};
pub use video_processing::{extract_audio, trim_video, burn_subtitles}; // Updated this line
pub use subtitle_generation::generate_subtitle_file;

// Initialize FFmpeg globally for the library.
// This should ideally be called by the application, but for simplicity in this context,
// we'll do it here. It's safe to call multiple times.
// TODO: Consider moving initialization to the application or providing an explicit init function.
// Made this function public within the crate so video_processing.rs can call it.
// Making it fully public so main.rs can also call it easily.
pub fn init_ffmpeg() {
    static FFMPEG_INIT: std::sync::Once = std::sync::Once::new();
    FFMPEG_INIT.call_once(|| {
        ffmpeg_next::init().expect("Failed to initialize FFmpeg");
    });
}


#[cfg(test)]
mod tests {
    use super::*; // Ensure init_ffmpeg is in scope if needed for tests later

    #[test]
    fn it_works() {
        init_ffmpeg(); // Example: Call if tests use ffmpeg directly or indirectly
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
