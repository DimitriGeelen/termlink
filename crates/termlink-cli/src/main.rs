mod cli;
mod commands;
mod config;
mod manifest;
mod target;
mod util;
#[cfg(test)]
mod test_env_lock;

use anyhow::Result;
use clap::{CommandFactory, Parser};

use cli::*;
use commands::ListDisplayOpts;
use commands::remote::RemoteConn;
use config::resolve_hub_profile;
use util::resolve_target;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "termlink=info".into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        // Session management
        Command::Register { name, roles, tags, cap, shell, self_mode, token_secret, allowed_commands, json, quiet } => {
            if self_mode {
                commands::session::cmd_register_self(name, roles, tags, cap, json).await
            } else {
                commands::session::cmd_register(commands::session::RegisterOpts { name, roles, tags, cap, shell, enable_token_secret: token_secret, allowed_commands, json, quiet }).await
            }
        }
        Command::List { all, json, tag, name, role, cap, count, names, ids, first, wait, wait_timeout, no_header, sort } => {
            let display = ListDisplayOpts { count, first, names, ids, no_header, json };
            let filter = commands::session::ListFilterOpts { include_stale: all, tag: tag.as_deref(), name: name.as_deref(), role: role.as_deref(), cap: cap.as_deref(), wait, wait_timeout };
            commands::session::cmd_list(&filter, &display, sort.as_deref()).await
        }
        Command::Ping { target, json, timeout, hub, secret_file, secret, scope } => {
            let session = if hub.is_some() {
                // Cross-host: session name is mandatory (no interactive picker
                // over TCP yet — that would need a remote discover round-trip).
                target.ok_or_else(|| anyhow::anyhow!(
                    "--target requires an explicit session name (positional arg)"
                ))?
            } else {
                resolve_target(target)?
            };
            let opts = target::TargetOpts {
                hub,
                secret_file,
                secret,
                scope,
                session,
            };
            commands::session::cmd_ping(&opts, json, timeout).await
        }
        Command::Status { target, json, short, timeout, hub, secret_file, secret, scope } => {
            let session = if hub.is_some() {
                target.ok_or_else(|| anyhow::anyhow!(
                    "--target requires an explicit session name (positional arg)"
                ))?
            } else {
                resolve_target(target)?
            };
            let opts = target::TargetOpts {
                hub,
                secret_file,
                secret,
                scope,
                session,
            };
            commands::session::cmd_status(&opts, json, short, timeout).await
        }
        Command::Info { json, short, check } => commands::session::cmd_info(json, short, check),
        Command::Send { target, method, params, json, timeout } => commands::session::cmd_send(&target, &method, &params, json, timeout).await,
        Command::Interact { target, command, timeout, poll_ms, strip_ansi, json } => {
            commands::pty::cmd_interact(&target, &command, timeout, poll_ms, strip_ansi, json).await
        }
        Command::Exec { target, command, cwd, timeout, json } => {
            commands::session::cmd_exec(&target, &command, cwd.as_deref(), timeout, json).await
        }
        Command::Signal { target, signal, json, timeout, hub, secret_file, secret, scope } => {
            let opts = target::TargetOpts {
                hub,
                secret_file,
                secret,
                scope,
                session: target,
            };
            commands::session::cmd_signal(&opts, &signal, json, timeout).await
        }

        // PTY subcommand group
        Command::Pty(pty) => match pty {
            PtyCommand::Output { target, lines, bytes, strip_ansi, json, timeout } => commands::pty::cmd_output(&resolve_target(target)?, lines, bytes, strip_ansi, json, timeout).await,
            PtyCommand::Inject { target, text, enter, key, json, timeout } => {
                commands::pty::cmd_inject(&target, &text, enter, key.as_deref(), json, timeout).await
            }
            PtyCommand::Attach { target, poll_ms } => commands::pty::cmd_attach(&resolve_target(target)?, poll_ms).await,
            PtyCommand::Resize { target, cols, rows, json, timeout } => commands::pty::cmd_resize(&target, cols, rows, json, timeout).await,
            PtyCommand::Stream { target } => commands::pty::cmd_stream(&resolve_target(target)?).await,
            PtyCommand::Mirror { target, scrollback, raw, tag } => {
                if let Some(t) = tag {
                    commands::mirror_grid_composer::cmd_mirror_tag(&t, scrollback).await
                } else {
                    commands::pty::cmd_mirror(&resolve_target(target)?, scrollback, raw).await
                }
            }
        },

        // Event subcommand group
        Command::Event(ev) => match ev {
            EventCommand::Poll { target, since, topic, json, timeout, payload_only } => {
                commands::events::cmd_events(&resolve_target(target)?, since, topic.as_deref(), json, timeout, payload_only).await
            }
            EventCommand::Watch { targets, interval, topic, json, timeout, count, payload_only, since } => {
                let watch_opts = commands::events::WatchOpts { interval_ms: interval, topic_filter: topic.as_deref(), json, timeout_secs: timeout, max_count: count, payload_only, since };
                commands::events::cmd_watch(targets, watch_opts).await
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
            EventCommand::Wait { target, topic, timeout, interval, json, since } => {
                commands::events::cmd_wait(&resolve_target(target)?, &topic, timeout, interval, json, since).await
            }
            EventCommand::Topics { target, json, timeout, no_header } => commands::events::cmd_topics(target.as_deref(), json, timeout, no_header).await,
            EventCommand::Collect { targets, topic, interval, count, json, timeout, payload_only, since } => {
                let collect_opts = commands::events::CollectOpts { topic_filter: topic.as_deref(), interval_ms: interval, max_count: count, json, timeout_secs: timeout, payload_only, since };
                commands::events::cmd_collect(targets, collect_opts).await
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
        Command::Mirror { target, scrollback, raw, tag } => {
            if let Some(t) = tag {
                commands::mirror_grid_composer::cmd_mirror_tag(&t, scrollback).await
            } else {
                commands::pty::cmd_mirror(&resolve_target(target)?, scrollback, raw).await
            }
        }

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
        Command::Watch { targets, interval, topic, json, timeout, count, payload_only, since } => {
            let watch_opts = commands::events::WatchOpts { interval_ms: interval, topic_filter: topic.as_deref(), json, timeout_secs: timeout, max_count: count, payload_only, since };
            commands::events::cmd_watch(targets, watch_opts).await
        }
        Command::Topics { target, json, timeout, no_header } => commands::events::cmd_topics(target.as_deref(), json, timeout, no_header).await,
        Command::Collect { targets, topic, interval, count, json, timeout, payload_only, since } => {
            let collect_opts = commands::events::CollectOpts { topic_filter: topic.as_deref(), interval_ms: interval, max_count: count, json, timeout_secs: timeout, payload_only, since };
            commands::events::cmd_collect(targets, collect_opts).await
        }
        Command::Wait { target, topic, timeout, interval, json, since } => {
            commands::events::cmd_wait(&resolve_target(target)?, &topic, timeout, interval, json, since).await
        }

        // Metadata & Discovery
        Command::Tag { target, set, add, remove, new_name, role, add_role, remove_role, json, timeout, hub, secret_file, secret, scope } => {
            let tag_opts = commands::metadata::TagOpts { set, add, remove, new_name, role, add_role, remove_role };
            let tgt_opts = target::TargetOpts {
                hub,
                secret_file,
                secret,
                scope,
                session: target,
            };
            commands::metadata::cmd_tag(&tgt_opts, tag_opts, json, timeout).await
        }
        Command::Discover { tag, role, cap, name, json, count, first, wait, wait_timeout, id, names, ids, no_header } => {
            let display = ListDisplayOpts { count, first, names, ids, no_header, json };
            let opts = commands::metadata::DiscoverOpts { tags: tag, roles: role, caps: cap, name, wait, wait_timeout, id };
            commands::metadata::cmd_discover(opts, &display).await
        }
        Command::Whoami { session, name, json } => {
            commands::metadata::cmd_whoami(session, name, json).await
        }
        Command::Kv { target, json, timeout, raw, keys, hub, secret_file, secret, scope, action } => {
            let action = action.unwrap_or(KvAction::List);
            let opts = target::TargetOpts {
                hub,
                secret_file,
                secret,
                scope,
                session: target,
            };
            commands::metadata::cmd_kv(&opts, action, json, raw, keys, timeout).await
        }

        // Execution
        Command::Run { name, roles, tags, cap, timeout, json, command } => {
            commands::execution::cmd_run(name, roles, tags, cap, timeout, json, command).await
        }
        Command::Request { target, topic, payload, reply_topic, timeout, interval, json } => {
            commands::execution::cmd_request(&target, &topic, &payload, &reply_topic, timeout, interval, json).await
        }
        Command::Spawn { name, roles, tags, cap, env_vars, wait, wait_timeout, shell, backend, json, command } => {
            commands::execution::cmd_spawn(commands::execution::SpawnOpts { name, roles, tags, cap, env_vars, wait, wait_timeout, shell, backend, json, command }).await
        }
        Command::Dispatch { count, timeout, topic, name, roles, tags, cap, env_vars, backend, workdir, isolate, auto_merge, json, model, command } => {
            commands::dispatch::cmd_dispatch(commands::dispatch::DispatchOpts { count, timeout, topic, name_prefix: name, roles, tags, cap, env_vars, backend, workdir, isolate, auto_merge, json_output: json, command, model }).await
        }
        Command::DispatchStatus { check, json } => {
            commands::dispatch::cmd_dispatch_status(check, json)
        }

        // Infrastructure
        Command::Clean { dry_run, json, no_header, count } => commands::session::cmd_clean(dry_run, json, no_header, count),
        Command::Hub { action } => match action {
            None | Some(HubAction::Start { tcp: None, json: false }) => commands::infrastructure::cmd_hub_start(None, false).await,
            Some(HubAction::Start { tcp: None, json: true }) => commands::infrastructure::cmd_hub_start(None, true).await,
            Some(HubAction::Start { tcp: Some(ref addr), json }) => commands::infrastructure::cmd_hub_start(Some(addr), json).await,
            Some(HubAction::Stop { json }) => commands::infrastructure::cmd_hub_stop(json),
            Some(HubAction::Restart { json }) => commands::infrastructure::cmd_hub_restart(json),
            Some(HubAction::Status { json, short, check }) => commands::infrastructure::cmd_hub_status(json, short, check),
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
            AgentAction::Negotiate { specialist, schema, draft, from, max_rounds, timeout, interval, json } => {
                commands::agent::cmd_agent_negotiate(commands::agent::NegotiateOpts { specialist: &specialist, schema_str: &schema, draft_str: &draft, from: from.as_deref(), max_rounds, timeout, interval, json }).await
            }
        },
        Command::File { action } => match action {
            FileAction::Send { target, path, chunk_size, json, timeout } => {
                commands::file::cmd_file_send(&target, &path, chunk_size, json, timeout).await
            }
            FileAction::Receive { target, output_dir, timeout, interval, replay, json } => {
                commands::file::cmd_file_receive(&target, &output_dir, timeout, interval, replay, json).await
            }
        },
        Command::Remote { action } => match action {
            RemoteAction::Ping { hub, session, secret_file, secret, scope, json, timeout } => {
                let p = resolve_hub_profile(&hub, secret_file.as_deref(), secret.as_deref(), &scope)?;
                let conn = RemoteConn { hub: &p.address, secret_file: p.secret_file.as_deref(), secret_hex: p.secret.as_deref(), scope: p.scope.as_deref().unwrap_or("observe") };
                commands::remote::cmd_remote_ping(&conn, session.as_deref(), json, timeout).await
            }
            RemoteAction::List { hub, secret_file, secret, scope, name, tags, roles, cap, count, first, names, ids, no_header, json, timeout } => {
                let p = resolve_hub_profile(&hub, secret_file.as_deref(), secret.as_deref(), &scope)?;
                let conn = RemoteConn { hub: &p.address, secret_file: p.secret_file.as_deref(), secret_hex: p.secret.as_deref(), scope: p.scope.as_deref().unwrap_or("observe") };
                let display = ListDisplayOpts { count, first, names, ids, no_header, json };
                commands::remote::cmd_remote_list(&conn, name.as_deref(), tags.as_deref(), roles.as_deref(), cap.as_deref(), &display, timeout).await
            }
            RemoteAction::Status { hub, session, secret_file, secret, scope, json, short, timeout } => {
                let p = resolve_hub_profile(&hub, secret_file.as_deref(), secret.as_deref(), &scope)?;
                let conn = RemoteConn { hub: &p.address, secret_file: p.secret_file.as_deref(), secret_hex: p.secret.as_deref(), scope: p.scope.as_deref().unwrap_or("observe") };
                let session = commands::remote::resolve_remote_target(session, &conn).await?;
                commands::remote::cmd_remote_status(&conn, &session, json, short, timeout).await
            }
            RemoteAction::Inject { hub, session, text, secret_file, secret, enter, key, delay_ms, scope, json, timeout } => {
                let p = resolve_hub_profile(&hub, secret_file.as_deref(), secret.as_deref(), &scope)?;
                let conn = RemoteConn { hub: &p.address, secret_file: p.secret_file.as_deref(), secret_hex: p.secret.as_deref(), scope: p.scope.as_deref().unwrap_or("control") };
                let inject_opts = commands::remote::RemoteInjectOpts { session: &session, text: &text, enter, key: key.as_deref(), delay_ms, json, timeout_secs: timeout };
                commands::remote::cmd_remote_inject(&conn, &inject_opts).await
            }
            RemoteAction::SendFile { hub, session, path, secret_file, secret, chunk_size, scope, json, timeout } => {
                let p = resolve_hub_profile(&hub, secret_file.as_deref(), secret.as_deref(), &scope)?;
                let conn = RemoteConn { hub: &p.address, secret_file: p.secret_file.as_deref(), secret_hex: p.secret.as_deref(), scope: p.scope.as_deref().unwrap_or("control") };
                commands::remote::cmd_remote_send_file(&conn, &session, &path, chunk_size, json, timeout).await
            }
            RemoteAction::Events { hub, secret_file, secret, scope, topic, targets, interval, count, json, payload_only } => {
                let p = resolve_hub_profile(&hub, secret_file.as_deref(), secret.as_deref(), &scope)?;
                let conn = RemoteConn { hub: &p.address, secret_file: p.secret_file.as_deref(), secret_hex: p.secret.as_deref(), scope: p.scope.as_deref().unwrap_or("observe") };
                commands::remote::cmd_remote_events(&conn, topic.as_deref(), targets.as_deref(), interval, count, json, payload_only).await
            }
            RemoteAction::Exec { hub, session, command, secret_file, secret, scope, timeout, cwd, json } => {
                let p = resolve_hub_profile(&hub, secret_file.as_deref(), secret.as_deref(), &scope)?;
                let conn = RemoteConn { hub: &p.address, secret_file: p.secret_file.as_deref(), secret_hex: p.secret.as_deref(), scope: p.scope.as_deref().unwrap_or("execute") };
                commands::remote::cmd_remote_exec(&conn, &session, &command, timeout, cwd.as_deref(), json).await
            }
            RemoteAction::Push { hub, session, file, message, secret_file, secret, scope, json, timeout } => {
                let p = resolve_hub_profile(&hub, secret_file.as_deref(), secret.as_deref(), &scope)?;
                let conn = RemoteConn { hub: &p.address, secret_file: p.secret_file.as_deref(), secret_hex: p.secret.as_deref(), scope: p.scope.as_deref().unwrap_or("execute") };
                commands::push::cmd_push(&conn, &session, file.as_deref(), message.as_deref(), json, timeout).await
            }
            RemoteAction::Inbox { hub, action, secret_file, secret, scope, timeout } => {
                let action = action.unwrap_or(RemoteInboxAction::Status { json: false });
                let p = resolve_hub_profile(&hub, secret_file.as_deref(), secret.as_deref(), &scope)?;
                let conn = RemoteConn { hub: &p.address, secret_file: p.secret_file.as_deref(), secret_hex: p.secret.as_deref(), scope: p.scope.as_deref().unwrap_or("observe") };
                commands::remote::cmd_remote_inbox(&conn, action, timeout).await
            }
            RemoteAction::Doctor { hub, secret_file, secret, scope, json, timeout } => {
                let p = resolve_hub_profile(&hub, secret_file.as_deref(), secret.as_deref(), &scope)?;
                let conn = RemoteConn { hub: &p.address, secret_file: p.secret_file.as_deref(), secret_hex: p.secret.as_deref(), scope: p.scope.as_deref().unwrap_or("observe") };
                commands::remote::cmd_remote_doctor(&conn, json, timeout).await
            }
            RemoteAction::Profile { action } => {
                commands::remote::cmd_remote_profile(action)
            }
        },
        Command::Inbox { action } => match action {
            InboxAction::Status { json } => commands::infrastructure::cmd_inbox_status(json).await,
            InboxAction::List { target, json } => commands::infrastructure::cmd_inbox_list(&target, json).await,
            InboxAction::Clear { target, all, json } => commands::infrastructure::cmd_inbox_clear(target.as_deref(), all, json).await,
        },
        Command::Fleet { action } => match action.unwrap_or(FleetAction::Status { json: false, timeout: 10, verbose: false }) {
            FleetAction::Status { json, timeout, verbose } => {
                commands::remote::cmd_fleet_status(json, timeout, verbose).await
            }
            FleetAction::Doctor { json, timeout } => {
                commands::remote::cmd_fleet_doctor(json, timeout).await
            }
            FleetAction::Reauth { profile, bootstrap_from } => {
                commands::remote::cmd_fleet_reauth(&profile, bootstrap_from.as_deref())
            }
        },
        Command::Net { action } => match action {
            NetAction::Test { profile, json, timeout } => {
                commands::remote::cmd_net_test(profile.as_deref(), json, timeout).await
            }
        },
        Command::Tofu { action } => match action {
            TofuAction::List { json } => commands::infrastructure::cmd_tofu_list(json),
            TofuAction::Clear { host, all, json } => commands::infrastructure::cmd_tofu_clear(host.as_deref(), all, json),
        },
        Command::Identity { action } => match action {
            IdentityAction::Init { force, json } => commands::identity::cmd_identity_init(force, json),
            IdentityAction::Show { json } => commands::identity::cmd_identity_show(json),
            IdentityAction::Rotate { force, json } => commands::identity::cmd_identity_rotate(force, json),
        },
        Command::Channel { action } => match action {
            ChannelAction::Create { name, retention, hub, json } => {
                commands::channel::cmd_channel_create(&name, &retention, hub.as_deref(), json).await
            }
            ChannelAction::Post {
                topic,
                msg_type,
                payload,
                artifact_ref,
                sender_id,
                reply_to,
                metadata,
                mentions,
                hub,
                json,
            } => {
                let mut metadata = metadata;
                if !mentions.is_empty() {
                    metadata.push(format!("mentions={}", mentions.join(",")));
                }
                commands::channel::cmd_channel_post(
                    &topic,
                    &msg_type,
                    payload.as_deref(),
                    artifact_ref.as_deref(),
                    sender_id.as_deref(),
                    reply_to,
                    &metadata,
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::Dm {
                peer,
                send,
                reply_to,
                mentions,
                topic_only,
                list,
                unread,
                hub,
                json,
            } => {
                if list {
                    commands::channel::cmd_channel_dm_list(unread, hub.as_deref(), json).await
                } else {
                    commands::channel::cmd_channel_dm(
                        peer.as_deref().expect("clap required_unless_present guarantees peer when !list"),
                        send.as_deref(),
                        reply_to,
                        &mentions,
                        topic_only,
                        hub.as_deref(),
                        json,
                    )
                    .await
                }
            }
            ChannelAction::Ack {
                topic,
                up_to,
                since,
                sender_id,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_ack(
                    &topic,
                    up_to,
                    since,
                    sender_id.as_deref(),
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::Receipts { topic, hub, json } => {
                commands::channel::cmd_channel_receipts(&topic, hub.as_deref(), json).await
            }
            ChannelAction::Edit {
                topic,
                replaces,
                payload,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_edit(
                    &topic,
                    replaces,
                    &payload,
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::Info { topic, since, hub, json } => {
                commands::channel::cmd_channel_info(&topic, since, hub.as_deref(), json).await
            }
            ChannelAction::Reply {
                topic,
                payload,
                mention,
                sender_id,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_reply(
                    &topic,
                    &payload,
                    &mention,
                    sender_id.as_deref(),
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::Unread { topic, sender, hub, json } => {
                commands::channel::cmd_channel_unread(
                    &topic,
                    sender.as_deref(),
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::Thread {
                topic,
                root,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_thread(&topic, root, hub.as_deref(), json).await
            }
            ChannelAction::Ancestors {
                topic,
                offset,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_ancestors(&topic, offset, hub.as_deref(), json)
                    .await
            }
            ChannelAction::Members {
                topic,
                include_meta,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_members(
                    &topic,
                    include_meta,
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::Describe {
                topic,
                description,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_describe(
                    &topic,
                    &description,
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::Redact {
                topic,
                redacts,
                reason,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_redact(
                    &topic,
                    redacts,
                    reason.as_deref(),
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::React {
                topic,
                parent_offset,
                reaction,
                sender_id,
                remove,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_react(
                    &topic,
                    parent_offset,
                    &reaction,
                    sender_id.as_deref(),
                    remove,
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::Subscribe {
                topic,
                cursor,
                resume,
                reset,
                limit,
                follow,
                conversation_id,
                in_reply_to,
                reactions,
                by_sender,
                collapse_edits,
                hide_redacted,
                filter_mentions,
                since,
                until,
                show_parent,
                tail,
                senders,
                show_forwards,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_subscribe(
                    &topic,
                    cursor,
                    resume,
                    reset,
                    limit,
                    follow,
                    conversation_id.as_deref(),
                    in_reply_to,
                    reactions,
                    by_sender,
                    collapse_edits,
                    hide_redacted,
                    filter_mentions.as_deref(),
                    since,
                    until,
                    show_parent,
                    tail,
                    senders.as_deref(),
                    show_forwards,
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::Quote {
                topic,
                offset,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_quote(&topic, offset, hub.as_deref(), json).await
            }
            ChannelAction::Pin {
                topic,
                offset,
                unpin,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_pin(
                    &topic,
                    offset,
                    unpin,
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::Pinned { topic, hub, json } => {
                commands::channel::cmd_channel_pinned(&topic, hub.as_deref(), json).await
            }
            ChannelAction::Star {
                topic,
                offset,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_star(
                    &topic,
                    offset,
                    false,
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::Unstar {
                topic,
                offset,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_star(
                    &topic,
                    offset,
                    true,
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::Starred {
                topic,
                all,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_starred(
                    &topic,
                    all,
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::Digest {
                topic,
                since_mins,
                since,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_digest(
                    &topic,
                    since_mins,
                    since,
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::Poll { action } => match action {
                PollAction::Start {
                    topic,
                    question,
                    options,
                    hub,
                    json,
                } => {
                    commands::channel::cmd_channel_poll_start(
                        &topic,
                        &question,
                        &options,
                        hub.as_deref(),
                        json,
                    )
                    .await
                }
                PollAction::Vote {
                    topic,
                    poll_id,
                    choice,
                    hub,
                    json,
                } => {
                    commands::channel::cmd_channel_poll_vote(
                        &topic,
                        poll_id,
                        choice,
                        hub.as_deref(),
                        json,
                    )
                    .await
                }
                PollAction::End {
                    topic,
                    poll_id,
                    hub,
                    json,
                } => {
                    commands::channel::cmd_channel_poll_end(
                        &topic,
                        poll_id,
                        hub.as_deref(),
                        json,
                    )
                    .await
                }
                PollAction::Results {
                    topic,
                    poll_id,
                    hub,
                    json,
                } => {
                    commands::channel::cmd_channel_poll_results(
                        &topic,
                        poll_id,
                        hub.as_deref(),
                        json,
                    )
                    .await
                }
            },
            ChannelAction::Typing {
                topic,
                emit,
                ttl_ms,
                hub,
                json,
            } => {
                if emit {
                    commands::channel::cmd_channel_typing_emit(
                        &topic,
                        ttl_ms,
                        hub.as_deref(),
                        json,
                    )
                    .await
                } else {
                    commands::channel::cmd_channel_typing_list(
                        &topic,
                        hub.as_deref(),
                        json,
                    )
                    .await
                }
            }
            ChannelAction::Forward {
                src_topic,
                offset,
                dst_topic,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_forward(
                    &src_topic,
                    offset,
                    &dst_topic,
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::List { prefix, stats, hub, json } => {
                commands::channel::cmd_channel_list(prefix.as_deref(), stats, hub.as_deref(), json).await
            }
            ChannelAction::Mentions {
                target,
                prefix,
                limit,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_mentions(
                    target.as_deref(),
                    prefix.as_deref(),
                    limit,
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::Search {
                topic,
                pattern,
                regex,
                case_sensitive,
                all,
                limit,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_search(
                    &topic,
                    &pattern,
                    regex,
                    case_sensitive,
                    all,
                    limit,
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::QueueStatus { queue_path, json } => {
                commands::channel::cmd_channel_queue_status(queue_path.as_deref(), json)
            }
        },
        Command::Doctor { json, fix, strict, runtime_dir } => {
            if let Some(ref dir) = runtime_dir {
                unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", dir) };
            }
            commands::infrastructure::cmd_doctor(json, fix, strict).await
        },
        Command::Vendor { action, source, target, dry_run, json } => {
            if let Some(action) = action {
                match action {
                    VendorAction::Status { target, json, check } => commands::vendor::cmd_vendor_status(target.as_deref(), json, check),
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
        Command::Version { json, short } => {
            let version = env!("CARGO_PKG_VERSION");
            let commit = option_env!("GIT_COMMIT").unwrap_or("unknown");
            let target = option_env!("BUILD_TARGET").unwrap_or("unknown");
            let mcp_tools = termlink_mcp::tool_count();

            if short {
                println!("{version}");
            } else if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "ok": true,
                        "version": version,
                        "commit": commit,
                        "target": target,
                        "mcp_tools": mcp_tools,
                    })
                );
            } else {
                println!("termlink {version} ({commit}) [{target}] — {mcp_tools} MCP tools");
            }
            Ok(())
        }
    }
}
