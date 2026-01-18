//! Self-upgrade command for the skill CLI
//!
//! Downloads and installs the latest version of the skill binary
//! from Vercel Blob storage.

use anyhow::{Context, Result};
use colored::*;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

const BLOB_BASE_URL: &str = "https://dqkbk9o7ynwhxfjx.public.blob.vercel-storage.com";
const VERSION_URL: &str = "https://dqkbk9o7ynwhxfjx.public.blob.vercel-storage.com/latest-version.txt";

/// Get the current version from Cargo.toml
fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Get the target triple for the current platform
fn get_target() -> Result<&'static str> {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return Ok("aarch64-apple-darwin");

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    return Ok("x86_64-apple-darwin");

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return Ok("x86_64-unknown-linux-gnu");

    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    return Ok("aarch64-unknown-linux-gnu");

    #[cfg(not(any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "aarch64"),
    )))]
    anyhow::bail!("Unsupported platform. Please build from source.");
}

/// Fetch the latest version from Vercel Blob
async fn get_latest_version() -> Result<String> {
    let client = reqwest::Client::new();
    let response = client
        .get(VERSION_URL)
        .send()
        .await
        .context("Failed to fetch latest version")?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Failed to fetch version info: {}",
            response.status()
        );
    }

    let version = response
        .text()
        .await
        .context("Failed to read version")?
        .trim()
        .to_string();

    Ok(version)
}

/// Download the binary for the current platform
async fn download_binary(version: &str, target: &str) -> Result<Vec<u8>> {
    let version_num = version.strip_prefix('v').unwrap_or(version);
    let url = format!(
        "{}/skill-{}-{}.tar.gz",
        BLOB_BASE_URL, version_num, target
    );

    eprintln!("  {} {}", "Downloading:".dimmed(), url);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to download binary")?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Download failed: {} - {}",
            response.status(),
            url
        );
    }

    response
        .bytes()
        .await
        .map(|b| b.to_vec())
        .context("Failed to read binary data")
}

/// Extract the binary from the tarball
fn extract_binary(tarball: &[u8]) -> Result<Vec<u8>> {
    use flate2::read::GzDecoder;
    use tar::Archive;
    use std::io::Read;

    let decoder = GzDecoder::new(tarball);
    let mut archive = Archive::new(decoder);

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;

        // Look for the skill binary
        if path.file_name().map(|n| n == "skill").unwrap_or(false) {
            let mut contents = Vec::new();
            entry.read_to_end(&mut contents)?;
            return Ok(contents);
        }
    }

    anyhow::bail!("Binary 'skill' not found in archive")
}

/// Get the path to the current executable
fn get_current_exe() -> Result<PathBuf> {
    env::current_exe().context("Failed to get current executable path")
}

/// Replace the current binary with the new one
fn replace_binary(new_binary: &[u8], exe_path: &PathBuf) -> Result<()> {
    // Create a temporary file in the same directory
    let temp_path = exe_path.with_extension("new");
    let backup_path = exe_path.with_extension("backup");

    // Write new binary to temp file
    fs::write(&temp_path, new_binary)
        .context("Failed to write new binary")?;

    // Make it executable
    let mut perms = fs::metadata(&temp_path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&temp_path, perms)?;

    // Backup current binary
    if exe_path.exists() {
        fs::rename(exe_path, &backup_path)
            .context("Failed to backup current binary")?;
    }

    // Move new binary into place
    if let Err(e) = fs::rename(&temp_path, exe_path) {
        // Try to restore backup
        if backup_path.exists() {
            let _ = fs::rename(&backup_path, exe_path);
        }
        return Err(e).context("Failed to install new binary");
    }

    // Remove backup
    let _ = fs::remove_file(&backup_path);

    Ok(())
}

/// Compare versions (simple semver comparison)
fn is_newer_version(current: &str, latest: &str) -> bool {
    let current = current.strip_prefix('v').unwrap_or(current);
    let latest = latest.strip_prefix('v').unwrap_or(latest);

    let parse_version = |v: &str| -> (u32, u32, u32) {
        let parts: Vec<u32> = v
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        (
            parts.first().copied().unwrap_or(0),
            parts.get(1).copied().unwrap_or(0),
            parts.get(2).copied().unwrap_or(0),
        )
    };

    parse_version(latest) > parse_version(current)
}

/// Execute the upgrade command
pub async fn execute(force: bool, check_only: bool) -> Result<()> {
    let current = current_version();

    println!();
    println!("{}", "Skill Engine Upgrade".bold());
    println!("{}", "─".repeat(50));
    println!();
    println!("  {} {}", "Current version:".dimmed(), format!("v{}", current).cyan());

    // Fetch latest version from Vercel Blob
    print!("  {} ", "Checking for updates...".dimmed());
    let latest = get_latest_version().await?;
    println!("{}", "done".green());

    println!("  {} {}", "Latest version:".dimmed(), format!("v{}", latest).cyan());
    println!();

    // Check if upgrade is needed
    let needs_upgrade = is_newer_version(current, &latest);

    if !needs_upgrade && !force {
        println!(
            "{} You're already running the latest version!",
            "✓".green().bold()
        );
        println!();
        return Ok(());
    }

    if check_only {
        if needs_upgrade {
            println!(
                "{} A new version is available: {} → {}",
                "ℹ".blue().bold(),
                format!("v{}", current).yellow(),
                format!("v{}", latest).green()
            );
            println!();
            println!("Run {} to upgrade.", "skill upgrade".cyan());
        }
        return Ok(());
    }

    if !needs_upgrade && force {
        println!(
            "{} Forcing reinstall of version {}",
            "⚠".yellow().bold(),
            latest
        );
    } else {
        println!(
            "{} Upgrading: {} → {}",
            "→".blue().bold(),
            format!("v{}", current).yellow(),
            format!("v{}", latest).green()
        );
    }
    println!();

    // Get target platform
    let target = get_target()?;
    println!("  {} {}", "Platform:".dimmed(), target);

    // Download binary
    println!("  {} downloading...", "Status:".dimmed());
    let tarball = download_binary(&latest, target).await?;
    println!("  {} extracting...", "Status:".dimmed());

    // Extract binary
    let binary = extract_binary(&tarball)?;
    println!(
        "  {} {} bytes",
        "Binary size:".dimmed(),
        binary.len().to_string().cyan()
    );

    // Get current exe path
    let exe_path = get_current_exe()?;
    println!("  {} {}", "Install path:".dimmed(), exe_path.display());

    // Replace binary
    println!("  {} installing...", "Status:".dimmed());
    replace_binary(&binary, &exe_path)?;

    println!();
    println!(
        "{} Successfully upgraded to v{}!",
        "✓".green().bold(),
        latest.green()
    );
    println!();

    Ok(())
}

/// Check for updates without installing
#[allow(dead_code)]
pub async fn check() -> Result<()> {
    execute(false, true).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(is_newer_version("0.1.0", "0.2.0"));
        assert!(is_newer_version("0.1.0", "v0.2.0"));
        assert!(is_newer_version("v0.1.0", "0.2.0"));
        assert!(is_newer_version("0.1.9", "0.2.0"));
        assert!(is_newer_version("0.2.0", "1.0.0"));
        assert!(!is_newer_version("0.2.0", "0.2.0"));
        assert!(!is_newer_version("0.2.0", "0.1.0"));
        assert!(!is_newer_version("1.0.0", "0.9.9"));
    }
}
