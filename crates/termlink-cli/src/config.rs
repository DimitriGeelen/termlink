use anyhow::Result;

/// Resolved hub connection parameters from profile or CLI args.
#[derive(Debug)]
pub(crate) struct HubProfile {
    pub address: String,
    pub secret_file: Option<String>,
    pub secret: Option<String>,
    pub scope: Option<String>,
    /// T-3543: scoped token minted on the hub host. Used only when no secret is
    /// configured or passed — a secret, when present, still wins.
    pub token_file: Option<String>,
}

// T-3214: the hubs.toml model, config-dir resolver and read-only loader moved to
// the neutral `termlink-doctor` crate so the CLI and MCP doctors read hubs.toml
// through ONE parser. Re-exported so every `crate::config::...` path still works.
#[allow(unused_imports)]
pub(crate) use termlink_doctor::hubs_config::{
    hubs_config_path, load_hubs_config, resolve_config_dir_from, termlink_config_dir, HubEntry,
    HubsConfig,
};

/// Strict loader for MUTATION callers (T-2437). Propagates a parse
/// error instead of defaulting so a load→modify→save cycle over a
/// corrupt hubs.toml refuses the edit rather than silently replacing
/// the whole file with near-empty content (losing every declared hub
/// + bootstrap anchor). Missing file is still a normal empty config.
pub(crate) fn load_hubs_config_strict() -> Result<HubsConfig> {
    let path = hubs_config_path();
    match std::fs::read_to_string(&path) {
        Ok(content) => toml::from_str(&content).map_err(|e| {
            anyhow::anyhow!(
                "{} is unparseable ({e}); refusing to modify it — fix the syntax first",
                path.display()
            )
        }),
        Err(_) => Ok(HubsConfig::default()),
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
        // Direct address. T-2269: if the caller passed no explicit secret,
        // reverse-resolve it by matching the `address` field of a hubs.toml
        // profile — mirrors `channel post --hub`'s resolve_hub_secret_hex so a
        // bare host:port works exactly like the profile name for the same hub.
        // An explicit --secret-file/--secret still wins; a no-match stays
        // address-only (caller must supply a secret).
        let mut secret_file = cli_secret_file.map(String::from);
        let mut secret = cli_secret.map(String::from);
        let mut token_file = None;
        if secret_file.is_none() && secret.is_none() {
            if let Some(entry) = config.hubs.values().find(|e| e.address == hub_arg) {
                secret_file = entry.secret_file.clone();
                secret = entry.secret.clone();
                token_file = entry.token_file.clone();
            }
        }
        return Ok(HubProfile {
            address: hub_arg.to_string(),
            secret_file,
            secret,
            scope: Some(cli_scope.to_string()),
            token_file,
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
        token_file: entry.token_file.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_dir_uses_home_when_set() {
        // Behavior-preserving: HOME set → $HOME/.termlink, never last_resort.
        let (dir, lr) = resolve_config_dir_from(Some("/home/u"), std::path::Path::new("/tmp"), 1000);
        assert_eq!(dir, std::path::PathBuf::from("/home/u/.termlink"));
        assert!(!lr);
    }

    #[test]
    fn config_dir_empty_home_is_treated_as_unset_not_root() {
        // An exported-but-empty `HOME=` must NOT resolve to `/.termlink`.
        let (dir, lr) = resolve_config_dir_from(Some(""), std::path::Path::new("/var/tmp"), 1000);
        assert_eq!(dir, std::path::PathBuf::from("/var/tmp/termlink-1000"));
        assert!(lr);
    }

    // LOAD-BEARING (T-2629): with HOME unset the resolver must NOT land in the
    // shared, world-writable `/tmp/.termlink` (which would leak hub profiles +
    // bootstrap trust anchors to any local user, and make fleet discovery read a
    // nonexistent path). Temp-revert `resolve_config_dir_from` to the old
    // `HOME.unwrap_or("/tmp") + .termlink` shape and this test FAILS — proving the
    // guard is what keeps auth material out of the shared dir.
    #[test]
    fn config_dir_home_unset_never_shared_dot_termlink() {
        let (dir, lr) = resolve_config_dir_from(None, std::path::Path::new("/tmp"), 4242);
        assert!(lr, "HOME-unset must flag the last_resort (loud) path");
        assert_eq!(dir, std::path::PathBuf::from("/tmp/termlink-4242"));
        // The specific regression guard: never the shared, world-writable dir.
        assert_ne!(dir, std::path::PathBuf::from("/tmp/.termlink"));
        assert!(
            !dir.ends_with(".termlink"),
            "must not be the shared .termlink dir"
        );
        assert!(
            dir.to_string_lossy().contains("termlink-4242"),
            "must be UID-namespaced"
        );
    }

    #[test]
    fn config_dir_honors_tmpdir_base_for_last_resort() {
        let (dir, lr) = resolve_config_dir_from(None, std::path::Path::new("/custom/tmp"), 7);
        assert_eq!(dir, std::path::PathBuf::from("/custom/tmp/termlink-7"));
        assert!(lr);
    }

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
            token_file: None,
            bootstrap_from: None,
        });

        let p = resolve_hub_profile_with_config("prod", None, None, "observe", &config).unwrap();
        assert_eq!(p.address, "prod.example.com:9100");
        assert_eq!(p.secret_file.as_deref(), Some("/etc/termlink/prod.key"));
        assert_eq!(p.scope.as_deref(), Some("observe"));
    }

    #[test]
    fn resolve_profile_carries_token_file_by_name_and_by_address() {
        // T-3543: a node profile with only token_file resolves to it either way.
        let mut config = HubsConfig::default();
        config.hubs.insert("node".to_string(), HubEntry {
            address: "hub.example:9100".to_string(),
            secret_file: None,
            secret: None,
            scope: Some("interact".to_string()),
            token_file: Some("~/.termlink/node.token".to_string()),
            bootstrap_from: None,
        });
        for arg in ["node", "hub.example:9100"] {
            let p = resolve_hub_profile_with_config(arg, None, None, "interact", &config).unwrap();
            assert_eq!(p.token_file.as_deref(), Some("~/.termlink/node.token"), "{arg}");
            assert!(p.secret_file.is_none() && p.secret.is_none(), "{arg}");
        }
    }

    #[test]
    fn resolve_profile_cli_overrides_profile() {
        let mut config = HubsConfig::default();
        config.hubs.insert("dev".to_string(), HubEntry {
            address: "dev.local:9100".to_string(),
            secret_file: Some("/default/key".to_string()),
            secret: None,
            scope: Some("observe".to_string()),
            token_file: None,
            bootstrap_from: None,
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

    // T-2269: bare host:port reverse-resolves the secret from a matching profile.
    #[test]
    fn resolve_bare_address_reverse_resolves_secret() {
        let mut config = HubsConfig::default();
        config.hubs.insert("ring20-management".to_string(), HubEntry {
            address: "192.168.10.122:9100".to_string(),
            secret_file: Some("/root/.termlink/secrets/ring20-management.hex".to_string()),
            secret: None,
            scope: Some("observe".to_string()),
            token_file: None,
            bootstrap_from: None,
        });

        // Passing the raw address (with ':') now resolves the same secret the
        // profile name would, instead of dying "secret required".
        let p = resolve_hub_profile_with_config(
            "192.168.10.122:9100", None, None, "observe", &config,
        ).unwrap();
        assert_eq!(p.address, "192.168.10.122:9100");
        assert_eq!(
            p.secret_file.as_deref(),
            Some("/root/.termlink/secrets/ring20-management.hex"),
            "bare address should reverse-resolve the profile's secret_file"
        );
        assert_eq!(p.scope.as_deref(), Some("observe"));
    }

    // T-2269: an explicit CLI secret still wins over the reverse-resolved one.
    #[test]
    fn resolve_bare_address_explicit_secret_overrides() {
        let mut config = HubsConfig::default();
        config.hubs.insert("ring20-management".to_string(), HubEntry {
            address: "192.168.10.122:9100".to_string(),
            secret_file: Some("/profile/key".to_string()),
            secret: None,
            scope: Some("observe".to_string()),
            token_file: None,
            bootstrap_from: None,
        });

        let p = resolve_hub_profile_with_config(
            "192.168.10.122:9100", Some("/cli/override"), None, "control", &config,
        ).unwrap();
        assert_eq!(p.secret_file.as_deref(), Some("/cli/override"));
    }

    // T-2269: a bare address with no matching profile stays address-only
    // (no regression to the prior direct-address path).
    #[test]
    fn resolve_bare_address_no_match_stays_bare() {
        let mut config = HubsConfig::default();
        config.hubs.insert("other".to_string(), HubEntry {
            address: "10.0.0.1:9100".to_string(),
            secret_file: Some("/other/key".to_string()),
            secret: None,
            scope: None,
            token_file: None,
            bootstrap_from: None,
        });

        let p = resolve_hub_profile_with_config(
            "192.168.10.122:9100", None, None, "observe", &config,
        ).unwrap();
        assert_eq!(p.address, "192.168.10.122:9100");
        assert!(p.secret_file.is_none(), "no matching profile → no secret");
        assert!(p.secret.is_none());
    }

    #[test]
    fn hubs_config_toml_roundtrip() {
        let mut config = HubsConfig::default();
        config.hubs.insert("staging".to_string(), HubEntry {
            address: "staging.example.com:9100".to_string(),
            secret_file: Some("/keys/staging.key".to_string()),
            secret: None,
            scope: Some("control".to_string()),
            token_file: None,
            bootstrap_from: None,
        });
        config.hubs.insert("minimal".to_string(), HubEntry {
            address: "min.local:9100".to_string(),
            secret_file: None,
            secret: None,
            scope: None,
            token_file: None,
            bootstrap_from: None,
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
    fn corrupt_hubs_toml_strict_errs_lenient_defaults() {
        // T-2437: a hubs.toml that EXISTS but fails to parse must (a)
        // refuse mutation loads with an error naming the path, and (b)
        // degrade read-only loads to empty (loudly, via stderr — the
        // behavioral contract asserted here is the fallback itself).
        let _guard = crate::test_env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        let tmp = std::env::temp_dir().join(format!(
            "tl-config-corrupt-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(tmp.join(".termlink")).unwrap();
        std::fs::write(
            tmp.join(".termlink/hubs.toml"),
            "[hubs.broken\naddress = \"oops",
        )
        .unwrap();

        let orig_home = std::env::var("HOME").ok();
        // SAFETY: guarded by ENV_LOCK.
        unsafe { std::env::set_var("HOME", &tmp); }

        let strict = load_hubs_config_strict();
        let lenient = load_hubs_config();

        // Restore HOME before asserting so a panic can't leak the override.
        // SAFETY: guarded by ENV_LOCK.
        unsafe {
            match orig_home {
                Some(h) => std::env::set_var("HOME", h),
                None => std::env::remove_var("HOME"),
            }
        }
        let _ = std::fs::remove_dir_all(&tmp);

        let err = match strict {
            Err(e) => e,
            Ok(_) => panic!("strict load must refuse a corrupt hubs.toml"),
        };
        assert!(
            err.to_string().contains("hubs.toml"),
            "error must name the file: {err}"
        );
        assert!(lenient.hubs.is_empty(), "lenient load degrades to empty");
    }

    #[test]
    fn missing_hubs_toml_is_normal_empty_for_both_loaders() {
        let _guard = crate::test_env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        let tmp = std::env::temp_dir().join(format!(
            "tl-config-missing-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let orig_home = std::env::var("HOME").ok();
        // SAFETY: guarded by ENV_LOCK.
        unsafe { std::env::set_var("HOME", &tmp); }

        let strict = load_hubs_config_strict();
        let lenient = load_hubs_config();

        // SAFETY: guarded by ENV_LOCK.
        unsafe {
            match orig_home {
                Some(h) => std::env::set_var("HOME", h),
                None => std::env::remove_var("HOME"),
            }
        }
        let _ = std::fs::remove_dir_all(&tmp);

        let strict = match strict {
            Ok(cfg) => cfg,
            Err(e) => panic!("missing file is not an error: {e}"),
        };
        assert!(strict.hubs.is_empty());
        assert!(lenient.hubs.is_empty());
    }

    #[test]
    fn save_and_load_hubs_config() {
        // Serialize with the crate-wide env lock — cargo runs tests in
        // parallel threads, and other modules also mutate HOME in tests.
        let _guard = crate::test_env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        let tmp = std::env::temp_dir().join(format!(
            "tl-config-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        // Temporarily override HOME
        let orig_home = std::env::var("HOME").ok();
        // SAFETY: guarded by ENV_LOCK.
        unsafe { std::env::set_var("HOME", &tmp); }

        let mut config = HubsConfig::default();
        config.hubs.insert("test".to_string(), HubEntry {
            address: "test.local:9100".to_string(),
            secret_file: None,
            secret: Some("s3cret".to_string()),
            scope: None,
            token_file: None,
            bootstrap_from: None,
        });

        save_hubs_config(&config).unwrap();
        let loaded = load_hubs_config();

        // Restore HOME
        // SAFETY: guarded by ENV_LOCK.
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
