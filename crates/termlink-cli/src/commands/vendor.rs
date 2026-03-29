use anyhow::Result;
use serde_json::Value;
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
    json: bool,
) -> Result<()> {
    // Resolve source binary
    let source_path = if let Some(s) = source {
        PathBuf::from(s)
    } else {
        match std::env::current_exe() {
            Ok(p) => p,
            Err(e) => {
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Cannot determine current binary path: {}", e)}));
                }
                return Err(e.into());
            }
        }
    };

    if !source_path.exists() {
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Source binary not found: {}", source_path.display())}));
        }
        anyhow::bail!("Source binary not found: {}", source_path.display());
    }

    // Resolve target project directory
    let project_dir = if let Some(t) = target {
        PathBuf::from(t)
    } else {
        match std::env::current_dir() {
            Ok(p) => p,
            Err(e) => {
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Cannot determine current directory: {}", e)}));
                }
                return Err(e.into());
            }
        }
    };

    let dest_bin = project_dir.join(VENDOR_BIN);
    let dest_version = project_dir.join(VENDOR_VERSION);
    let dest_dir = project_dir.join(VENDOR_DIR);

    // Get source binary metadata
    let source_meta = match std::fs::metadata(&source_path) {
        Ok(m) => m,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Cannot read source binary metadata: {}", e)}));
            }
            return Err(e.into());
        }
    };
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
        println!("Would configure MCP server in .claude/settings.local.json");
        return Ok(());
    }

    // Create directory structure
    let bin_dir = project_dir.join(".termlink/bin");
    if let Err(e) = std::fs::create_dir_all(&bin_dir) {
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Cannot create {}: {}", bin_dir.display(), e)}));
        }
        return Err(e.into());
    }

    // Copy binary via atomic rename to avoid ETXTBSY (text file busy) when
    // the destination binary is currently running (e.g., as an MCP server).
    // Pattern: copy to temp file, set permissions, then rename over destination.
    let temp_bin = dest_bin.with_extension("new");
    if let Err(e) = std::fs::copy(&source_path, &temp_bin) {
        let _ = std::fs::remove_file(&temp_bin);
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Cannot copy binary to {}: {}", temp_bin.display(), e)}));
        }
        return Err(e.into());
    }

    // Set executable permission on temp file before rename
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Err(e) = std::fs::set_permissions(&temp_bin, std::fs::Permissions::from_mode(0o755)) {
            let _ = std::fs::remove_file(&temp_bin);
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Cannot set executable permission: {}", e)}));
            }
            return Err(e.into());
        }
    }

    // Atomic rename — works even if dest_bin is running (replaces inode reference)
    if let Err(e) = std::fs::rename(&temp_bin, &dest_bin) {
        let _ = std::fs::remove_file(&temp_bin);
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Cannot rename binary to {}: {}", dest_bin.display(), e)}));
        }
        return Err(e.into());
    }

    // Write VERSION file
    if let Some(ref v) = source_version
        && let Err(e) = std::fs::write(&dest_version, format!("{v}\n"))
    {
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Cannot write VERSION file: {}", e)}));
        }
        return Err(e.into());
    }

    // Check if .gitignore has the vendor binary
    check_gitignore(&project_dir, &dest_dir);

    // Configure MCP server in Claude Code settings
    configure_mcp(&project_dir);

    // Report
    if json {
        println!("{}", serde_json::json!({
            "ok": true,
            "action": if existing_version.is_some() { "updated" } else { "vendored" },
            "source": source_path.display().to_string(),
            "binary": dest_bin.display().to_string(),
            "version": source_version,
            "previous_version": existing_version,
            "size_bytes": source_size,
        }));
    } else {
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
    }

    Ok(())
}

