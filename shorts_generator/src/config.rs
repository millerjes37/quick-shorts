use serde::{Serialize, Deserialize};
use clap::Parser; // Added clap::Parser

#[derive(Parser, Serialize, Deserialize, Debug, Clone)] // Added Parser, Clone
#[clap(author, version, about, long_about = None)]
pub struct VideoConfig {
    #[clap(long, help = "Path to the input video file")]
    pub input_path: String,
    
    #[clap(long, help = "Path to save the output video file(s)")]
    pub output_path: String,
    
    #[clap(long, default_value = "60", help = "Duration of each short video in seconds")]
    pub short_duration_secs: u64,
    
    #[clap(long, help = "Optional output width for the video")]
    pub output_width: Option<u32>,
    
    #[clap(long, help = "Optional output height for the video")]
    pub output_height: Option<u32>,
}

#[derive(Parser, Serialize, Deserialize, Debug, Clone)] // Added Parser, Clone
#[clap(author, version, about, long_about = None)]
pub struct SubtitleConfig {
    #[clap(long, default_value = "true", help = "Enable or disable subtitle generation and burning")]
    pub use_subtitles: bool,
    
    #[clap(long, help = "Path to the Whisper model file (e.g., tiny.en, base, small, medium, large) or directory")]
    pub whisper_model_path: String,
    
    #[clap(long, help = "Path to the font file for subtitles (.ttf, .otf)")]
    pub font_path: String,
    
    #[clap(long, default_value = "24", help = "Font size for subtitles")]
    pub font_size: u32,
    
    #[clap(long, default_value = "white", help = "Font color for subtitles (e.g., 'white', '#FFFFFF')")]
    pub font_color: String,
    
    #[clap(long, default_value = "bottom", help = "Vertical alignment for subtitles (top, center, bottom)")]
    pub subtitle_position_vertical_alignment: String,
    
    #[clap(long, default_value = "center", help = "Horizontal alignment for subtitles (left, center, right)")]
    pub subtitle_position_horizontal_alignment: String,
}

#[derive(Parser, Serialize, Deserialize, Debug, Clone)] // Added Parser, Clone
#[clap(author, version, about = "Main application configuration for generating video shorts.", long_about = None)]
pub struct AppConfig {
    #[clap(flatten)]
    pub video: VideoConfig,
    
    #[clap(flatten)]
    pub subtitles: SubtitleConfig,
}

impl AppConfig {
    pub fn save_to_file(&self, path: &str) -> Result<(), anyhow::Error> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_file(path: &str) -> Result<Self, anyhow::Error> {
        let json = std::fs::read_to_string(path)?;
        let config = serde_json::from_str(&json)?;
        Ok(config)
    }
}
