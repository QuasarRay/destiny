//! Shared production phase transitions. The executable bodies are also checked
//! by Verus; engine/world and foreign callback correspondence is separate.
macro_rules! evolution_phase_bodies {
    ($declare:ident) => {
        $declare!(begin_tick(phase: u8) -> Option<u8> {
            if phase == 0 { Some(1) } else { None }
        });
        $declare!(enter_callback(phase: u8) -> Option<u8> {
            if phase == 1 { Some(2) } else { None }
        });
        $declare!(leave_callback(phase: u8) -> Option<u8> {
            if phase == 2 { Some(1) } else { None }
        });
        $declare!(finish_tick(phase: u8) -> Option<u8> {
            if phase == 1 { Some(0) } else { None }
        });
        $declare!(add_allowed(phase: u8) -> bool { phase == 0 });
    };
}
#[cfg(not(verus_keep_ghost))]
macro_rules! ordinary_phase_function {
    ($name:ident($phase:ident: u8) -> $result:ty $body:block) => {
        pub(crate) fn $name($phase: u8) -> $result {
            #[cfg(kani)]
            kani::cover!(true, "actual production evolution phase body executed");
            $body
        }
    };
}
#[cfg(not(verus_keep_ghost))]
evolution_phase_bodies!(ordinary_phase_function);
#[cfg(verus_keep_ghost)]
pub(crate) use evolution_phase_bodies;
