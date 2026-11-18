#!/usr/bin/env python3
"""T-3704 (arc-011 P-16, B-6 slice A): regenerate the unit's mutant patches (run from the repo root).

Each mutant breaks one rule and names the test that must go red; scripts/arc011-mutants.sh runs them:
  ARC011_MUTANTS_DIR=crates/termlink-hub/tests/mutants ARC011_MUTANTS_PACKAGE=termlink-hub \
    bash scripts/arc011-mutants.sh --report <file>
(the directory also holds T-3366's patches; `--only t3704-` style filtering is by name prefix).
A source edit that changes a mutated line makes this script fail loudly (the old text is not found).
"""
import os, subprocess, sys, tempfile

KC = 'crates/termlink-hub/src/key_card.rs'
CH = 'crates/termlink-hub/src/channel.rs'
RT = 'crates/termlink-hub/src/router.rs'
RA = 'crates/termlink-hub/src/rpc_audit.rs'
W = 'crates/termlink-sidecar/src/wire.rs'
LIB = '--lib key_card::tests::'
L4 = '--test t3704_key_card e69_posts_are_checked_against_the_card_and_e169_no_card_is_refused'
L4C = '--test t3704_key_card e169_a_corrupt_card_store_refuses_every_gated_post'
RTT = '--lib router::tests::t3704_key_card_writes_are_refused_off_the_local_socket'
WT = '--lib wire::wire_tests::t3704_named_roster_binds_from_agent_to_its_key'

