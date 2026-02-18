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
    
    // Check if ./polymarket exists and get its version
    let target_binary = std::path::Path::new("polymarket");
    
    if target_binary.exists() {
        match get_binary_version(target_binary) {
            Ok(current_version) => {
                if current_version == target_version {
                    println!("Already up to date (version {})", current_version);
                    return Ok(());
                }
                println!("New version available: {} (current: {})", tag_name, current_version);
            },
            Err(e) => {
                println!("Could not determine version of ./polymarket: {}", e);
                println!("Proceeding with update to {}", tag_name);
            }
        }
    } else {
        println!("Installing {} (version {})", target_binary.display(), tag_name);
    }

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

    // Create temp file in current directory
    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let mut temp_file = tempfile::Builder::new()
        .prefix(".polymarket-update")
        .suffix(".tmp")
        .tempfile_in(&current_dir)
        .context("Failed to create temp file in current directory")?;
        
    // Stream download to file
    while let Some(chunk) = download_resp.chunk().await? {
        temp_file.write_all(&chunk).context("Failed to write to temp file")?;
    }
    
    // Set executable permissions
    let mut perms = temp_file.as_file().metadata()?.permissions();
    perms.set_mode(0o755);
    temp_file.as_file().set_permissions(perms)?;

    // Rename temp file to ./polymarket
    // Using persist to atomically replace
    match temp_file.persist(target_binary) {
        Ok(_) => println!("Successfully updated ./polymarket to {}!", tag_name),
        Err(e) => bail!("Failed to replace binary: {}", e.error),
    }
    
    Ok(())
}

fn get_binary_version(path: &std::path::Path) -> Result<String> {
    let output = std::process::Command::new(path)
        .arg("--version")
        .output()
        .context("Failed to execute binary")?;
        
    if !output.status.success() {
        bail!("Binary returned non-zero exit code");
    }
    
    let stdout = String::from_utf8(output.stdout).context("Invalid UTF-8 in output")?;
    // Expected output format: "polymarket-cli v0.9.0" or similar
    // We split by whitespace and take the last part
    let version_part = stdout.trim().split_whitespace().last()
        .context("Empty version output")?;
        
    Ok(version_part.trim_start_matches('v').to_string())
}
