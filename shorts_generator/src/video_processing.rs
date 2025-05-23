use anyhow::{Result, Error, bail}; // Added bail
use ffmpeg_next as ffmpeg;
use std::path::Path;

// Ensure FFmpeg is initialized.
// This function is declared in lib.rs and should be called before ffmpeg operations.
// For now, we assume it's handled at a higher level or called within each function if necessary.
// A more robust solution would be to have an explicit init function in the library's public API.
// fn init_ffmpeg_if_needed() {
//     // Placeholder: In a real scenario, this would call or ensure lib.rs:init_ffmpeg() is run.
//     // For now, relying on the init in lib.rs being sufficient for when these functions are called.
//     // Alternatively, each public function here could call crate::init_ffmpeg();
// }

fn ensure_ffmpeg_initialized() {
    // This function ensures that FFmpeg is initialized.
    // It calls the init_ffmpeg function from lib.rs.
    // Note: The init_ffmpeg in lib.rs uses std::sync::Once, so it's safe to call this multiple times.
    crate::init_ffmpeg();
}

pub fn trim_video(
    input_path: &str,
    output_path: &str,
    start_secs: f64,
    duration_secs: f64,
) -> Result<(), Error> {
    ensure_ffmpeg_initialized();

    let mut ictx = ffmpeg::format::input(&Path::new(input_path))?;
    
    let mut opts = ffmpeg::Dictionary::new();
    opts.set("ss", &start_secs.to_string()); // set returns (), no ?
    opts.set("t", &duration_secs.to_string()); // set returns (), no ?
    opts.set("c", "copy"); // Use stream copy // set returns (), no ?

    let mut octx = ffmpeg::format::output_with(&Path::new(output_path), opts)?;

    for ist_stream in ictx.streams() {
        let ist_params = ist_stream.parameters();
        if ist_params.medium() == ffmpeg::media::Type::Video || ist_params.medium() == ffmpeg::media::Type::Audio {
            let mut ost_stream = octx.add_stream(None)?; 
            ost_stream.set_parameters(ist_params.clone());
            // Codec tag is part of parameters, should be copied by set_parameters if relevant.
            // Explicit tag setting removed as it was causing errors and set_parameters should handle it.
        }
    }
    
    octx.set_metadata(ictx.metadata().to_owned());
    octx.write_header()?; // Options should be applied by the context based on the dictionary.

    for (stream, mut packet) in ictx.packets() {
        let ist_idx = stream.index();
        if octx.stream(ist_idx).is_some() { // Check if the stream was actually added to output
            packet.set_stream(ist_idx);
            match packet.write_interleaved(&mut octx) {
                Ok(_) => (),
                Err(e) => {
                    // Log error but continue if it's a minor issue, or break/return
                    eprintln!("Failed to write packet: {}", e);
                    // Depending on the error, you might want to break or return Err(e.into())
                }
            }
        }
    }

    octx.write_trailer()?;
    Ok(())
}

// Helper function to escape paths for FFmpeg filter strings, especially for Windows.
// FFmpeg expects colons to be escaped, e.g., 'C\:/path/to/file.srt'
fn escape_path_for_ffmpeg_filter(path: &str) -> String {
    path.replace(":", "\\:")
}

// Helper function to convert color strings to FFmpeg's &HBBGGRR format (or &HAABBGGRR)
// For simplicity, this version will handle common names and hex codes without alpha.
// FFmpeg's PrimaryColour for ASS/SSA is &HAABBGGRR. For `subtitles` filter, it might be similar.
// Let's assume BGR format for now, &HBBGGRR. Alpha will be FF (opaque).
fn convert_color_to_ffmpeg_bgr(color_str: &str) -> Result<String, Error> {
    let color_str = color_str.trim_start_matches('#');
    match color_str.to_lowercase().as_str() {
        "white" => Ok("&HFFFFFF".to_string()), // BGR: FF FF FF
        "black" => Ok("&H000000".to_string()), // BGR: 00 00 00
        "red"   => Ok("&H0000FF".to_string()), // BGR: 00 00 FF
        "green" => Ok("&H00FF00".to_string()), // BGR: 00 FF 00
        "blue"  => Ok("&HFF0000".to_string()), // BGR: FF 00 00
        hex if hex.len() == 6 => {
            // Assuming RRGGBB input, convert to BBGGRR
            let r = &hex[0..2];
            let g = &hex[2..4];
            let b = &hex[4..6];
            // Check if valid hex
            u8::from_str_radix(r, 16)?;
            u8::from_str_radix(g, 16)?;
            u8::from_str_radix(b, 16)?;
            Ok(format!("&H{}{}{}", b, g, r).to_uppercase())
        }
        _ => bail!("Unsupported color string: {}. Use common names or #RRGGBB hex.", color_str),
    }
}

