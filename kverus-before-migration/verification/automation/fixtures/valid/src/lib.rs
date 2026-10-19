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

pub fn independent_inputs(a: u8, b: u8) {
    assert!(a == a && b == b);
}

pub fn assumed_valid(x: u8) {
    kani::assume(x > 0);
    assert!(x > 0);
}

pub fn conditional_assume_valid(x: bool, y: bool) {
    if x {
        kani::assume(y);
        assert!(y);
    } else {
        assert!(!x);
    }
}

pub fn guarded_add_valid(x: u8) {
    if x == 255 {
        assert!(true);
    } else {
        if x + 1 > x {
            assert!(true);
        } else {
            assert!(false);
        }
    }
}

pub fn guarded_sub_valid(x: u8) {
    if x == 0 {
        assert!(true);
    } else {
        if x - 1 < x {
            assert!(true);
        } else {
            assert!(false);
        }
    }
}

pub fn generic_selected<T>(x: T) {
    assert!(true);
}