/// Check TermLink vendor status for a project directory.
pub(crate) fn cmd_vendor_status(target: Option<&str>, json: bool, check: bool) -> Result<()> {
    let project_dir = if let Some(t) = target {
        PathBuf::from(t)
    } else {
        std::env::current_dir()?
    };

    let dest_bin = project_dir.join(VENDOR_BIN);
    let dest_version = project_dir.join(VENDOR_VERSION);

    if !dest_bin.exists() {
        if json {
            let mut obj = serde_json::json!({"ok": true, "vendored": false});
            if check {
                obj["needs_update"] = serde_json::json!(true);
            }
            println!("{}", obj);
        } else {
            println!("Not vendored. Run: termlink vendor");
        }
        if check {
            use std::io::Write;
            let _ = std::io::stdout().flush();
            std::process::exit(1);
        }
        return Ok(());
    }

    let version = std::fs::read_to_string(&dest_version)
        .ok()
        .map(|v| v.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let meta = match std::fs::metadata(&dest_bin) {
        Ok(m) => m,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Cannot read vendor binary metadata: {e}")}));
            }
            anyhow::bail!("Cannot read vendor binary metadata: {e}");
        }
    };
    let size = meta.len();

    // Compare with current binary
    let current_exe = std::env::current_exe().ok();
    let current_version = current_exe.as_ref().and_then(|p| get_binary_version(p));

    // Check MCP configuration
    let settings_path = project_dir.join(".claude/settings.local.json");
    let mcp_configured = settings_path.exists()
        && std::fs::read_to_string(&settings_path)
            .ok()
            .and_then(|c| serde_json::from_str::<Value>(&c).ok())
            .and_then(|v| v.get("mcpServers")?.get("termlink").cloned())
            .is_some();

    // Check .gitignore
    let gitignore = project_dir.join(".gitignore");
    let gi_ok = std::fs::read_to_string(&gitignore)
        .ok()
        .map(|c| c.contains(".termlink"))
        .unwrap_or(false);

    let version_matches = current_version.as_ref().map(|cv| *cv == version).unwrap_or(false);
    let needs_update = !version_matches || !mcp_configured || !gi_ok;

    if json {
        let mut obj = serde_json::json!({
            "ok": true,
            "vendored": true,
            "binary": dest_bin.display().to_string(),
            "version": version,
            "size_bytes": size,
            "global_version": current_version,
            "version_matches": version_matches,
            "mcp_configured": mcp_configured,
            "gitignore_ok": gi_ok,
        });
        if check {
            obj["needs_update"] = serde_json::json!(needs_update);
        }
        println!("{}", obj);
    } else {
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

        if mcp_configured {
            println!("  MCP:     configured in .claude/settings.local.json");
        } else {
            println!("  MCP:     NOT configured (run: termlink vendor)");
        }

        if gi_ok {
            println!("  Ignore:  .termlink in .gitignore");
        } else {
            println!("  Ignore:  NOT in .gitignore (run: termlink vendor)");
        }
    }

    if check && needs_update {
        use std::io::Write;
        let _ = std::io::stdout().flush();
        std::process::exit(1);
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

/// Ensure .gitignore excludes the vendored binary. Creates or appends as needed.
fn check_gitignore(project_dir: &Path, vendor_dir: &Path) {
    let gitignore = project_dir.join(".gitignore");
    let vendor_rel = vendor_dir
        .strip_prefix(project_dir)
        .map(|p| format!("{}/", p.display()))
        .unwrap_or_else(|_| format!("{VENDOR_DIR}/"));

    let content = std::fs::read_to_string(&gitignore).unwrap_or_default();

    if content.contains(&vendor_rel) || content.contains(".termlink/bin") {
        return;
    }

    // Append entry (create file if needed)
    let entry = if content.is_empty() || content.ends_with('\n') {
        format!("{vendor_rel}\n")
    } else {
        format!("\n{vendor_rel}\n")
    };

    match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&gitignore)
    {
        Ok(mut f) => {
            use std::io::Write;
            let _ = f.write_all(entry.as_bytes());
            println!("\n.gitignore: added {vendor_rel}");
        }
        Err(e) => {
            println!("\nWARN: Cannot update .gitignore: {e}");
            println!("  Add manually: {vendor_rel}");
        }
    }
}

/// Configure TermLink MCP server in `.claude/settings.local.json`.
///
/// Merges the termlink MCP entry into existing settings, preserving all other content.
fn configure_mcp(project_dir: &Path) {
    let claude_dir = project_dir.join(".claude");
    let settings_path = claude_dir.join("settings.local.json");

    // Read existing settings or start with empty object
    let mut settings: Value = if settings_path.exists() {
        match std::fs::read_to_string(&settings_path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(v) => v,
                Err(e) => {
                    println!("\nWARN: Cannot parse {}: {e}", settings_path.display());
                    println!("  MCP server not configured. Add manually to .claude/settings.local.json");
                    return;
                }
            },
            Err(e) => {
                println!("\nWARN: Cannot read {}: {e}", settings_path.display());
                return;
            }
        }
    } else {
        serde_json::json!({})
    };

    // Build the expected MCP entry
    let expected = serde_json::json!({
        "command": ".termlink/bin/termlink",
        "args": ["mcp", "serve"]
    });

    // Check if already configured correctly
    let already_configured = settings
        .get("mcpServers")
        .and_then(|s| s.get("termlink"))
        == Some(&expected);

    if already_configured {
        println!("\nMCP server: already configured in .claude/settings.local.json");
        return;
    }

    // Merge the entry
    let mcp_servers = settings
        .as_object_mut()
        .expect("settings is an object")
        .entry("mcpServers")
        .or_insert_with(|| serde_json::json!({}));

    mcp_servers
        .as_object_mut()
        .expect("mcpServers is an object")
        .insert("termlink".to_string(), expected);

    // Ensure .claude/ directory exists
    if let Err(e) = std::fs::create_dir_all(&claude_dir) {
        println!("\nWARN: Cannot create {}: {e}", claude_dir.display());
        return;
    }

    // Write back with pretty formatting
    match serde_json::to_string_pretty(&settings) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&settings_path, format!("{json}\n")) {
                println!("\nWARN: Cannot write {}: {e}", settings_path.display());
            } else {
                println!("\nMCP server: configured in .claude/settings.local.json");
                println!("  Claude Code will load TermLink tools on next session start.");
            }
        }
        Err(e) => {
            println!("\nWARN: Cannot serialize settings: {e}");
        }
    }
}
