#[kani::requires(x)]
#[kani::ensures(|_| x)]
pub fn contracted_valid(x: bool) {
    assert!(x);
}

#[kani::requires(x)]
#[kani::ensures(|_| !x)]
pub fn contracted_invalid(x: bool) {
    assert!(x);
}

#[kani::requires(x)]
#[kani::requires(!x)]
#[kani::ensures(|_| true)]
pub fn contracted_vacuous(x: bool) {
    assert!(true);
}

#[kani::requires(x + 1 > x)]
#[kani::ensures(|_| true)]
pub fn contracted_arithmetic(x: u8) {
    assert!(true);
}

#[kani::requires(true)]
#[kani::ensures(|result: &u8| *result == x)]
pub fn contracted_result_valid(x: u8) -> u8 {
    x
}

#[kani::requires(true)]
#[kani::ensures(|result: &u8| *result != x)]
pub fn contracted_result_invalid(x: u8) -> u8 {
    x
}
