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
        Command::Register { name, roles, tags, cap, shell, self_mode, token_secret, allowed_commands, json, quiet, identity_key } => {
            if self_mode {
                commands::session::cmd_register_self(name, roles, tags, cap, json, identity_key).await
            } else {
                commands::session::cmd_register(commands::session::RegisterOpts { name, roles, tags, cap, shell, enable_token_secret: token_secret, allowed_commands, json, quiet, identity_key }).await
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
            EventCommand::Watch { targets, hub, interval, topic, json, timeout, count, payload_only, since } => {
                let watch_opts = commands::events::WatchOpts { interval_ms: interval, topic_filter: topic.as_deref(), json, timeout_secs: timeout, max_count: count, payload_only, since };
                if hub {
                    commands::events::cmd_watch_hub(watch_opts).await
                } else {
                    commands::events::cmd_watch(targets, watch_opts).await
                }
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
            Some(HubAction::ExportSecret { out, json }) => commands::infrastructure::cmd_hub_export_secret(out.as_deref(), json),
            Some(HubAction::Fingerprint { json }) => commands::infrastructure::cmd_hub_fingerprint(json),
            Some(HubAction::Probe { addr, json }) => commands::infrastructure::cmd_hub_probe(&addr, json).await,
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
            AgentAction::Contact { target, target_fp, message, file, thread, hub, json, dry_run, require_online, online_window_secs, ack_required, ack_timeout_secs } => {
                let resolved_message = commands::agent::resolve_contact_message(message.as_deref(), file.as_deref())?;
                commands::agent::cmd_agent_contact(target.as_deref(), target_fp.as_deref(), &resolved_message, thread.as_deref(), hub.as_deref(), json, dry_run, require_online, online_window_secs, ack_required, ack_timeout_secs).await
            }
            AgentAction::Who { target_fp, target, window_secs, hub, json, filter_thread } => {
                commands::agent::cmd_agent_who(target_fp.as_deref(), target.as_deref(), window_secs, hub.as_deref(), json, filter_thread.as_deref()).await
            }
            AgentAction::Presence { window_secs, hub, json, filter_project, filter_thread, watch, watch_interval, top, by_project } => {
                commands::agent::cmd_agent_presence(window_secs, hub.as_deref(), json, filter_project.as_deref(), filter_thread.as_deref(), watch, watch_interval, top, by_project).await
            }
            AgentAction::Ping { target, target_fp, window_secs, hub, json } => {
                commands::agent::cmd_agent_ping(target.as_deref(), target_fp.as_deref(), window_secs, hub.as_deref(), json).await
            }
            AgentAction::Recent { target, target_fp, n, window_secs, filter_thread, filter_project, filter_msg_types, filter_grep, hub, json, watch, watch_interval, depth } => {
                let mt: Vec<&str> = filter_msg_types.iter().map(String::as_str).collect();
                let mt_opt = if mt.is_empty() { None } else { Some(mt.as_slice()) };
                commands::agent::cmd_agent_recent(target.as_deref(), target_fp.as_deref(), n, window_secs, filter_thread.as_deref(), filter_project.as_deref(), mt_opt, filter_grep.as_deref(), hub.as_deref(), json, watch, watch_interval, depth).await
            }
            AgentAction::OnThread { thread, n, window_secs, filter_project, filter_msg_types, filter_grep, peer, peer_fp, hub, json, watch, watch_interval, depth } => {
                let mt: Vec<&str> = filter_msg_types.iter().map(String::as_str).collect();
                let mt_opt = if mt.is_empty() { None } else { Some(mt.as_slice()) };
                commands::agent::cmd_agent_on_thread(&thread, n, window_secs, filter_project.as_deref(), mt_opt, filter_grep.as_deref(), peer.as_deref(), peer_fp.as_deref(), hub.as_deref(), json, watch, watch_interval, depth).await
            }
            AgentAction::Overview { window_secs, top, hub, json, depth, watch, watch_interval } => {
                commands::agent::cmd_agent_overview(window_secs, top, hub.as_deref(), json, watch, watch_interval, depth).await
            }
            AgentAction::Timeline { n, window_secs, filter_thread, filter_project, filter_msg_types, filter_grep, hub, json, watch, watch_interval, depth } => {
                let mt: Vec<&str> = filter_msg_types.iter().map(String::as_str).collect();
                let mt_opt = if mt.is_empty() { None } else { Some(mt.as_slice()) };
                commands::agent::cmd_agent_timeline(n, window_secs, filter_thread.as_deref(), filter_project.as_deref(), mt_opt, filter_grep.as_deref(), hub.as_deref(), json, watch, watch_interval, depth).await
            }
            AgentAction::Post { text, thread, project, msg_type, hub, json } => {
                commands::agent::cmd_agent_post(&text, thread.as_deref(), project.as_deref(), &msg_type, hub.as_deref(), json).await
            }
            AgentAction::Stats { window_secs, top, hub, json } => {
                commands::agent::cmd_agent_stats(window_secs, top, hub.as_deref(), json).await
            }
            AgentAction::Quote { offset, hub, json } => {
                commands::channel::cmd_channel_quote("agent-chat-arc", offset, hub.as_deref(), json).await
            }
            AgentAction::Reply { offset, text, thread, project, msg_type, hub, json } => {
                commands::agent::cmd_agent_reply(offset, &text, thread.as_deref(), project.as_deref(), &msg_type, hub.as_deref(), json).await
            }
            AgentAction::Search { query, n, hub, json } => {
                commands::agent::cmd_agent_search(&query, n, hub.as_deref(), json).await
            }
            AgentAction::Thread { root, hub, json } => {
                commands::channel::cmd_channel_thread("agent-chat-arc", root, hub.as_deref(), json).await
            }
            AgentAction::Ancestors { offset, hub, json } => {
                commands::channel::cmd_channel_ancestors("agent-chat-arc", offset, hub.as_deref(), json).await
            }
            AgentAction::Digest { since_mins, since, hub, json } => {
                commands::channel::cmd_channel_digest("agent-chat-arc", since_mins, since, hub.as_deref(), json).await
            }
            AgentAction::Unread { sender, hub, json, watch, watch_interval } => {
                if watch && json {
                    anyhow::bail!(
                        "--watch and --json are incompatible: --watch streams \
                         re-rendered text frames; --json is one-shot. Pick one."
                    );
                }
                if watch {
                    let clamped = watch_interval.clamp(1, 300);
                    loop {
                        print!("\x1b[2J\x1b[H");
                        let now_secs = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0);
                        let now_str = manifest::secs_to_rfc3339(now_secs);
                        println!(
                            "# agent unread --watch | interval={}s | {}",
                            clamped, now_str
                        );
                        if let Err(e) = commands::channel::cmd_channel_unread(
                            "agent-chat-arc", sender.as_deref(), hub.as_deref(), false,
                        ).await {
                            println!("# fetch error (will retry next tick): {e}");
                        }
                        tokio::time::sleep(std::time::Duration::from_secs(clamped)).await;
                    }
                } else {
                    commands::channel::cmd_channel_unread("agent-chat-arc", sender.as_deref(), hub.as_deref(), json).await
                }
            }
            AgentAction::Mentions { user, hub, json } => {
                commands::channel::cmd_channel_mentions_of("agent-chat-arc", &user, hub.as_deref(), json).await
            }
            AgentAction::Reactions { offset, hub, json } => {
                commands::channel::cmd_channel_reactions_on("agent-chat-arc", offset, hub.as_deref(), json).await
            }
            AgentAction::EmojiStats { by_sender, top, hub, json } => {
                commands::channel::cmd_channel_emoji_stats("agent-chat-arc", by_sender, top, hub.as_deref(), json).await
            }
            AgentAction::TopicStats { hub, json } => {
                commands::channel::cmd_channel_topic_stats("agent-chat-arc", hub.as_deref(), json).await
            }
            AgentAction::Pinned { hub, json } => {
                commands::channel::cmd_channel_pinned("agent-chat-arc", hub.as_deref(), json).await
            }
            AgentAction::Starred { all, hub, json } => {
                commands::channel::cmd_channel_starred("agent-chat-arc", all, hub.as_deref(), json).await
            }
            AgentAction::Snippet { offset, lines, header, hub, json } => {
                commands::channel::cmd_channel_snippet("agent-chat-arc", offset, lines, header, hub.as_deref(), json).await
            }
            AgentAction::Peers { include_meta, as_of, hub, json } => {
                commands::channel::cmd_channel_members("agent-chat-arc", include_meta, as_of, hub.as_deref(), json).await
            }
            AgentAction::ReactionsOf { sender, hub, json } => {
                commands::channel::cmd_channel_reactions_of("agent-chat-arc", sender.as_deref(), hub.as_deref(), json).await
            }
            AgentAction::ForwardsOf { sender, hub, json } => {
                commands::channel::cmd_channel_forwards_of("agent-chat-arc", sender.as_deref(), hub.as_deref(), json).await
            }
            AgentAction::RepliesOf { sender, hub, json } => {
                commands::channel::cmd_channel_replies_of("agent-chat-arc", sender.as_deref(), hub.as_deref(), json).await
            }
            AgentAction::Info { since, hub, json } => {
                commands::channel::cmd_channel_info("agent-chat-arc", since, hub.as_deref(), json).await
            }
            AgentAction::React { offset, emoji, remove, hub, json } => {
                commands::channel::cmd_channel_react("agent-chat-arc", offset, &emoji, None, remove, hub.as_deref(), json).await
            }
            AgentAction::Ack { up_to, since_ms, hub, json } => {
                commands::channel::cmd_channel_ack("agent-chat-arc", up_to, since_ms, None, hub.as_deref(), json).await
            }
            AgentAction::Pin { offset, unpin, hub, json } => {
                commands::channel::cmd_channel_pin("agent-chat-arc", offset, unpin, hub.as_deref(), json).await
            }
            AgentAction::Star { offset, unstar, hub, json } => {
                commands::channel::cmd_channel_star("agent-chat-arc", offset, unstar, hub.as_deref(), json).await
            }
            AgentAction::Forward { offset, to, hub, json } => {
                commands::channel::cmd_channel_forward("agent-chat-arc", offset, &to, hub.as_deref(), json).await
            }
            AgentAction::Edit { offset, text, hub, json } => {
                commands::channel::cmd_channel_edit("agent-chat-arc", offset, &text, hub.as_deref(), json).await
            }
            AgentAction::Redact { offset, reason, hub, json } => {
                commands::channel::cmd_channel_redact("agent-chat-arc", offset, reason.as_deref(), hub.as_deref(), json).await
            }
            AgentAction::Describe { text, hub, json } => {
                commands::channel::cmd_channel_describe("agent-chat-arc", &text, hub.as_deref(), json).await
            }
            AgentAction::Threads { top, hub, json } => {
                commands::channel::cmd_channel_threads("agent-chat-arc", top, hub.as_deref(), json).await
            }
            AgentAction::Redactions { hub, json } => {
                commands::channel::cmd_channel_redactions("agent-chat-arc", hub.as_deref(), json).await
            }
            AgentAction::PinHistory { hub, json } => {
                commands::channel::cmd_channel_pin_history("agent-chat-arc", hub.as_deref(), json).await
            }
            AgentAction::EditsOf { offset, hub, json } => {
                commands::channel::cmd_channel_edits_of("agent-chat-arc", offset, hub.as_deref(), json).await
            }
            AgentAction::Relations { offset, hub, json } => {
                commands::channel::cmd_channel_relations("agent-chat-arc", offset, hub.as_deref(), json).await
            }
            AgentAction::AckHistory { user, hub, json } => {
                commands::channel::cmd_channel_ack_history("agent-chat-arc", user.as_deref(), hub.as_deref(), json).await
            }
            AgentAction::AckStatus { pending_only, hub, json } => {
                commands::channel::cmd_channel_ack_status("agent-chat-arc", pending_only, hub.as_deref(), json).await
            }
            AgentAction::State { include_redacted, hub, json } => {
                commands::channel::cmd_channel_state("agent-chat-arc", include_redacted, hub.as_deref(), json).await
            }
            AgentAction::QuoteStats { hub, json } => {
                commands::channel::cmd_channel_quote_stats("agent-chat-arc", hub.as_deref(), json).await
            }
            AgentAction::EditStats { hub, json } => {
                commands::channel::cmd_channel_edit_stats("agent-chat-arc", hub.as_deref(), json).await
            }
            AgentAction::PollStart { question, option, hub, json } => {
                commands::channel::cmd_channel_poll_start("agent-chat-arc", &question, &option, hub.as_deref(), json).await
            }
            AgentAction::Vote { poll_id, choice, hub, json } => {
                commands::channel::cmd_channel_poll_vote("agent-chat-arc", poll_id, choice, hub.as_deref(), json).await
            }
            AgentAction::PollEnd { poll_id, hub, json } => {
                commands::channel::cmd_channel_poll_end("agent-chat-arc", poll_id, hub.as_deref(), json).await
            }
            AgentAction::PollResults { poll_id, hub, json } => {
                commands::channel::cmd_channel_poll_results("agent-chat-arc", poll_id, hub.as_deref(), json).await
            }
            AgentAction::Snapshot { as_of, include_redacted, hub, json } => {
                commands::channel::cmd_channel_snapshot("agent-chat-arc", as_of, include_redacted, hub.as_deref(), json).await
            }
            AgentAction::StateSince { since, include_redacted, hub, json } => {
                commands::channel::cmd_channel_state_since("agent-chat-arc", since, include_redacted, hub.as_deref(), json).await
            }
            AgentAction::SnapshotDiff { from, to, include_redacted, include_unchanged, hub, json } => {
                commands::channel::cmd_channel_snapshot_diff("agent-chat-arc", from, to, include_redacted, include_unchanged, hub.as_deref(), json).await
            }
            AgentAction::Typing { ttl_ms, hub, json } => {
                commands::channel::cmd_channel_typing_emit("agent-chat-arc", ttl_ms, hub.as_deref(), json).await
            }
            AgentAction::Typers { hub, json, watch, watch_interval } => {
                if watch && json {
                    anyhow::bail!(
                        "--watch and --json are incompatible: --watch streams \
                         re-rendered text frames; --json is one-shot. Pick one."
                    );
                }
                if watch {
                    let clamped = watch_interval.clamp(1, 60);
                    loop {
                        // ANSI: clear screen + cursor home, matching the
                        // existing presence/recent --watch UX (T-1486/T-1498).
                        print!("\x1b[2J\x1b[H");
                        let now_secs = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0);
                        let now_str = manifest::secs_to_rfc3339(now_secs);
                        println!(
                            "# agent typers --watch | interval={}s | {}",
                            clamped, now_str
                        );
                        // Per-iteration fetch + render. Errors are non-fatal:
                        // a transient hub blip should not kill the dashboard.
                        if let Err(e) = commands::channel::cmd_channel_typing_list(
                            "agent-chat-arc", hub.as_deref(), false,
                        ).await {
                            println!("# fetch error (will retry next tick): {e}");
                        }
                        tokio::time::sleep(std::time::Duration::from_secs(clamped)).await;
                    }
                } else {
                    commands::channel::cmd_channel_typing_list("agent-chat-arc", hub.as_deref(), json).await
                }
            }
            AgentAction::Dms { unread, hub, json, watch, watch_interval } => {
                if watch && json {
                    anyhow::bail!(
                        "--watch and --json are incompatible: --watch streams \
                         re-rendered text frames; --json is one-shot. Pick one."
                    );
                }
                if watch {
                    let clamped = watch_interval.clamp(1, 300);
                    loop {
                        print!("\x1b[2J\x1b[H");
                        let now_secs = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0);
                        let now_str = manifest::secs_to_rfc3339(now_secs);
                        println!(
                            "# agent dms --watch | interval={}s | {}",
                            clamped, now_str
                        );
                        if let Err(e) = commands::channel::cmd_channel_dm_list(unread, hub.as_deref(), false).await {
                            println!("# fetch error (will retry next tick): {e}");
                        }
                        tokio::time::sleep(std::time::Duration::from_secs(clamped)).await;
                    }
                } else {
                    commands::channel::cmd_channel_dm_list(unread, hub.as_deref(), json).await
                }
            }
            AgentAction::Inbox { hub, json, watch, watch_interval } => {
                if watch && json {
                    anyhow::bail!(
                        "--watch and --json are incompatible: --watch streams \
                         re-rendered text frames; --json is one-shot. Pick one."
                    );
                }
                if watch {
                    let clamped = watch_interval.clamp(1, 300);
                    loop {
                        print!("\x1b[2J\x1b[H");
                        let now_secs = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0);
                        let now_str = manifest::secs_to_rfc3339(now_secs);
                        println!(
                            "# agent inbox --watch | interval={}s | {}",
                            clamped, now_str
                        );
                        if let Err(e) = commands::channel::cmd_channel_inbox(hub.as_deref(), false).await {
                            println!("# fetch error (will retry next tick): {e}");
                        }
                        tokio::time::sleep(std::time::Duration::from_secs(clamped)).await;
                    }
                } else {
                    commands::channel::cmd_channel_inbox(hub.as_deref(), json).await
                }
            }
            AgentAction::Identity { json } => {
                commands::identity::cmd_identity_show(json)
            }
            AgentAction::Verbs => {
                print_agent_help();
                Ok(())
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
            FleetAction::Doctor { json, timeout, legacy_usage, legacy_window_days, topic_durability, include_pin_check, diff, save_snapshot, exit_code_on_verdict, trend, trend_keep, top_callers, watch, notify, auto_heal, dry_run } => {
                commands::remote::cmd_fleet_doctor(json, timeout, legacy_usage, legacy_window_days, topic_durability, include_pin_check, diff, save_snapshot, exit_code_on_verdict, trend, trend_keep, top_callers, watch, notify, auto_heal, dry_run).await
            }
            FleetAction::Reauth { profile, bootstrap_from, all_drifted, json } => {
                if all_drifted {
                    commands::remote::cmd_fleet_reauth_all().await
                } else {
                    match profile {
                        Some(p) => commands::remote::cmd_fleet_reauth(&p, bootstrap_from.as_deref(), json),
                        None => anyhow::bail!(
                            "fleet reauth: specify a profile name or --all-drifted (see --help)"
                        ),
                    }
                }
            }
            FleetAction::Verify { json, exit_on_drift_only } => {
                commands::remote::cmd_fleet_verify(json, exit_on_drift_only).await
            }
            FleetAction::History { since, hub, json, include_heals, analyze } => {
                commands::remote::cmd_fleet_history(since, hub.as_deref(), json, include_heals, analyze)
            }
            FleetAction::BootstrapCheck { profile, all, json } => {
                commands::remote::cmd_fleet_bootstrap_check(profile.as_deref(), all, json)
            }
            FleetAction::SecretsAudit {
                dir,
                check_drift,
                target_cache,
                json,
            } => commands::remote::cmd_fleet_secrets_audit(
                dir.as_deref(),
                check_drift.as_deref(),
                target_cache.as_deref(),
                json,
            ),
        },
        Command::Net { action } => match action {
            NetAction::Test { profile, json, timeout } => {
                commands::remote::cmd_net_test(profile.as_deref(), json, timeout).await
            }
        },
        Command::Tofu { action } => match action {
            TofuAction::List { json } => commands::infrastructure::cmd_tofu_list(json),
            TofuAction::Clear { host, all, json } => commands::infrastructure::cmd_tofu_clear(host.as_deref(), all, json),
            TofuAction::Verify { host, json } => commands::infrastructure::cmd_tofu_verify(&host, json).await,
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
                ensure_topic,
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
                    ensure_topic,
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
                        &[],   // T-1429 Phase-2: extra_metadata — `channel dm` has no flag for it yet
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
                as_of,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_members(
                    &topic,
                    include_meta,
                    as_of,
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
            ChannelAction::Inbox { hub, json } => {
                commands::channel::cmd_channel_inbox(hub.as_deref(), json).await
            }
            ChannelAction::Snippet {
                topic,
                offset,
                lines,
                header,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_snippet(
                    &topic,
                    offset,
                    lines,
                    header,
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::ReactionsOf {
                topic,
                sender,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_reactions_of(
                    &topic,
                    sender.as_deref(),
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::AckStatus {
                topic,
                pending_only,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_ack_status(
                    &topic,
                    pending_only,
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::EmojiStats {
                topic,
                by_sender,
                top,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_emoji_stats(
                    &topic,
                    by_sender,
                    top,
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::TopicStats { topic, hub, json } => {
                commands::channel::cmd_channel_topic_stats(
                    &topic,
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::ForwardsOf {
                topic,
                sender,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_forwards_of(
                    &topic,
                    sender.as_deref(),
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::RepliesOf {
                topic,
                sender,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_replies_of(
                    &topic,
                    sender.as_deref(),
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::MentionsOf {
                topic,
                user,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_mentions_of(
                    &topic,
                    &user,
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::PinHistory { topic, hub, json } => {
                commands::channel::cmd_channel_pin_history(
                    &topic,
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::Redactions { topic, hub, json } => {
                commands::channel::cmd_channel_redactions(
                    &topic,
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::ReactionsOn {
                topic,
                offset,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_reactions_on(
                    &topic,
                    offset,
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::EditStats { topic, hub, json } => {
                commands::channel::cmd_channel_edit_stats(
                    &topic,
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::State {
                topic,
                include_redacted,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_state(
                    &topic,
                    include_redacted,
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::StateSince {
                topic,
                since_ms,
                include_redacted,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_state_since(
                    &topic,
                    since_ms,
                    include_redacted,
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::SnapshotDiff {
                topic,
                from_ms,
                to_ms,
                include_redacted,
                include_unchanged,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_snapshot_diff(
                    &topic,
                    from_ms,
                    to_ms,
                    include_redacted,
                    include_unchanged,
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::AckHistory {
                topic,
                user,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_ack_history(
                    &topic,
                    user.as_deref(),
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::Snapshot {
                topic,
                as_of,
                include_redacted,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_snapshot(
                    &topic,
                    as_of,
                    include_redacted,
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::QuoteStats { topic, hub, json } => {
                commands::channel::cmd_channel_quote_stats(
                    &topic,
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::Relations {
                topic,
                offset,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_relations(
                    &topic,
                    offset,
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::EditsOf {
                topic,
                offset,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_edits_of(
                    &topic,
                    offset,
                    hub.as_deref(),
                    json,
                )
                .await
            }
            ChannelAction::Threads {
                topic,
                top,
                hub,
                json,
            } => {
                commands::channel::cmd_channel_threads(
                    &topic,
                    top,
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
        Command::Help {
            target,
            json,
            category,
            name_filter,
            list_categories,
            tool_detail,
            summary,
            essentials,
            max_parameters,
            min_parameters,
            exclude_deprecated,
            deprecated_only,
            limit,
            offset,
            sort_by,
            fields,
            categories,
            exclude_categories,
        } => {
            commands::help::run(commands::help::HelpInvocation {
                target,
                json,
                category,
                name_filter,
                list_categories,
                tool_detail,
                summary,
                essentials,
                max_parameters,
                min_parameters,
                exclude_deprecated,
                deprecated_only,
                limit,
                offset,
                sort_by,
                fields,
                categories,
                exclude_categories,
            })
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

/// T-1556: Categorized index of `agent.*` verbs grouped by purpose.
/// `agent --help` is clap's flat-alphabetical listing; this is the
/// operator-discoverable directory at >60 verbs.
fn print_agent_help() {
    let sections: &[(&str, &[(&str, &str)])] = &[
        ("READING (chat-arc views)", &[
            ("recent <peer>", "last N posts from a peer"),
            ("on-thread <T-XXX>", "all posts on a thread across peers"),
            ("timeline", "fleet-wide chronological log (tail -f for fleet)"),
            ("search <query>", "full-arc substring lookup, unbounded by window"),
            ("snippet <offset>", "windowed context around an offset"),
            ("threads", "list all thread roots"),
            ("thread <root-offset>", "render full reply subtree"),
            ("ancestors <offset>", "walk up in_reply_to chain to root"),
            ("relations <offset>", "all relations of a post"),
            ("redactions", "list all retracted posts"),
            ("pinned", "list pinned posts"),
            ("starred", "list starred posts"),
            ("quote <offset>", "fetch single post by offset"),
            ("digest", "period summary"),
            ("overview", "single-shot fleet digest"),
            ("info", "topic metadata + counts"),
            ("state", "current reduced state"),
            ("members", "list participating identities"),
            ("listen", "subscribe stream"),
        ]),
        ("WRITING (chat-arc emit)", &[
            ("post <text>", "focus-aware post"),
            ("reply <offset> <text>", "threaded write"),
            ("edit <offset> <text>", "edit a post"),
            ("redact <offset>", "retract a post"),
            ("react <offset> <emoji>", "emit reaction"),
            ("ack <offset>", "explicit receipt"),
            ("pin <offset>", "pin a post (--unpin to undo)"),
            ("star <offset>", "star a post (--unstar to undo)"),
            ("forward <offset> --to <topic>", "re-publish elsewhere"),
            ("describe <text>", "set topic metadata"),
        ]),
        ("PRESENCE (who-and-where)", &[
            ("who [--target <name>]", "peer observability primitive"),
            ("presence [--watch]", "fleet-wide peer activity summary"),
            ("ping <target>", "operator-facing presence check"),
            ("peers", "fleet directory of every chat-arc participant"),
            ("contact <name>", "high-level cross-host contact verb"),
            ("typing", "emit typing indicator (default ttl 5s)"),
            ("typers", "list active typers right now"),
        ]),
        ("STATS (analytics)", &[
            ("stats", "fleet-wide aggregate counts"),
            ("topic-stats", "lifetime structural breakdown"),
            ("emoji-stats", "fleet-wide emoji reaction counts"),
            ("quote-stats", "per-offset quote counts"),
            ("edit-stats", "edit-rate analytics"),
            ("reactions <offset>", "reactions on a post"),
            ("reactions-of [--sender]", "reactions emitted by an identity"),
            ("forwards-of [--sender]", "forwards emitted by an identity"),
            ("replies-of [--sender]", "replies authored by an identity"),
            ("edits-of <offset>", "edit history of a post"),
            ("mentions <user>", "find references"),
            ("ack-history", "receipt log"),
            ("ack-status", "current ack frontiers per sender"),
            ("pin-history", "pin/unpin event log"),
        ]),
        ("POLLS (collaborative decision)", &[
            ("poll-start <q> <opts>", "open a poll"),
            ("vote <poll-id> <choice>", "cast a vote"),
            ("poll-end <poll-id>", "close a poll"),
            ("poll-results <poll-id>", "render tally"),
        ]),
        ("SNAPSHOTS (point-in-time)", &[
            ("snapshot --as-of <ts>", "point-in-time state"),
            ("state-since --since <ts>", "envelopes since timestamp"),
            ("snapshot-diff --from --to", "state delta"),
        ]),
        ("PERSONAL (per-identity, beyond chat-arc)", &[
            ("identity", "show local FP + display name"),
            ("dms [--unread]", "list my DM topics + unread"),
            ("inbox", "unread counts across all subscribed topics"),
            ("unread", "count new posts on chat-arc since my last ack"),
        ]),
        ("META", &[
            ("help", "this categorized index"),
            ("ask <question>", "RPC ask peer"),
            ("negotiate", "capability negotiation"),
        ]),
    ];

    println!("agent.* — categorized verb index");
    println!("Use `agent <verb> --help` for full flag listing on any verb.");
    println!();
    for (heading, rows) in sections {
        println!("{heading}");
        for (verb, blurb) in rows.iter() {
            println!("  {verb:<32}  {blurb}");
        }
        println!();
    }
    println!("Surface: {} verbs across {} categories.",
        sections.iter().map(|(_, r)| r.len()).sum::<usize>(),
        sections.len());
}
