pub fn branch_invalid(x: bool) {
    assert!(x);
}

pub fn contradictory_assumption(x: bool) {
    kani::assume(x);
    kani::assume(!x);
    assert!(x);
}

pub fn add_overflow_invalid(x: u8) {
    if x + 1 > x {
        assert!(true);
    } else {
        assert!(true);
    }
}

pub fn sub_underflow_invalid(x: u8) {
    if x - 1 < x {
        assert!(true);
    } else {
        assert!(true);
    }
}
