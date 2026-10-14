pub fn branch_valid(x: bool) {
    if x {
        assert!(x);
    } else {
        assert!(!x);
    }
}

pub fn fixed_loop_valid(x: bool) {
    for _ in 0..3 {
        if x {
            assert!(x);
        } else {
            assert!(!x);
        }
    }
}
