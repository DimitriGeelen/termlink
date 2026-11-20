//! T-3708 (E-59, r1 MAJOR 9 / BLOCKER): the deadline ages the sidecar uses do not move with a wall jump, through the
//! real clock path (`store::test_shift_wall_ms` shifts what `store::now_ms` returns). Own binary: the shift is
//! process-wide, so nothing else may run in this process.

use termlink_sidecar::fields;
use termlink_sidecar::store;

#[test]
fn t3708_e59_deadline_ages_ignore_wall_jumps_on_the_real_clock_path() {
    fields::look();
    let stamp = store::now_ms() - 10_000; // stored 10 s ago, before any jump
    assert!((10_000..12_000).contains(&fields::age_ms(stamp, store::now_ms())));

    store::test_shift_wall_ms(3_600_000); // +1 h
    fields::look();
    let a = fields::age_ms(stamp, store::now_ms());
    assert!(a < 15_000, "a +1 h jump aged the stamp to {a} ms: a deadline would fire early");

    store::test_shift_wall_ms(-7_200_000); // then -2 h
    fields::look();
    let b = fields::age_ms(stamp, store::now_ms());
    assert!((10_000..15_000).contains(&b), "after -2 h the stamp's age is {b} ms: a deadline would fire late");

    // a stamp written after the jumps is aged plainly
    let late = store::now_ms() - 1_000;
    assert!((1_000..3_000).contains(&fields::age_ms(late, store::now_ms())));
    store::test_shift_wall_ms(3_600_000);
}