// Helper function to map alignment strings to FFmpeg's numeric Alignment values (1-9 for numpad layout)
// Vertical: "bottom", "center", "top"
// Horizontal: "left", "center", "right"
fn map_alignment_to_ffmpeg_value(vertical: &str, horizontal: &str) -> Result<u8, Error> {
    match (vertical.to_lowercase().as_str(), horizontal.to_lowercase().as_str()) {
        ("bottom", "left") => Ok(1),
        ("bottom", "center") => Ok(2),
        ("bottom", "right") => Ok(3),
        ("center", "left") | ("middle", "left") => Ok(4),
        ("center", "center") | ("middle", "center") => Ok(5),
        ("center", "right") | ("middle", "right") => Ok(6),
        ("top", "left") => Ok(7),
        ("top", "center") => Ok(8),
        ("top", "right") => Ok(9),
        _ => bail!("Invalid alignment combination: vertical='{}', horizontal='{}'. Use 'top/center/bottom' and 'left/center/right'.", vertical, horizontal),
    }
}


pub fn burn_subtitles(
    input_video_path: &str,
    subtitle_file_path: &str,
    output_video_path: &str,
    font_path: &str,
    font_size: u32,
    font_color: &str,
    vertical_alignment: &str, // e.g., "bottom", "center", "top"
    horizontal_alignment: &str, // e.g., "center", "left", "right"
) -> Result<(), Error> {
    ensure_ffmpeg_initialized();

    let mut ictx = ffmpeg::format::input(&Path::new(input_video_path))?;
    
    let mut opts = ffmpeg::Dictionary::new();

    // --- Subtitle filter configuration ---
    let escaped_subtitle_path = escape_path_for_ffmpeg_filter(subtitle_file_path);
    let escaped_font_path = escape_path_for_ffmpeg_filter(font_path);
    
    // FontName for FFmpeg's force_style can be tricky.
    // Often, it's the font's actual name, not the file path.
    // However, some FFmpeg builds/platforms might accept the (escaped) path directly with `force_style`.
    // For subtitles filter, `Fontfile=<path>` is a more robust way if available with `force_style`.
    // Let's try referencing the font by its escaped path in `FontFile` if possible, or `FontName` if not.
    // The `subtitles` filter syntax is `subtitles=filename='<file>':force_style='FontName=<name>,FontSize=<size>,...'`
    // Or with `Fontfile`: `subtitles=filename='<file>':force_style='Fontfile=<font_file_path>,FontSize=<size>,...'`
    // Font names can be tricky. Using `Fontfile` is generally more robust if the FFmpeg build supports it within `force_style`.
    // Let's assume for now we try to use the font path directly as FontName, or try Fontfile.
    // A simpler approach is to hope that fontconfig is set up and font name is enough.
    // For maximum robustness, providing an escaped path to `Fontfile` is best.
    // let font_name_or_path = Path::new(font_path) // This variable was unused, Fontfile is used directly.
    //     .file_name()
    //     .and_then(|s| s.to_str())
    //     .unwrap_or("Arial"); // Fallback font name

    let ffmpeg_color = convert_color_to_ffmpeg_bgr(font_color)?;
    let ffmpeg_alignment = map_alignment_to_ffmpeg_value(vertical_alignment, horizontal_alignment)?;

    // Construct force_style string for SRT
    // PrimaryColour format is &HAABBGGRR (Alpha, Blue, Green, Red)
    // We'll use opaque (FF for alpha). So &HFFBBGGRR
    let primary_colour_bgr = ffmpeg_color.trim_start_matches("&H");
    let force_style = format!(
        "Fontfile='{}',FontSize={},PrimaryColour=&HFF{},Alignment={}",
        escaped_font_path, // Using Fontfile with escaped path
        font_size,
        primary_colour_bgr, // Assuming ffmpeg_color is &HBBGGRR, so FF + BBGGRR
        ffmpeg_alignment
    );
    
    let filter_string = format!(
        "subtitles=filename='{}':force_style='{}'",
        escaped_subtitle_path,
        force_style
    );

    opts.set("vf", &filter_string);
    opts.set("c:v", "libx264"); // Re-encode video
    opts.set("c:a", "copy");    // Copy audio

    let mut octx = ffmpeg::format::output_with(&Path::new(output_video_path), opts)?;

    // Stream mapping and parameter copying
    for ist_stream in ictx.streams() {
        let ist_params = ist_stream.parameters();
        let mut ost_stream = octx.add_stream(None)?; 
        
        // For both video (being re-encoded with filter) and audio (being copied),
        // copying original parameters is a good starting point. 
        // FFmpeg will adjust video parameters as needed based on libx264 and filter requirements.
        ost_stream.set_parameters(ist_params.clone());
        // Explicit tag setting removed as it was causing errors and set_parameters should handle it.
    }
    
    octx.set_metadata(ictx.metadata().to_owned());
    octx.write_header()?;

    // Transcoding/Filtering loop (simplified for -vf and -c:a copy)
    // When -vf is used with re-encoding, and -c:a copy, ffmpeg handles the complexities.
    // We just need to feed all packets.
    for (stream, mut packet) in ictx.packets() {
        let ist_idx = stream.index();
        if octx.stream(ist_idx).is_some() { // If this stream is part of our output
            packet.set_stream(ist_idx); // Map to the same stream index in output
            match packet.write_interleaved(&mut octx) {
                Ok(_) => (),
                Err(e) => eprintln!("Failed to write packet: {}", e), // Log and continue or break
            }
        }
    }

    octx.write_trailer()?;
    Ok(())
}


