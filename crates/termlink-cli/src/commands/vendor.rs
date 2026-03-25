use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

const VENDOR_DIR: &str = ".termlink";
const VENDOR_BIN: &str = ".termlink/bin/termlink";
const VENDOR_VERSION: &str = ".termlink/VERSION";

/// Vendor the TermLink binary into a project directory for path isolation.
///
/// Same pattern as the Agentic Engineering Framework's `.agentic-framework/`:
/// each project gets its own copy of the binary, decoupled from the global install.
pub(crate) fn cmd_vendor(
    source: Option<&str>,
    target: Option<&str>,
    dry_run: bool,
) -> Result<()> {
    // Resolve source binary
    let source_path = if let Some(s) = source {
        PathBuf::from(s)
    } else {
        std::env::current_exe()
            .context("Cannot determine current binary path")?
    };

    if !source_path.exists() {
        anyhow::bail!("Source binary not found: {}", source_path.display());
    }

    // Resolve target project directory
    let project_dir = if let Some(t) = target {
        PathBuf::from(t)
    } else {
        std::env::current_dir()
            .context("Cannot determine current directory")?
    };

    let dest_bin = project_dir.join(VENDOR_BIN);
    let dest_version = project_dir.join(VENDOR_VERSION);
    let dest_dir = project_dir.join(VENDOR_DIR);

    // Get source binary metadata
    let source_meta = std::fs::metadata(&source_path)
        .context("Cannot read source binary metadata")?;
    let source_size = source_meta.len();

    // Get version from running the binary
    let source_version = get_binary_version(&source_path);

    // Check if already vendored
    let existing_version = std::fs::read_to_string(&dest_version)
        .ok()
        .map(|v| v.trim().to_string());

    if dry_run {
        println!("termlink vendor --dry-run");
        println!("  Source:  {} ({:.1} MB)", source_path.display(), source_size as f64 / 1_048_576.0);
        if let Some(ref v) = source_version {
            println!("  Version: {v}");
        }
        println!("  Target:  {}", dest_bin.display());
        if let Some(ref v) = existing_version {
            println!("  Current: {v} (will be overwritten)");
        } else {
            println!("  Current: (not vendored)");
        }
        println!("\nWould copy binary and write VERSION file.");
        return Ok(());
    }

    // Create directory structure
    let bin_dir = project_dir.join(".termlink/bin");
    std::fs::create_dir_all(&bin_dir)
        .context(format!("Cannot create {}", bin_dir.display()))?;

    // Copy binary
    std::fs::copy(&source_path, &dest_bin)
        .context(format!("Cannot copy binary to {}", dest_bin.display()))?;

    // Set executable permission
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dest_bin, std::fs::Permissions::from_mode(0o755))
            .context("Cannot set executable permission")?;
    }

    // Write VERSION file
    if let Some(ref v) = source_version {
        std::fs::write(&dest_version, format!("{v}\n"))
            .context("Cannot write VERSION file")?;
    }

    // Report
    let action = if existing_version.is_some() { "Updated" } else { "Vendored" };
    println!("{action} TermLink binary into project");
    println!("  Source:  {} ({:.1} MB)", source_path.display(), source_size as f64 / 1_048_576.0);
    if let Some(ref v) = source_version {
        println!("  Version: {v}");
    }
    if let Some(ref v) = existing_version {
        println!("  Previous: {v}");
    }
    println!("  Binary:  {}", dest_bin.display());
    println!("\nProject scripts should use: {VENDOR_BIN}");

    // Check if .gitignore has the vendor binary
    check_gitignore(&project_dir, &dest_dir);

    Ok(())
}

/// Check TermLink vendor status for a project directory.
pub(crate) fn cmd_vendor_status(target: Option<&str>) -> Result<()> {
    let project_dir = if let Some(t) = target {
        PathBuf::from(t)
    } else {
        std::env::current_dir()?
    };

    let dest_bin = project_dir.join(VENDOR_BIN);
    let dest_version = project_dir.join(VENDOR_VERSION);

    if !dest_bin.exists() {
        println!("Not vendored. Run: termlink vendor");
        return Ok(());
    }

    let version = std::fs::read_to_string(&dest_version)
        .ok()
        .map(|v| v.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let meta = std::fs::metadata(&dest_bin)?;
    let size = meta.len();

    // Compare with current binary
    let current_exe = std::env::current_exe().ok();
    let current_version = current_exe.as_ref().and_then(|p| get_binary_version(p));

    println!("TermLink vendor status");
    println!("  Binary:  {} ({:.1} MB)", dest_bin.display(), size as f64 / 1_048_576.0);
    println!("  Version: {version}");

    if let Some(ref cv) = current_version {
        if *cv != version {
            println!("  Global:  {cv} (DIFFERS — run: termlink vendor)");
        } else {
            println!("  Global:  {cv} (matches)");
        }
    }

    Ok(())
}

/// Try to get version string from a termlink binary.
fn get_binary_version(path: &Path) -> Option<String> {
    std::process::Command::new(path)
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| {
            String::from_utf8(o.stdout).ok().map(|s| {
                s.trim()
                    .strip_prefix("termlink ")
                    .unwrap_or(s.trim())
                    .to_string()
            })
        })
}

/// Warn if .gitignore doesn't exclude the vendored binary.
fn check_gitignore(project_dir: &Path, vendor_dir: &Path) {
    let gitignore = project_dir.join(".gitignore");
    if let Ok(content) = std::fs::read_to_string(&gitignore) {
        let vendor_rel = vendor_dir
            .strip_prefix(project_dir)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| VENDOR_DIR.to_string());

        if !content.contains(&vendor_rel) && !content.contains(".termlink/bin") {
            println!("\nWARN: .gitignore does not exclude vendored binary.");
            println!("  Add: .termlink/bin/");
        }
    }
}
