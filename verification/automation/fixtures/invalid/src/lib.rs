pub fn branch_invalid(x: bool) {
    assert!(x);
}

pub fn contradictory_assumption(x: bool) {
    kani::assume(x);
    kani::assume(!x);
    assert!(x);
}
