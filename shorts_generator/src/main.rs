use clap::Parser;
use shorts_generator::config::AppConfig;
use shorts_generator::{video_processing, subtitle_generation}; // Removed init_ffmpeg from here
use anyhow::{Result, Error, Context};
use std::path::Path;
use std::fs;
use log::{info, error, warn}; // Added log imports

#[derive(Parser, Debug)]
#[clap(name = "shorts_wizard", version = "0.1.0", author = "AI Agent")]
struct Cli {
    #[clap(subcommand)]
    command: CliCommand,
}

#[derive(Parser, Debug)]
enum CliCommand {
    #[clap(about = "Generate video shorts directly with specified configuration options")]
    Generate(AppConfig), // AppConfig already derives Parser and flattens Video/SubtitleConfig

    #[clap(about = "Configure and save settings to a JSON file")]
    Configure {
        #[clap(long, help = "Path to save the configuration JSON file")]
        output_config_path: String,

        #[clap(flatten)]
        config: AppConfig, // Flatten AppConfig to get all its arguments
    },

    #[clap(about = "Run video generation using a configuration file")]
    RunFromFile {
        #[clap(long, help = "Path to the configuration JSON file")]
        config_path: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    env_logger::init(); // Initialize logger

    match cli.command {
        CliCommand::Generate(config) => {
            info!("Starting video generation with directly provided config...");
            if let Err(e) = process_video_with_config(config.clone()) {
                error!("Video processing failed: {:?}", e);
                std::process::exit(1);
            }
            info!("Video processing completed successfully.");
        }
        CliCommand::Configure { output_config_path, config } => {
            info!("Saving configuration to {}...", output_config_path);
            if let Err(e) = config.save_to_file(&output_config_path) {
                error!("Failed to save configuration: {:?}", e);
                std::process::exit(1);
            }
            info!("Configuration saved successfully to {}", output_config_path);
        }
        CliCommand::RunFromFile { config_path } => {
            info!("Loading configuration from {}...", config_path);
            let config = match AppConfig::load_from_file(&config_path) {
                Ok(c) => c,
                Err(e) => {
                    error!("Failed to load configuration from '{}': {:?}", config_path, e);
                    std::process::exit(1);
                }
            };
            info!("Starting video generation with config from file: {}...", config_path);
            if let Err(e) = process_video_with_config(config.clone()) {
                error!("Video processing failed: {:?}", e);
                std::process::exit(1);
            }
            info!("Video processing completed successfully.");
        }
    }

    Ok(())
}

fn process_video_with_config(config: AppConfig) -> Result<(), Error> {
    shorts_generator::init_ffmpeg();
    info!("Starting video processing for: {}", config.video.output_path);

    // Create a temporary processing directory
    let output_dir_path = Path::new(&config.video.output_path)
        .parent()
        .ok_or_else(|| Error::msg(format!("Invalid output path (could not get parent directory): {}", config.video.output_path)))?;
    
    let input_file_stem = Path::new(&config.video.input_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("video");
    let temp_dir_name = format!("{}_processing_temp_{}", input_file_stem, chrono::Utc::now().timestamp_millis());
    let temp_dir = output_dir_path.join(&temp_dir_name);
    
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)
            .with_context(|| format!("Failed to clean up existing temp directory: {:?}", temp_dir))?;
    }
    fs::create_dir_all(&temp_dir)
        .with_context(|| format!("Failed to create temp directory: {:?}", temp_dir))?;

    info!("Temporary processing directory created at: {:?}", temp_dir);

    // Trim Video
    let trimmed_video_filename = format!("{}_trimmed.mp4", input_file_stem);
    let trimmed_video_path = temp_dir.join(&trimmed_video_filename);
    let trimmed_video_path_str = trimmed_video_path.to_str()
        .ok_or_else(|| Error::msg("Failed to convert trimmed video path to string"))?;

