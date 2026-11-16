#!/usr/bin/env python3
"""T-3366: regenerate the unit's mutant patches from the current sources (run from the repo root).

Each mutant breaks one rule and names the test that must go red; scripts/arc011-mutants.sh runs them:
  ARC011_MUTANTS_DIR=crates/termlink-hub/tests/mutants ARC011_MUTANTS_PACKAGE=termlink-hub \
    bash scripts/arc011-mutants.sh --report <file>
A source edit that changes a mutated line makes this script fail loudly (the old text is not found).
"""
import os, subprocess, sys, tempfile

H = 'crates/termlink-hub/src/hub_identity.rs'
R = 'crates/termlink-hub/src/router.rs'
P = 'crates/termlink-sidecar/src/hub_key_pin.rs'
LIB = '--lib hub_identity::tests::'
L4 = '--test t3366_hub_identity '
SC = '--test t3366_hub_key_pin '
E37 = SC + 'e37_first_contact_pins_restart_keeps_and_a_swapped_key_is_refused_and_named'

M = [
 ('t3366-m1-derive-id-per-call', 'AL-17 inv.3', 'hub.version derives hub_id from the certificate on each call (ARC4 6.1.7)', R,
  [('let hub_id = ident.as_ref().map(|i| i.hub_id.clone());', 'let hub_id = instance.as_deref().and_then(crate::tls::hub_id_from_fingerprint);')],
  'termlink', L4 + 'e36_e37_restart_and_certificate_rotation_keep_id_and_signing_key'),
 ('t3366-m2-no-fcw79', 'FCW-79', 'a missing hub.id beside prior state is minted instead of refused', H,
  [('        (None, _) if has_prior_state(dir) => Err(refuse(format!(', '        (None, _) if false && has_prior_state(dir) => Err(refuse(format!(')], 'termlink-hub', LIB + 'e36_fcw79_missing_id_beside_state_refuses_and_writes_nothing'),
 ('t3366-m3-lenient-id', 'FCW-80', 'any 16-character hub.id is accepted', H,
  [("    s.len() == 16 && s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))", '    s.len() == 16')],
  'termlink-hub', LIB + 'e36_fcw80_malformed_id_refuses'),
 ('t3366-m4-no-key-mode-check', 'FCW-82', 'a signing key file with any mode is accepted', H,
  [('    if mode != 0o600 {', '    if false && mode != 0o600 {')], 'termlink-hub', LIB + 'e37_fcw82_bad_key_file_refuses'),
 ('t3366-m5-key-regenerated', 'AL-18', 'a missing signing key beside an existing id is regenerated at start', H,
  [('        (Some(_), None) => Err(refuse(format!(\n            "hub.signing.key missing beside existing state',
    '        (Some(hub_id), None) => Ok(HubIdentity { hub_id, signing: generate_key(dir)? }),\n        #[allow(unreachable_patterns)]\n        (Some(_), None) => Err(refuse(format!(\n            "hub.signing.key missing beside existing state')],
  'termlink-hub', LIB + 'e37_key_missing_beside_id_refuses'),
 ('t3366-m6-write-in-place', 'FCW-81', 'hub.id is written in place, not via temp + publish', H,
  [('.create_new(true).write(true).mode(mode).open(&tmp)?;', '.create(true).truncate(true).write(true).mode(mode).open(&path)?;')],
  'termlink-hub', LIB + 'e166_failure_before_rename_leaves_no_id_and_no_temp'),
 ('t3366-m7-adopt-overwrites', 'AL-17 adopt', 'adopt-id replaces an id already on disk', H,
  [('        (Some(id), Some(k)) => {\n            check(&id)?;',
    '        (Some(_), Some(k)) => {\n            let id = mint_value(dir).0;\n            let _ = std::fs::remove_file(dir.join(ID_FILE));\n            write_id(dir, &id)?;\n            check(&id)?;')],
  'termlink-hub', LIB + 'adopt_is_idempotent_and_never_changes_an_existing_id'),
 ('t3366-m8-debug-prints-key', 'SI-25', 'the debug form prints the private seed', H,
  [('            .field("signing_key", &self.signing_key_fingerprint())', '            .field("signing_key", &hex(&self.signing.to_bytes()))')],
  'termlink-hub', LIB + 'e37_private_key_is_never_printed'),
 ('t3366-m9-silent-repin', 'FCW-83 / E-37 control', 'a changed hub key is re-pinned silently', P,
  [('                Some(p) => mismatch(store, hub_id, p, off),', '                Some(_p) => { let _ = store.write_state(&pin_state_name(hub_id), &off); KeyState::Pinned(off) }')],
  'termlink-sidecar', E37),
 ('t3366-m10-pin-untrusted', 'FCW-84', 'a first contact off the local socket pins the key', P,
  [('                None if !trusted_path || !valid_fp(&off) => KeyState::Unverified(off),', '                None if !valid_fp(&off) => KeyState::Unverified(off),')],
  'termlink-sidecar', SC + 'e166_fcw84_untrusted_first_contact_pins_nothing_and_no_offer_changes_nothing'),
 ('t3366-m11-gate-ignores-pin', 'FCW-83', 'verified_hub_key accepts any offered key', P,
  [('    (pin.trim() == off).then_some(off)', '    let _ = pin;\n    Some(off)')], 'termlink-sidecar', E37),
 ('t3366-m12-mismatch-silent', 'FCW-83 / OBS-36', 'a changed hub key raises no needs-attention entry', P,
  [('    if let Err(e) = store.attention_coalesced("hub-signing-key-changed", &format!("{day}:{off}"), &detail) {',
    '    if let Err(e) = Ok::<bool, crate::store::StoreError>(false) {\n        let _ = (&detail, day);')],
  'termlink-sidecar', E37),
 # review round 1 fixes
 ('t3366-m13-adopt-rekeys', 'AL-18 / r1 #1', 'adopt-id generates a new key for a migrated hub that lost its key', H,
  [('        (Some(_), None) => Err(refuse(format!(\n            "hub.signing.key missing beside hub.id',
    '        (Some(id), None) => { let k = generate_key(dir)?; Ok(Adopted { hub_id: id, id_source: "kept", key_generated: true, signing_key: signing_fingerprint(&k) }) }\n        #[allow(unreachable_patterns)]\n        (Some(_), None) => Err(refuse(format!(\n            "hub.signing.key missing beside hub.id')],
  'termlink-hub', LIB + 'adopt_refuses_a_lost_key_beside_an_id_and_writes_no_key'),
 ('t3366-m14-adopt-remints', 'FCW-79 / r1 #1', 'adopt-id re-mints a migrated hub that lost its id', H,
  [('        (None, Some(_)) => Err(refuse(format!(', '        (None, Some(_)) if false => Err(refuse(format!('),
   ('        (None, None) => {', '        (None, _) => {')],
  'termlink-hub', LIB + 'adopt_refuses_a_lost_id_beside_a_key_and_writes_no_id'),
 ('t3366-m15-expect-ignored', 'r1 #4', 'adopt-id --expect is ignored', H,
  [('            Some(e) if e != found => Err(refuse(format!(', '            Some(e) if false && e != found => Err(refuse(format!(')],
  'termlink-hub', LIB + 'adopt_expect_mismatch_writes_nothing'),
 ('t3366-m16-publish-clobbers', 'AL-17 inv.1 / r1 #3', 'the publish replaces an existing hub.id', H,
  [('        std::fs::hard_link(&tmp, &path)?;', '        std::fs::rename(&tmp, &path)?;')],
  'termlink-hub', LIB + 'e166_write_never_clobbers_an_existing_id'),
 ('t3366-m17-read-error-repins', 'FCW-83 / r1 #2', 'an unreadable pin is taken as no pin and re-pinned', P,
  [('    let pin = match store.read_state(&pin_state_name(hub_id)) {',
    '    let pin = match store.read_state(&pin_state_name(hub_id)).or_else(|_| Ok::<_, crate::store::StoreError>(None)) {')],
  'termlink-sidecar', SC + 'e37_unreadable_pin_is_not_replaced'),
 ('t3366-m18-status-always-match', 'E-126', 'status says the hub id matches whatever answered', P,
  [('    let m = seen.as_deref() == Some(d);', '    let m = true || seen.as_deref() == Some(d);')],
  'termlink-sidecar', SC + 'e126_status_states_the_answering_hub_id_and_whether_it_matches'),
 ('t3366-m19-cert-not-prior', 'FCW-79 / ruling B', 'a certificate-only directory mints silently instead of refusing', H,
  [('    ["hub.secret", "hub.cert.pem", KEY_FILE]', '    ["hub.secret", KEY_FILE]')],
  'termlink-hub', LIB + 'e36_cert_only_dir_refuses_until_adopt_gives_the_certificate_id'),
 # review round 2 fixes
 ('t3366-m20-journal-ignored', 'E-166 crash table / r2 #5', 'an interrupted mint is not finished by the next start', H,
  [('    if let Some(done) = finish_pending(dir)? {\n        tracing::info!(hub_id = %done.hub_id, "hub identity: an interrupted mint was finished (T-3366)");',
    '    if let Some(done) = None::<HubIdentity> {\n        tracing::info!(hub_id = %done.hub_id, "hub identity: an interrupted mint was finished (T-3366)");')],
  'termlink-hub', LIB + 'e166_interrupted_first_start_is_finished_by_the_next_start'),
 ('t3366-m21-pin-clobbers', 'AL-18 inv.3 / r2 #2', 'the pin publish replaces another thread\'s pin', P,
  [('match store.write_state_new(&pin_state_name(hub_id), &off) {', 'match store.write_state(&pin_state_name(hub_id), &off).map(|_| true) {')],
  'termlink-sidecar', SC + 'e37_concurrent_first_contacts_with_different_offers_pin_once'),
 ('t3366-m22-pin-without-record', 'OBS-37 / r2 #2', 'the pin is written although its record could not be', P,
  [('                    if let Err(e) = store.attention("hub-signing-key-pinned", &format!("hub {hub_id} signing key pinned: {off}")) {',
    '                    if let Err(e) = Ok::<(), crate::store::StoreError>(()) {')],
  'termlink-sidecar', SC + 'e37_no_record_means_no_pin'),
 ('t3366-m23-no-admin-row', 'OBS-37 / r2 #2b', 'a new pin writes no admin row', P,
  [('                                admin_row_pinned(store, hub_id, &off);\n', '')],
  'termlink-sidecar', E37),
]

out_dir = 'crates/termlink-hub/tests/mutants'
for f in os.listdir(out_dir):
    if f.startswith('t3366-') and f.endswith('.patch'):
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
