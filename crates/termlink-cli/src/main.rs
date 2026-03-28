mod cli;
mod commands;
mod config;
mod util;

use anyhow::Result;
use clap::{CommandFactory, Parser};

use cli::*;
use config::resolve_hub_profile;
use util::resolve_target;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "termlink=info".into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        // Session management
        Command::Register { name, roles, tags, shell, self_mode, token_secret, allowed_commands } => {
            if self_mode {
                commands::session::cmd_register_self(name, roles, tags).await
            } else {
                commands::session::cmd_register(name, roles, tags, shell, token_secret, allowed_commands).await
            }
        }
        Command::List { all, json } => commands::session::cmd_list(all, json),
        Command::Ping { target } => commands::session::cmd_ping(&resolve_target(target)?).await,
        Command::Status { target, json } => commands::session::cmd_status(&resolve_target(target)?, json).await,
        Command::Info { json } => commands::session::cmd_info(json),
        Command::Send { target, method, params } => commands::session::cmd_send(&target, &method, &params).await,
        Command::Interact { target, command, timeout, poll_ms, strip_ansi, json } => {
            commands::pty::cmd_interact(&resolve_target(target)?, &command, timeout, poll_ms, strip_ansi, json).await
        }
        Command::Exec { target, command, cwd, timeout } => {
            commands::session::cmd_exec(&target, &command, cwd.as_deref(), timeout).await
        }
        Command::Signal { target, signal } => commands::session::cmd_signal(&target, &signal).await,

        // PTY subcommand group
        Command::Pty(pty) => match pty {
            PtyCommand::Output { target, lines, bytes, strip_ansi } => commands::pty::cmd_output(&resolve_target(target)?, lines, bytes, strip_ansi).await,
            PtyCommand::Inject { target, text, enter, key } => {
                commands::pty::cmd_inject(&resolve_target(target)?, &text, enter, key.as_deref()).await
            }
            PtyCommand::Attach { target, poll_ms } => commands::pty::cmd_attach(&resolve_target(target)?, poll_ms).await,
            PtyCommand::Resize { target, cols, rows } => commands::pty::cmd_resize(&target, cols, rows).await,
            PtyCommand::Stream { target } => commands::pty::cmd_stream(&resolve_target(target)?).await,
            PtyCommand::Mirror { target, scrollback } => commands::pty::cmd_mirror(&resolve_target(target)?, scrollback).await,
        },

        // Event subcommand group
        Command::Event(ev) => match ev {
            EventCommand::Poll { target, since, topic, json: _ } => {
                commands::events::cmd_events(&resolve_target(target)?, since, topic.as_deref()).await
            }
            EventCommand::Watch { targets, interval, topic } => {
                commands::events::cmd_watch(targets, interval, topic.as_deref()).await
            }
            EventCommand::Emit { target, topic, payload } => {
                commands::events::cmd_emit(&target, &topic, &payload).await
            }
            EventCommand::EmitTo { target, topic, payload, from } => {
                commands::events::cmd_emit_to(&target, &topic, &payload, from.as_deref()).await
            }
            EventCommand::Broadcast { topic, payload, targets } => {
                commands::events::cmd_broadcast(&topic, &payload, targets).await
            }
            EventCommand::Wait { target, topic, timeout, interval } => {
                commands::events::cmd_wait(&resolve_target(target)?, &topic, timeout, interval).await
            }
            EventCommand::Topics { target, json: _ } => commands::events::cmd_topics(target.as_deref()).await,
            EventCommand::Collect { targets, topic, interval, count } => {
                commands::events::cmd_collect(targets, topic.as_deref(), interval, count).await
            }
        },

        // Hidden backward-compat aliases (PTY)
        Command::Output { target, lines, bytes, strip_ansi } => commands::pty::cmd_output(&resolve_target(target)?, lines, bytes, strip_ansi).await,
        Command::Inject { target, text, enter, key } => {
            commands::pty::cmd_inject(&resolve_target(target)?, &text, enter, key.as_deref()).await
        }
        Command::Attach { target, poll_ms } => commands::pty::cmd_attach(&resolve_target(target)?, poll_ms).await,
        Command::Resize { target, cols, rows } => commands::pty::cmd_resize(&target, cols, rows).await,
        Command::Stream { target } => commands::pty::cmd_stream(&resolve_target(target)?).await,
        Command::Mirror { target, scrollback } => commands::pty::cmd_mirror(&resolve_target(target)?, scrollback).await,

        // Hidden backward-compat aliases (Event)
        Command::Events { target, since, topic, json: _ } => {
            commands::events::cmd_events(&resolve_target(target)?, since, topic.as_deref()).await
        }
        Command::Broadcast { topic, payload, targets } => {
            commands::events::cmd_broadcast(&topic, &payload, targets).await
        }
        Command::Emit { target, topic, payload } => {
            commands::events::cmd_emit(&target, &topic, &payload).await
        }
        Command::EmitTo { target, topic, payload, from } => {
            commands::events::cmd_emit_to(&target, &topic, &payload, from.as_deref()).await
        }
        Command::Watch { targets, interval, topic } => {
            commands::events::cmd_watch(targets, interval, topic.as_deref()).await
        }
        Command::Topics { target, json: _ } => commands::events::cmd_topics(target.as_deref()).await,
        Command::Collect { targets, topic, interval, count } => {
            commands::events::cmd_collect(targets, topic.as_deref(), interval, count).await
        }
        Command::Wait { target, topic, timeout, interval } => {
            commands::events::cmd_wait(&resolve_target(target)?, &topic, timeout, interval).await
        }

        // Metadata & Discovery
        Command::Tag { target, set, add, remove } => {
            commands::metadata::cmd_tag(&target, set, add, remove).await
        }
        Command::Discover { tag, role, cap, name, json } => {
            commands::metadata::cmd_discover(tag, role, cap, name, json)
        }
        Command::Kv { target, action } => commands::metadata::cmd_kv(&target, action).await,

        // Execution
        Command::Run { name, tags, timeout, command } => {
            commands::execution::cmd_run(name, tags, timeout, command).await
        }
        Command::Request { target, topic, payload, reply_topic, timeout, interval } => {
            commands::execution::cmd_request(&target, &topic, &payload, &reply_topic, timeout, interval).await
        }
        Command::Spawn { name, roles, tags, wait, wait_timeout, shell, backend, command } => {
            commands::execution::cmd_spawn(name, roles, tags, wait, wait_timeout, shell, backend, command).await
        }
        Command::Dispatch { count, timeout, topic, name, tags, backend, json, command } => {
            commands::dispatch::cmd_dispatch(count, timeout, &topic, name, tags, backend, json, command).await
        }

        // Infrastructure
        Command::Clean { dry_run } => commands::session::cmd_clean(dry_run),
        Command::Hub { action } => match action {
            None | Some(HubAction::Start { tcp: None }) => commands::infrastructure::cmd_hub_start(None).await,
            Some(HubAction::Start { tcp: Some(ref addr) }) => commands::infrastructure::cmd_hub_start(Some(addr)).await,
            Some(HubAction::Stop) => commands::infrastructure::cmd_hub_stop(),
            Some(HubAction::Status) => commands::infrastructure::cmd_hub_status(),
        },
        Command::Mcp { action } => match action {
            McpAction::Serve => termlink_mcp::server::run_stdio().await,
        },
        Command::Token { action } => match action {
            TokenAction::Create { target, scope, ttl } => {
                commands::token::cmd_token_create(&target, &scope, ttl).await
            }
            TokenAction::Inspect { token } => commands::token::cmd_token_inspect(&token),
        },
        Command::Agent { action } => match action {
            AgentAction::Ask { target, action, params, from, timeout, interval } => {
                commands::agent::cmd_agent_ask(&target, &action, &params, from.as_deref(), timeout, interval).await
            }
            AgentAction::Listen { target, timeout, interval } => {
                commands::agent::cmd_agent_listen(&target, timeout, interval).await
            }
            AgentAction::Negotiate { specialist, schema, draft, from, max_rounds, timeout, interval } => {
                commands::agent::cmd_agent_negotiate(&specialist, &schema, &draft, from.as_deref(), max_rounds, timeout, interval).await
            }
        },
        Command::File { action } => match action {
            FileAction::Send { target, path, chunk_size } => {
                commands::file::cmd_file_send(&target, &path, chunk_size).await
            }
            FileAction::Receive { target, output_dir, timeout, interval } => {
                commands::file::cmd_file_receive(&target, &output_dir, timeout, interval).await
            }
        },
        Command::Remote { action } => match action {
            RemoteAction::Ping { hub, session, secret_file, secret, scope } => {
                let p = resolve_hub_profile(&hub, secret_file.as_deref(), secret.as_deref(), &scope)?;
                commands::remote::cmd_remote_ping(&p.address, session.as_deref(), p.secret_file.as_deref(), p.secret.as_deref(), p.scope.as_deref().unwrap_or("observe")).await
            }
            RemoteAction::List { hub, secret_file, secret, scope, name, tags, roles, json } => {
                let p = resolve_hub_profile(&hub, secret_file.as_deref(), secret.as_deref(), &scope)?;
                commands::remote::cmd_remote_list(&p.address, p.secret_file.as_deref(), p.secret.as_deref(), p.scope.as_deref().unwrap_or("observe"), name.as_deref(), tags.as_deref(), roles.as_deref(), json).await
            }
            RemoteAction::Status { hub, session, secret_file, secret, scope, json } => {
                let p = resolve_hub_profile(&hub, secret_file.as_deref(), secret.as_deref(), &scope)?;
                let session = commands::remote::resolve_remote_target(session, &p.address, p.secret_file.as_deref(), p.secret.as_deref(), p.scope.as_deref().unwrap_or("observe")).await?;
                commands::remote::cmd_remote_status(&p.address, &session, p.secret_file.as_deref(), p.secret.as_deref(), p.scope.as_deref().unwrap_or("observe"), json).await
            }
            RemoteAction::Inject { hub, session, text, secret_file, secret, enter, key, delay_ms, scope, json } => {
                let p = resolve_hub_profile(&hub, secret_file.as_deref(), secret.as_deref(), &scope)?;
                commands::remote::cmd_remote_inject(&p.address, &session, &text, p.secret_file.as_deref(), p.secret.as_deref(), enter, key.as_deref(), delay_ms, p.scope.as_deref().unwrap_or("control"), json).await
            }
            RemoteAction::SendFile { hub, session, path, secret_file, secret, chunk_size, scope, json } => {
                let p = resolve_hub_profile(&hub, secret_file.as_deref(), secret.as_deref(), &scope)?;
                commands::remote::cmd_remote_send_file(&p.address, &session, &path, p.secret_file.as_deref(), p.secret.as_deref(), chunk_size, p.scope.as_deref().unwrap_or("control"), json).await
            }
            RemoteAction::Events { hub, secret_file, secret, scope, topic, targets, interval, count, json } => {
                let p = resolve_hub_profile(&hub, secret_file.as_deref(), secret.as_deref(), &scope)?;
                commands::remote::cmd_remote_events(&p.address, p.secret_file.as_deref(), p.secret.as_deref(), p.scope.as_deref().unwrap_or("observe"), topic.as_deref(), targets.as_deref(), interval, count, json).await
            }
            RemoteAction::Exec { hub, session, command, secret_file, secret, scope, timeout, cwd, json } => {
                let p = resolve_hub_profile(&hub, secret_file.as_deref(), secret.as_deref(), &scope)?;
                commands::remote::cmd_remote_exec(&p.address, &session, &command, p.secret_file.as_deref(), p.secret.as_deref(), p.scope.as_deref().unwrap_or("execute"), timeout, cwd.as_deref(), json).await
            }
            RemoteAction::Push { hub, session, file, message, secret_file, secret, scope, json } => {
                let p = resolve_hub_profile(&hub, secret_file.as_deref(), secret.as_deref(), &scope)?;
                commands::push::cmd_push(&p.address, &session, file.as_deref(), message.as_deref(), p.secret_file.as_deref(), p.secret.as_deref(), p.scope.as_deref().unwrap_or("execute"), json).await
            }
            RemoteAction::Profile { action } => {
                commands::remote::cmd_remote_profile(action)
            }
        },
        Command::Doctor { json, fix } => commands::infrastructure::cmd_doctor(json, fix).await,
        Command::Vendor { action, source, target, dry_run } => {
            if let Some(action) = action {
                match action {
                    VendorAction::Status { target } => commands::vendor::cmd_vendor_status(target.as_deref()),
                }
            } else {
                commands::vendor::cmd_vendor(source.as_deref(), target.as_deref(), dry_run)
            }
        }
        Command::Completions { shell } => {
            clap_complete::generate(
                shell,
                &mut Cli::command(),
                "termlink",
                &mut std::io::stdout(),
            );
            Ok(())
        }
        Command::Version { json } => {
            let version = env!("CARGO_PKG_VERSION");
            let commit = option_env!("GIT_COMMIT").unwrap_or("unknown");
            let target = option_env!("BUILD_TARGET").unwrap_or("unknown");

            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "version": version,
                        "commit": commit,
                        "target": target,
                    })
                );
            } else {
                println!("termlink {version} ({commit}) [{target}]");
            }
            Ok(())
        }
    }
}
