use anyhow::Result;

/// Resolved hub connection parameters from profile or CLI args.
#[derive(Debug)]
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
    resolve_hub_profile_with_config(hub_arg, cli_secret_file, cli_secret, cli_scope, &load_hubs_config())
}

fn resolve_hub_profile_with_config(
    hub_arg: &str,
    cli_secret_file: Option<&str>,
    cli_secret: Option<&str>,
    cli_scope: &str,
    config: &HubsConfig,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_direct_address() {
        let p = resolve_hub_profile("192.168.1.1:9100", None, None, "observe").unwrap();
        assert_eq!(p.address, "192.168.1.1:9100");
        assert_eq!(p.scope.as_deref(), Some("observe"));
        assert!(p.secret_file.is_none());
        assert!(p.secret.is_none());
    }

    #[test]
    fn resolve_direct_address_with_cli_overrides() {
        let p = resolve_hub_profile(
            "host:9100",
            Some("/path/to/secret"),
            Some("mysecret"),
            "control",
        ).unwrap();
        assert_eq!(p.address, "host:9100");
        assert_eq!(p.secret_file.as_deref(), Some("/path/to/secret"));
        assert_eq!(p.secret.as_deref(), Some("mysecret"));
        assert_eq!(p.scope.as_deref(), Some("control"));
    }

    #[test]
    fn resolve_profile_lookup() {
        let mut config = HubsConfig::default();
        config.hubs.insert("prod".to_string(), HubEntry {
            address: "prod.example.com:9100".to_string(),
            secret_file: Some("/etc/termlink/prod.key".to_string()),
            secret: None,
            scope: Some("observe".to_string()),
        });

        let p = resolve_hub_profile_with_config("prod", None, None, "observe", &config).unwrap();
        assert_eq!(p.address, "prod.example.com:9100");
        assert_eq!(p.secret_file.as_deref(), Some("/etc/termlink/prod.key"));
        assert_eq!(p.scope.as_deref(), Some("observe"));
    }

    #[test]
    fn resolve_profile_cli_overrides_profile() {
        let mut config = HubsConfig::default();
        config.hubs.insert("dev".to_string(), HubEntry {
            address: "dev.local:9100".to_string(),
            secret_file: Some("/default/key".to_string()),
            secret: None,
            scope: Some("observe".to_string()),
        });

        let p = resolve_hub_profile_with_config(
            "dev",
            Some("/override/key"),
            Some("inline-secret"),
            "execute",
            &config,
        ).unwrap();
        assert_eq!(p.address, "dev.local:9100");
        assert_eq!(p.secret_file.as_deref(), Some("/override/key"));
        assert_eq!(p.secret.as_deref(), Some("inline-secret"));
        assert_eq!(p.scope.as_deref(), Some("execute"));
    }

    #[test]
    fn resolve_profile_not_found() {
        let config = HubsConfig::default();
        let result = resolve_hub_profile_with_config("nonexistent", None, None, "observe", &config);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not found"), "Expected 'not found' in: {}", err);
    }

    #[test]
    fn hubs_config_toml_roundtrip() {
        let mut config = HubsConfig::default();
        config.hubs.insert("staging".to_string(), HubEntry {
            address: "staging.example.com:9100".to_string(),
            secret_file: Some("/keys/staging.key".to_string()),
            secret: None,
            scope: Some("control".to_string()),
        });
        config.hubs.insert("minimal".to_string(), HubEntry {
            address: "min.local:9100".to_string(),
            secret_file: None,
            secret: None,
            scope: None,
        });

        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: HubsConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(parsed.hubs.len(), 2);
        let staging = parsed.hubs.get("staging").unwrap();
        assert_eq!(staging.address, "staging.example.com:9100");
        assert_eq!(staging.secret_file.as_deref(), Some("/keys/staging.key"));
        assert_eq!(staging.scope.as_deref(), Some("control"));

        let minimal = parsed.hubs.get("minimal").unwrap();
        assert_eq!(minimal.address, "min.local:9100");
        assert!(minimal.secret_file.is_none());
        assert!(minimal.scope.is_none());
    }

    #[test]
    fn hubs_config_empty_deserialize() {
        let config: HubsConfig = toml::from_str("").unwrap();
        assert!(config.hubs.is_empty());
    }

    #[test]
    fn save_and_load_hubs_config() {
        let tmp = std::env::temp_dir().join(format!("tl-config-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        // Temporarily override HOME
        let orig_home = std::env::var("HOME").ok();
        // SAFETY: test runs single-threaded (cargo test default), no concurrent HOME reads
        unsafe { std::env::set_var("HOME", &tmp); }

        let mut config = HubsConfig::default();
        config.hubs.insert("test".to_string(), HubEntry {
            address: "test.local:9100".to_string(),
            secret_file: None,
            secret: Some("s3cret".to_string()),
            scope: None,
        });

        save_hubs_config(&config).unwrap();
        let loaded = load_hubs_config();

        // Restore HOME
        // SAFETY: restoring original value, test is single-threaded
        if let Some(h) = orig_home {
            unsafe { std::env::set_var("HOME", h); }
        }

        assert_eq!(loaded.hubs.len(), 1);
        let entry = loaded.hubs.get("test").unwrap();
        assert_eq!(entry.address, "test.local:9100");
        assert_eq!(entry.secret.as_deref(), Some("s3cret"));

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