pub fn extract_audio(input_path: &str, audio_output_path: &str) -> Result<(), Error> {
    ensure_ffmpeg_initialized();

    let mut ictx = ffmpeg::format::input(&Path::new(input_path))?;
    
    let mut opts = ffmpeg::Dictionary::new();
    opts.set("vn", "1"); // set returns (), no ?
    opts.set("acodec", "pcm_s16le"); // WAV codec // set returns (), no ?
    // Optionally set sample rate and channels if needed
    // opts.set("ar", "44100");
    // opts.set("ac", "2");

    let mut octx = ffmpeg::format::output_with(&Path::new(audio_output_path), opts)?;

    let best_audio_stream_index = ictx
        .streams()
        .best(ffmpeg::media::Type::Audio)
        .ok_or_else(|| Error::msg("No audio stream found in input"))?
        .index();

    let ist_audio = ictx.stream(best_audio_stream_index)
        .ok_or_else(|| Error::msg("Could not retrieve input audio stream"))?;
    let ist_audio_params = ist_audio.parameters();
    
    let mut ost_audio = octx.add_stream(None)?; 
    ost_audio.set_parameters(ist_audio_params.clone());
    // Codec tag for pcm_s16le is usually not needed or handled by format.
    // Explicit tag setting removed.

    octx.set_metadata(ictx.metadata().to_owned());
    octx.write_header()?; // Options should be applied by the context.

    for (stream, mut packet) in ictx.packets() {
        if stream.index() == best_audio_stream_index {
            packet.set_stream(0); // Output stream index for the single audio stream will be 0
            match packet.write_interleaved(&mut octx) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Failed to write audio packet: {}", e);
                }
            }
        }
    }

    octx.write_trailer()?;
    Ok(())
}
