#[kani::requires(x)]
#[kani::ensures(|_| x)]
pub fn contracted_valid(x: bool) {
    assert!(x);
}
