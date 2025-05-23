use anyhow::{Result, Error, bail};
use std::process::Command;
use std::path::{Path, PathBuf};

pub fn generate_subtitle_file(
    audio_input_path: &str,
    whisper_model_path: &str,
    output_dir: &str,
) -> Result<String, Error> {
    let audio_path = Path::new(audio_input_path);
    let model_path = Path::new(whisper_model_path);
    let out_dir_path = Path::new(output_dir);

    if !audio_path.exists() {
        bail!("Audio input path does not exist: {}", audio_input_path);
    }
    if !model_path.exists() {
        // Note: Whisper might load models by name (e.g., "base", "small") 
        // if a path isn't provided or if the path is a directory containing models.
        // This check assumes whisper_model_path is a direct file path or a directory that Whisper can use.
        // For simplicity, we'll check if the direct path exists.
        // If Whisper CLI supports model names like "base.en", this check might be too strict
        // or needs to be adapted based on how whisper_model_path is intended to be used.
        // For now, assuming it's a path that should exist.
        // bail!("Whisper model path does not exist: {}", whisper_model_path);
    }
    if !out_dir_path.exists() {
        std::fs::create_dir_all(out_dir_path)?;
    } else if !out_dir_path.is_dir() {
        bail!("Output directory path exists but is not a directory: {}", output_dir);
    }

    let mut command = Command::new("whisper");
    command
        .arg(audio_input_path)
        .arg("--model")
        .arg(whisper_model_path)
        .arg("--output_dir")
        .arg(output_dir)
        .arg("--output_format")
        .arg("srt");

    // Optional: Log the command
    // println!("Executing command: {:?}", command);

    let output = command.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "Whisper command failed with status: {}. Stderr: {}",
            output.status,
            stderr
        );
    }

    // Determine the expected output file path
    let audio_file_name = audio_path
        .file_stem()
        .ok_or_else(|| Error::msg(format!("Could not extract file stem from audio path: {}", audio_input_path)))?
        .to_str()
        .ok_or_else(|| Error::msg("Audio file stem is not valid UTF-8"))?;
    
    let srt_file_name = format!("{}.srt", audio_file_name);
    let mut srt_path = PathBuf::from(output_dir);
    srt_path.push(srt_file_name);

    if !srt_path.exists() {
        // Whisper might sometimes put files in a subdirectory named after the model,
        // or have other naming conventions if the input has unusual characters.
        // For now, we assume direct output in output_dir.
        // A more robust solution might involve listing files in output_dir if this assumption fails.
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "Subtitle file not found at expected path: {:?}. Whisper stdout: {}, stderr: {}",
            srt_path,
            stdout,
            stderr
        );
    }

    Ok(srt_path.to_str().unwrap().to_string())
}