    info!("Trimming video: {} to {}s. Output: {}", config.video.input_path, config.video.short_duration_secs, trimmed_video_path_str);
    video_processing::trim_video(
        &config.video.input_path,
        trimmed_video_path_str,
        0.0, // Assuming start from beginning for the short
        config.video.short_duration_secs as f64,
    )
    .with_context(|| format!("Failed to trim video from '{}'", config.video.input_path))?;
    info!("Video trimmed successfully. Output: {}", trimmed_video_path_str);

    let final_output_path_str = &config.video.output_path;

    if config.subtitles.use_subtitles {
        info!("Subtitle generation enabled.");
        // Extract Audio
        let audio_filename = format!("{}_extracted_audio.wav", input_file_stem);
        let audio_path = temp_dir.join(&audio_filename);
        let audio_path_str = audio_path.to_str()
            .ok_or_else(|| Error::msg("Failed to convert audio path to string"))?;

        info!("Extracting audio from: {}. Output: {}", trimmed_video_path_str, audio_path_str);
        video_processing::extract_audio(trimmed_video_path_str, audio_path_str)
            .with_context(|| format!("Failed to extract audio from '{}'", trimmed_video_path_str))?;
        info!("Audio extracted successfully. Output: {}", audio_path_str);

        // Generate Subtitle File
        info!("Generating subtitles for: {}. Model: {}", audio_path_str, config.subtitles.whisper_model_path);
        let subtitle_file_path_str = subtitle_generation::generate_subtitle_file(
            audio_path_str,
            &config.subtitles.whisper_model_path,
            temp_dir.to_str().ok_or_else(|| Error::msg("Failed to convert temp_dir to string for subtitle generation"))?,
        )
        .with_context(|| "Failed to generate subtitle file")?;
        info!("Subtitles generated successfully. Output: {}", subtitle_file_path_str);

        // Burn Subtitles
        info!("Burning subtitles from {} into video. Output: {}", subtitle_file_path_str, final_output_path_str);
        video_processing::burn_subtitles(
            trimmed_video_path_str,
            &subtitle_file_path_str,
            final_output_path_str,
            &config.subtitles.font_path,
            config.subtitles.font_size,
            &config.subtitles.font_color,
            &config.subtitles.subtitle_position_vertical_alignment,
            &config.subtitles.subtitle_position_horizontal_alignment,
        )
        .with_context(|| format!("Failed to burn subtitles onto '{}'", trimmed_video_path_str))?;
        info!("Subtitles burned successfully.");

    } else {
        info!("Subtitle generation disabled. Copying trimmed video to output: {}", final_output_path_str);
        fs::rename(&trimmed_video_path, Path::new(final_output_path_str))
            .or_else(|e| {
                warn!("Failed to move trimmed video (attempting copy instead): {:?}", e);
                fs::copy(&trimmed_video_path, Path::new(final_output_path_str)).map(|_| ()).map_err(anyhow::Error::from)
            })
            .and_then(|_| { 
                if Path::new(final_output_path_str).exists() && trimmed_video_path.exists() {
                    fs::remove_file(&trimmed_video_path)
                        .with_context(|| format!("Failed to remove original trimmed video after copy: {:?}", trimmed_video_path))?; // Add ? to propagate anyhow::Error
                }
                Ok(()) // Ensure this path returns Ok(()) of the correct type
            })
            .with_context(|| {
                format!(
                    "Failed to move or copy trimmed video from {:?} to {}",
                    trimmed_video_path, final_output_path_str
                )
            })?;
        info!("Trimmed video moved/copied to: {}", final_output_path_str);
    }

    info!("Cleaning up temporary directory: {:?}", temp_dir);
    fs::remove_dir_all(&temp_dir)
        .with_context(|| format!("Failed to clean up temp directory: {:?}", temp_dir))?;
    info!("Temporary directory cleaned up successfully.");
    
    info!("Video processing completed successfully for: {}", config.video.output_path);
    Ok(())
}
