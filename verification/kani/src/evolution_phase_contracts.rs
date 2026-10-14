//! Universal machine-value phase contracts; complete callback/FFI and numerical
//! reachability obligations remain in the separate concurrency case mapping.
#[path = "../../../src/evolution_phase.rs"]
mod production;

#[kani::proof]
fn evolution_phase_operations_preserve_the_guard_and_reentry_domains() {
    let phase: u8 = kani::any();
    let began = production::begin_tick(phase);
    let entered = production::enter_callback(phase);
    let left = production::leave_callback(phase);
    let finished = production::finish_tick(phase);
    let allowed = production::add_allowed(phase);
    assert_eq!(allowed, phase == 0);
    assert_eq!(began, if phase == 0 { Some(1) } else { None });
    assert_eq!(entered, if phase == 1 { Some(2) } else { None });
    assert_eq!(left, if phase == 2 { Some(1) } else { None });
    assert_eq!(finished, if phase == 1 { Some(0) } else { None });
    kani::cover!(phase == 0, "idle caller can begin and add");
    kani::cover!(phase == 1, "evolution owner can enter a callback");
    kani::cover!(
        phase == 2,
        "callback caller cannot add or reenter evolution"
    );
    kani::cover!(phase == u8::MAX, "invalid phase value is rejected");
}