M = [
 ('t3704-m1-fc-01-verify-accepts-any-signature', 'FCW-133 / E-69.5', 'the card key is not compared with the signer (plan control fc-01)', KC,
  [('        Some(k) if &k == signer => Check::Pass { agent },', '        Some(_) => Check::Pass { agent },\n        #[allow(unreachable_patterns)]\n        Some(k) if &k == signer => Check::Pass { agent },')],
  'termlink-hub', LIB + 'e69_fcw133_signed_by_a_sibling_key_is_refused_own_key_passes'),
 ('t3704-m2-no-card-passes', 'FCW-135 / E-169', 'a poster without a card passes', KC,
  [('        return refuse_no_card(&agent, format!("no key card for agent {agent:?}"));', '        return Check::Pass { agent };')],
  'termlink-hub', LIB + 'e169_fcw135_a_poster_without_a_card_is_refused'),
 ('t3704-m3-no-name-check', 'design v2 C-M1', 'from_agent is used without the plain-name check', KC,
  [('        Some(a) if valid_name(&a) => a,', '        Some(a) if !a.is_empty() => a,')],
  'termlink-hub', LIB + 'a_traversal_name_is_refused_and_never_read_as_a_path'),
 ('t3704-m4-corrupt-store-as-empty', 'design v2 O-M3', 'a corrupt card store reads as no cards', KC,
  [('    let s: Store = serde_json::from_slice(&bytes).map_err(|e| format!("key-card store corrupt: {e}"))?;',
    '    let s: Store = serde_json::from_slice(&bytes).unwrap_or(Store { schema: STORE_SCHEMA.into(), cards: BTreeMap::new() });')],
  'termlink-hub', LIB + 'an_unreadable_store_refuses_never_reads_as_empty'),
 ('t3704-m5-replace-not-required', 'design v2 C-3', 'a card key or project changes without replace', KC,
  [('        && !req.replace\n', '        && false\n')],
  'termlink-hub', LIB + 'a_key_or_project_change_needs_replace_and_seq_rises'),
 ('t3704-m6-bad-setting-is-off', 'design v2 (fail closed)', 'an unreadable or invalid setting reads as off', KC,
  [('    read_mode(dir).unwrap_or(Mode::Enforce)', '    read_mode(dir).unwrap_or(Mode::Off)')],
  'termlink-hub', LIB + 'mode_defaults_off_and_a_bad_setting_enforces'),
 ('t3704-m7-enforce-stores', 'FCW-133 at the hub', 'in enforce mode a refused post is stored anyway', CH,
  [('    if !enforced {\n        return None;\n    }\n    Some(\n        ErrorResponse::with_data(\n            id.clone(),\n            code,',
    '    if enforced || !enforced {\n        return None;\n    }\n    Some(\n        ErrorResponse::with_data(\n            id.clone(),\n            code,')],
  'termlink', L4),
 ('t3704-m8-gate-every-type', 'NID', 'the check also gates other message types and topics', KC,
  [('    topic.starts_with("inbox:") && GATED_TYPES.contains(&msg_type)', '    let _ = (topic, msg_type);\n    true')],
  'termlink-hub', LIB + 'other_types_and_topics_are_never_gated'),
 ('t3704-m9-put-over-tcp', 'operator ruling A (local socket only)', 'a card write is accepted from a TCP peer', RT,
  [('    peer_addr.map(|p| {\n        ErrorResponse::with_data(\n            id.clone(),\n            control::error_code::KEY_CARD_REFUSED,',
    '    peer_addr.filter(|_| false).map(|p| {\n        ErrorResponse::with_data(\n            id.clone(),\n            control::error_code::KEY_CARD_REFUSED,')],
  'termlink-hub', RTT),
 ('t3704-m10-put-not-admin', 'FCW-128', 'card writes are not admin RPCs (no intent row first)', RA,
  [('    "hub.key_card_put",\n    "hub.key_card_mode",\n', '')],
  'termlink-hub', RTT),
 ('t3704-m11-receiver-ignores-bound-name', 'FCW-98 / E-69.4', 'the receiver does not check a bound from_agent against the key', W,
  [('        if let Some(k) = self.1.get(from_agent)\n', '        if let Some(k) = None::<&VerifyingKey>\n')],
  'termlink-sidecar', WT),
 ('t3704-m12-open-path-skips-names', 'FCW-98', 'an unrostered key may claim a bound agent name', W,
  [('    roster.check_agent(&msg.from_agent, &pk)?;\n    Ok((msg, Signer::Unrostered))', '    Ok((msg, Signer::Unrostered))')],
  'termlink-sidecar', WT),
 ('t3704-m13-no-refusal-counts', 'design v2 C-12', 'refusals are not counted for the impersonated agent', KC,
  [('    if enforced { e.refused += 1 } else { e.would += 1 }', '    let _ = enforced;')],
  'termlink', L4),
 ('t3704-m14-refusals-never-logged', 'AL-33 step 2 (admin_log)', 'no admin row is ever written for a refusal', KC,
  [('        Some(s) => s.insert((agent.to_string(), signer_fp.to_string())),', '        Some(_) => false,')],
  'termlink-hub', LIB + 'refusal_rows_are_coalesced_per_agent_key_minute'),
 ('t3704-m15-impersonation-silent', 'design v2 C-12 (sidecar)', 'the impersonated agent\'s sidecar never raises the attention entry', 'crates/termlink-sidecar/src/pull.rs',
  [('            if total > base {', '            if false && total > base {')],
  'termlink-sidecar', '--test t3704_impersonation a_refused_post_claiming_to_be_me_raises_one_attention_entry'),
 ('t3704-m16-no-startup-reconcile', 'AL-33 2.17.f(2)', 'a card ahead of the admin log is never reported at start', KC,
  [('                    None => out.push(format!("card {} (seq {}) has no logged key-card-put", c.agent, c.seq)),', '                    None => {}')],
  'termlink-hub', LIB + 'al33_startup_reconcile_flags_a_card_the_admin_log_does_not_show'),
 # review round 2 (code) fixes
 ('t3704-m17-outside-edit-accepted', 'r2 MAJOR 2', 'a card store edited outside hub.key_card_put is used', KC,
  [('        && *k != d\n', '        && false\n')],
  'termlink-hub', LIB + 'r2_an_outside_edit_with_the_same_seq_is_refused_until_restart'),
 ('t3704-m18-row-not-first', 'r2 BLOCKER (operator ruling A)', 'the card is written without its admin row', KC,
  [('    log(&card, old.as_ref())?;\n', '')],
  'termlink-hub', LIB + 'r2_the_admin_row_is_written_first_and_a_failed_row_writes_nothing'),
 ('t3704-m19-encryption-key-silent', 'r2 MAJOR 3', 'the encryption key changes without replace', KC,
  [('            || o.encryption_key != req.encryption_key\n', '')],
  'termlink-hub', LIB + 'r2_every_field_change_needs_replace_and_a_key_has_one_card'),
 ('t3704-m20-receiver-admits-other-type', 'r2 MAJOR 2/5 (O-4/C-4)', 'the receiver handles an unlisted type as mail', 'crates/termlink-sidecar/src/pull.rs',
  [('            wire::MSG_TYPE => self.handle_msg(env),', '            wire::MSG_TYPE | "sidecar.receipt" => self.handle_msg(env),')],
  'termlink-sidecar', '--test t3704_impersonation an_unlisted_type_on_the_inbox_is_skipped_never_handled_as_mail'),
 ('t3704-m21-mode-not-reconciled', 'r2 MINOR 10', 'a setting changed outside the RPC is not reported at start', KC,
  [('        Some(m) if *m != now => out.push(', '        Some(m) if false && *m != now => out.push(')],
  'termlink-hub', LIB + 'r2_mode_changes_are_logged_first_and_reconciled_at_start'),
]

out_dir = 'crates/termlink-hub/tests/mutants'
for f in os.listdir(out_dir):
    if f.startswith('t3704-') and f.endswith('.patch'):
        os.remove(os.path.join(out_dir, f))
for name, fc, what, path, edits, pkg, test in M:
    src = open(path).read()
    mut = src
    for old, new in edits:
        if mut.count(old) != 1:
            sys.exit(f'{name}: mutated text found {mut.count(old)} times in {path}: {old!r}')
        mut = mut.replace(old, new, 1)
    with tempfile.NamedTemporaryFile('w', suffix='.rs', delete=False) as t:
        t.write(mut)
    d = subprocess.run(['diff', '-u', '--label', 'a/' + path, '--label', 'b/' + path, path, t.name],
                       capture_output=True, text=True).stdout
    os.unlink(t.name)
    with open(os.path.join(out_dir, name + '.patch'), 'w') as f:
        f.write(f'# fc: {fc}\n# mutation: {what}\n# package: {pkg}\n# test: {test}\n' + d)
print(f'{len(M)} patches written')
