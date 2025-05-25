{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  # Name for the shell environment (optional, but good for prompts)
  name = "shorts-generator-dev";

  # Packages to make available in the shell
  buildInputs = [
    pkgs.ffmpeg
    pkgs.rustc
    pkgs.cargo
    pkgs.pkg-config
    pkgs.clang_17 # Using a recent stable version of clang
    pkgs.openssl.dev
    pkgs.libclang.lib # For ffmpeg-next and bindgen

    # TODO: User may need to add their preferred whisper.cpp package or a custom derivation here.
    # No specific 'whisper.cpp' package was easily identifiable through automated search.
    # Common names might include 'whisper-cpp', 'whispercpp', or variants with GPU support
    # (e.g., 'whisper-cpp-cuda'). Please check the Nixpkgs repository for the latest available options.
    # For example, if whisper.cpp is available as 'pkgs.whisper-cpp-with-gpu':
    # pkgs.whisper-cpp-with-gpu

    # Placeholder for whisper CLI if no specific package is added by the user.
    # This ensures that a 'whisper' command exists, though it will only print a warning.
    (pkgs.runCommand "whisper-placeholder" {} ''
      mkdir -p $out/bin
      echo 'echo "WARN: Real whisper executable not configured in shell.nix. Please add a whisper.cpp package."' > $out/bin/whisper
      chmod +x $out/bin/whisper
    '')
  ];

  # Optional: Commands to run when the shell is entered
  shellHook = ''
    echo "Entered shorts-generator development environment."
    echo "FFmpeg, Rust, Cargo, Clang, and other dependencies are available."
    echo "Whisper CLI is currently a placeholder. You may need to add a specific whisper.cpp package to your shell.nix."
    # export RUST_SRC_PATH="${pkgs.rustPlatform.rustLibSrc}"; # For rust-analyzer if needed
  '';
}
