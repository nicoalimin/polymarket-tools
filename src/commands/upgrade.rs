use anyhow::{Context, Result, bail};
use std::env;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use reqwest;

pub async fn execute() -> Result<()> {
    println!("Checking for updates...");

    let (os, arch) = (env::consts::OS, env::consts::ARCH);
    let asset_suffix = match (os, arch) {
        ("linux", "x86_64") => "linux-amd64",
        ("macos", "x86_64") => "macos-amd64",
        ("macos", "aarch64") => "macos-arm64",
        _ => bail!("Unsupported platform: {} {}", os, arch),
    };

    let client = reqwest::Client::new();
    let release_url = "https://api.github.com/repos/nicoalimin/polymarket-tools/releases/latest";
    
    let resp = client.get(release_url)
        .header("User-Agent", "polymarket-cli")
        .send()
        .await
        .context("Failed to check for updates")?;

    if !resp.status().is_success() {
        bail!("Failed to get latest release: {}", resp.status());
    }

    let json: serde_json::Value = resp.json().await.context("Failed to parse release JSON")?;
    let tag_name = json["tag_name"].as_str().context("No tag_name in release")?;
    let target_version = tag_name.trim_start_matches('v');
    
    // Use the current executable's path as the target binary to replace.
    // This is more reliable than a hardcoded relative path, which can fail
    // if the working directory differs from the binary's location.
    let current_exe = env::current_exe().context("Failed to determine current executable path")?;
    let target_binary = current_exe.canonicalize().unwrap_or(current_exe);
    
    // Get the current version from the compiled-in version string
    // instead of spawning a subprocess (which can fail on macOS due to code signing,
    // SIP, or the binary being the same running process).
    let current_version = get_compiled_version();
    
    if current_version == target_version {
        println!("Already up to date (version {})", current_version);
        return Ok(());
    }
    println!("New version available: {} (current: {})", tag_name, current_version);

    let assets = json["assets"].as_array().context("No assets in release")?;
    let asset = assets.iter()
        .find(|a| a["name"].as_str().unwrap_or("").ends_with(asset_suffix))
        .context(format!("No asset found for platform suffix: {}", asset_suffix))?;

    let download_url = asset["browser_download_url"].as_str().context("No download URL")?;
    println!("Downloading from: {}", download_url);

    let mut download_resp = client.get(download_url)
        .header("User-Agent", "polymarket-cli")
        .send()
        .await
        .context("Failed to download update")?;

    if !download_resp.status().is_success() {
        bail!("Failed to download update: {}", download_resp.status());
    }

    // Create temp file in the same directory as the target binary
    let target_dir = target_binary.parent().context("Failed to get target binary directory")?;
    let mut temp_file = tempfile::Builder::new()
        .prefix(".polymarket-update")
        .suffix(".tmp")
        .tempfile_in(target_dir)
        .context("Failed to create temp file")?;
        
    // Stream download to file
    while let Some(chunk) = download_resp.chunk().await? {
        temp_file.write_all(&chunk).context("Failed to write to temp file")?;
    }
    
    // Set executable permissions
    let mut perms = temp_file.as_file().metadata()?.permissions();
    perms.set_mode(0o755);
    temp_file.as_file().set_permissions(perms)?;

    // Rename temp file to replace the current binary
    // Using persist to atomically replace
    match temp_file.persist(&target_binary) {
        Ok(_) => println!("Successfully updated {} to {}!", target_binary.display(), tag_name),
        Err(e) => bail!("Failed to replace binary: {}", e.error),
    }
    
    Ok(())
}

/// Get the version that was compiled into this binary from version.txt
fn get_compiled_version() -> &'static str {
    include_str!("../../version.txt").trim()
        .trim_start_matches('v')
}
