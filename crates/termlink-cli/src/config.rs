use anyhow::Result;

/// Resolved hub connection parameters from profile or CLI args.
pub(crate) struct HubProfile {
    pub address: String,
    pub secret_file: Option<String>,
    pub secret: Option<String>,
    pub scope: Option<String>,
}

/// Hub profiles config file (~/.termlink/hubs.toml)
#[derive(serde::Deserialize, serde::Serialize, Default)]
pub(crate) struct HubsConfig {
    #[serde(default)]
    pub hubs: std::collections::HashMap<String, HubEntry>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub(crate) struct HubEntry {
    pub address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

pub(crate) fn hubs_config_path() -> std::path::PathBuf {
    termlink_config_dir().join("hubs.toml")
}

pub(crate) fn termlink_config_dir() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    std::path::PathBuf::from(home).join(".termlink")
}

pub(crate) fn load_hubs_config() -> HubsConfig {
    let path = hubs_config_path();
    if let Ok(content) = std::fs::read_to_string(&path) {
        toml::from_str(&content).unwrap_or_default()
    } else {
        HubsConfig::default()
    }
}

pub(crate) fn save_hubs_config(config: &HubsConfig) -> Result<()> {
    let path = hubs_config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(config)?;
    std::fs::write(&path, content)?;
    Ok(())
}

/// Resolve hub argument: if it contains `:`, treat as address.
/// Otherwise look up as a profile name in ~/.termlink/hubs.toml.
/// CLI-provided secret_file/secret/scope override profile defaults.
pub(crate) fn resolve_hub_profile(
    hub_arg: &str,
    cli_secret_file: Option<&str>,
    cli_secret: Option<&str>,
    cli_scope: &str,
) -> Result<HubProfile> {
    if hub_arg.contains(':') {
        // Direct address
        return Ok(HubProfile {
            address: hub_arg.to_string(),
            secret_file: cli_secret_file.map(String::from),
            secret: cli_secret.map(String::from),
            scope: Some(cli_scope.to_string()),
        });
    }

    // Look up profile
    let config = load_hubs_config();
    let entry = config.hubs.get(hub_arg)
        .ok_or_else(|| anyhow::anyhow!(
            "Hub profile '{}' not found. Use host:port or add a profile:\n  termlink remote profile add {} <address> --secret-file <path>",
            hub_arg, hub_arg
        ))?;

    Ok(HubProfile {
        address: entry.address.clone(),
        secret_file: cli_secret_file.map(String::from).or_else(|| entry.secret_file.clone()),
        secret: cli_secret.map(String::from).or_else(|| entry.secret.clone()),
        scope: Some(cli_scope.to_string()).or_else(|| entry.scope.clone()),
    })
}
