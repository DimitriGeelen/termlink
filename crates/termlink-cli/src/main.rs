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
        Command::Register { name, roles, tags, shell, self_mode, token_secret, allowed_commands, json } => {
            if self_mode {
                commands::session::cmd_register_self(name, roles, tags, json).await
            } else {
                commands::session::cmd_register(name, roles, tags, shell, token_secret, allowed_commands, json).await
            }
        }
        Command::List { all, json, tag, name, role, cap, count, names, ids, no_header } => commands::session::cmd_list(all, json, tag.as_deref(), name.as_deref(), role.as_deref(), cap.as_deref(), count, names, ids, no_header),
        Command::Ping { target, json, timeout } => commands::session::cmd_ping(&resolve_target(target)?, json, timeout).await,
        Command::Status { target, json, short, timeout } => commands::session::cmd_status(&resolve_target(target)?, json, short, timeout).await,
        Command::Info { json } => commands::session::cmd_info(json),
        Command::Send { target, method, params, json, timeout } => commands::session::cmd_send(&target, &method, &params, json, timeout).await,
        Command::Interact { target, command, timeout, poll_ms, strip_ansi, json } => {
            commands::pty::cmd_interact(&target, &command, timeout, poll_ms, strip_ansi, json).await
        }
        Command::Exec { target, command, cwd, timeout, json } => {
            commands::session::cmd_exec(&target, &command, cwd.as_deref(), timeout, json).await
        }
        Command::Signal { target, signal, json, timeout } => commands::session::cmd_signal(&target, &signal, json, timeout).await,

        // PTY subcommand group
        Command::Pty(pty) => match pty {
            PtyCommand::Output { target, lines, bytes, strip_ansi, json, timeout } => commands::pty::cmd_output(&resolve_target(target)?, lines, bytes, strip_ansi, json, timeout).await,
            PtyCommand::Inject { target, text, enter, key, json, timeout } => {
                commands::pty::cmd_inject(&target, &text, enter, key.as_deref(), json, timeout).await
            }
            PtyCommand::Attach { target, poll_ms } => commands::pty::cmd_attach(&resolve_target(target)?, poll_ms).await,
            PtyCommand::Resize { target, cols, rows, json, timeout } => commands::pty::cmd_resize(&target, cols, rows, json, timeout).await,
            PtyCommand::Stream { target } => commands::pty::cmd_stream(&resolve_target(target)?).await,
            PtyCommand::Mirror { target, scrollback } => commands::pty::cmd_mirror(&resolve_target(target)?, scrollback).await,
        },

        // Event subcommand group
        Command::Event(ev) => match ev {
            EventCommand::Poll { target, since, topic, json, timeout, payload_only } => {
                commands::events::cmd_events(&resolve_target(target)?, since, topic.as_deref(), json, timeout, payload_only).await
            }
            EventCommand::Watch { targets, interval, topic, json, timeout, count, payload_only } => {
                commands::events::cmd_watch(targets, interval, topic.as_deref(), json, timeout, count, payload_only).await
            }
            EventCommand::Emit { target, topic, payload, json, timeout } => {
                commands::events::cmd_emit(&target, &topic, &payload, json, timeout).await
            }
            EventCommand::EmitTo { target, topic, payload, from, json, timeout } => {
                commands::events::cmd_emit_to(&target, &topic, &payload, from.as_deref(), json, timeout).await
            }
            EventCommand::Broadcast { topic, payload, targets, json, timeout } => {
                commands::events::cmd_broadcast(&topic, &payload, targets, json, timeout).await
            }
            EventCommand::Wait { target, topic, timeout, interval, json } => {
                commands::events::cmd_wait(&resolve_target(target)?, &topic, timeout, interval, json).await
            }
            EventCommand::Topics { target, json, timeout } => commands::events::cmd_topics(target.as_deref(), json, timeout).await,
            EventCommand::Collect { targets, topic, interval, count, json, timeout, payload_only } => {
                commands::events::cmd_collect(targets, topic.as_deref(), interval, count, json, timeout, payload_only).await
            }
        },

        // Hidden backward-compat aliases (PTY)
        Command::Output { target, lines, bytes, strip_ansi, json, timeout } => commands::pty::cmd_output(&resolve_target(target)?, lines, bytes, strip_ansi, json, timeout).await,
        Command::Inject { target, text, enter, key, json, timeout } => {
            commands::pty::cmd_inject(&target, &text, enter, key.as_deref(), json, timeout).await
        }
        Command::Attach { target, poll_ms } => commands::pty::cmd_attach(&resolve_target(target)?, poll_ms).await,
        Command::Resize { target, cols, rows, json, timeout } => commands::pty::cmd_resize(&target, cols, rows, json, timeout).await,
        Command::Stream { target } => commands::pty::cmd_stream(&resolve_target(target)?).await,
        Command::Mirror { target, scrollback } => commands::pty::cmd_mirror(&resolve_target(target)?, scrollback).await,

        // Hidden backward-compat aliases (Event)
        Command::Events { target, since, topic, json, timeout, payload_only } => {
            commands::events::cmd_events(&resolve_target(target)?, since, topic.as_deref(), json, timeout, payload_only).await
        }
        Command::Broadcast { topic, payload, targets, json, timeout } => {
            commands::events::cmd_broadcast(&topic, &payload, targets, json, timeout).await
        }
        Command::Emit { target, topic, payload, json, timeout } => {
            commands::events::cmd_emit(&target, &topic, &payload, json, timeout).await
        }
        Command::EmitTo { target, topic, payload, from, json, timeout } => {
            commands::events::cmd_emit_to(&target, &topic, &payload, from.as_deref(), json, timeout).await
        }
        Command::Watch { targets, interval, topic, json, timeout, count, payload_only } => {
            commands::events::cmd_watch(targets, interval, topic.as_deref(), json, timeout, count, payload_only).await
        }
        Command::Topics { target, json, timeout } => commands::events::cmd_topics(target.as_deref(), json, timeout).await,
        Command::Collect { targets, topic, interval, count, json, timeout, payload_only } => {
            commands::events::cmd_collect(targets, topic.as_deref(), interval, count, json, timeout, payload_only).await
        }
        Command::Wait { target, topic, timeout, interval, json } => {
            commands::events::cmd_wait(&resolve_target(target)?, &topic, timeout, interval, json).await
        }

        // Metadata & Discovery
        Command::Tag { target, set, add, remove, json, timeout } => {
            commands::metadata::cmd_tag(&target, set, add, remove, json, timeout).await
        }
        Command::Discover { tag, role, cap, name, json, count, first, wait, wait_timeout, id, no_header } => {
            commands::metadata::cmd_discover(tag, role, cap, name, json, count, first, wait, wait_timeout, id, no_header).await
        }
        Command::Kv { target, json, timeout, raw, keys, action } => commands::metadata::cmd_kv(&target, action, json, raw, keys, timeout).await,

        // Execution
        Command::Run { name, tags, timeout, json, command } => {
            commands::execution::cmd_run(name, tags, timeout, json, command).await
        }
        Command::Request { target, topic, payload, reply_topic, timeout, interval, json } => {
            commands::execution::cmd_request(&target, &topic, &payload, &reply_topic, timeout, interval, json).await
        }
        Command::Spawn { name, roles, tags, wait, wait_timeout, shell, backend, json, command } => {
            commands::execution::cmd_spawn(name, roles, tags, wait, wait_timeout, shell, backend, json, command).await
        }
        Command::Dispatch { count, timeout, topic, name, tags, backend, json, command } => {
            commands::dispatch::cmd_dispatch(count, timeout, &topic, name, tags, backend, json, command).await
        }

        // Infrastructure
        Command::Clean { dry_run, json } => commands::session::cmd_clean(dry_run, json),
        Command::Hub { action } => match action {
            None | Some(HubAction::Start { tcp: None }) => commands::infrastructure::cmd_hub_start(None).await,
            Some(HubAction::Start { tcp: Some(ref addr) }) => commands::infrastructure::cmd_hub_start(Some(addr)).await,
            Some(HubAction::Stop { json }) => commands::infrastructure::cmd_hub_stop(json),
            Some(HubAction::Status { json }) => commands::infrastructure::cmd_hub_status(json),
        },
        Command::Mcp { action } => match action {
            McpAction::Serve => termlink_mcp::server::run_stdio().await,
        },
        Command::Token { action } => match action {
            TokenAction::Create { target, scope, ttl, json } => {
                commands::token::cmd_token_create(&target, &scope, ttl, json).await
            }
            TokenAction::Inspect { token, json } => commands::token::cmd_token_inspect(&token, json),
        },
        Command::Agent { action } => match action {
            AgentAction::Ask { target, action, params, from, timeout, interval, json } => {
                commands::agent::cmd_agent_ask(&target, &action, &params, from.as_deref(), timeout, interval, json).await
            }
            AgentAction::Listen { target, timeout, interval, json } => {
                commands::agent::cmd_agent_listen(&target, timeout, interval, json).await
            }
            AgentAction::Negotiate { specialist, schema, draft, from, max_rounds, timeout, interval } => {
                commands::agent::cmd_agent_negotiate(&specialist, &schema, &draft, from.as_deref(), max_rounds, timeout, interval).await
            }
        },
        Command::File { action } => match action {
            FileAction::Send { target, path, chunk_size, json, timeout } => {
                commands::file::cmd_file_send(&target, &path, chunk_size, json, timeout).await
            }
            FileAction::Receive { target, output_dir, timeout, interval, json } => {
                commands::file::cmd_file_receive(&target, &output_dir, timeout, interval, json).await
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
        Command::Vendor { action, source, target, dry_run, json } => {
            if let Some(action) = action {
                match action {
                    VendorAction::Status { target, json } => commands::vendor::cmd_vendor_status(target.as_deref(), json),
                }
            } else {
                commands::vendor::cmd_vendor(source.as_deref(), target.as_deref(), dry_run, json)
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
