//! Source identity fields update before the synchronous Stop/Field events.
macro_rules! harmonic_action_body {
    ($declare:ident) => {
        $declare! {
            action(mode:u8,field:bool)->u8 {
                #[cfg(kani)]
                kani::cover!(true,"actual harmonic mode action executed");
                if field {10} else if mode==10 {2} else {255}
            }
        }
    };
}
#[cfg(verus_keep_ghost)]
pub(crate) use harmonic_action_body;
#[cfg(not(verus_keep_ghost))]
macro_rules! rust_harmonic_action {
    ($name:ident($mode:ident:u8,$field:ident:bool)->u8 $body:block)=>{
        pub(crate) fn $name($mode:u8,$field:bool)->u8 $body
    };
}
#[cfg(not(verus_keep_ghost))]
harmonic_action_body!(rust_harmonic_action);
#[cfg(not(verus_keep_ghost))]
pub(crate) fn fields(harmonic: i64, corporation: i32, alliance: i32) -> [i64; 3] {
    #[cfg(kani)]
    kani::cover!(true, "actual harmonic identity fields executed");
    [harmonic, i64::from(corporation), i64::from(alliance)]
}
