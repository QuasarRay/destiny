// Copyright © 2014 CCP ehf. MIT license retained from original Destiny.
//! Partition.cpp GetProximityCandidates/GetNearbyBalls flag branch and Ball.cpp
//! elapsed/reset rule. Geometry, candidate enumeration and callbacks are separate.
macro_rules! proximity_admission_body {
    ($declare:ident) => {
        $declare!(candidate_admitted(owner_cloaked:bool, free:bool, interactive:bool,
            only_interactives:bool, master:bool, same_bubble:bool, global:bool) -> bool {
            if only_interactives { interactive }
            else { (!master || same_bubble || global) && (owner_cloaked || free) }
        });
    };
}
#[cfg(not(verus_keep_ghost))]
macro_rules! ordinary_admission {
    ($name:ident($($argument:ident:$type:ty),*) -> $result:ty $body:block) => {
        pub(crate) fn $name($($argument:$type),*) -> $result {
            #[cfg(kani)]
            kani::cover!(true,"actual production proximity admission body executed");
            $body
        }
    };
}
#[cfg(not(verus_keep_ghost))]
proximity_admission_body!(ordinary_admission);
#[cfg(verus_keep_ghost)]
pub(crate) use proximity_admission_body;

macro_rules! notification_transition_body {
    ($declare:ident) => {
        $declare!(notification_transition(prior:bool,inside:bool,enabled:bool) -> (bool,bool) {
            if enabled { (inside,prior!=inside) } else { (prior,false) }
        });
    };
}
#[cfg(not(verus_keep_ghost))]
notification_transition_body!(ordinary_admission);
#[cfg(verus_keep_ghost)]
pub(crate) use notification_transition_body;

#[cfg(not(verus_keep_ghost))]
pub(crate) fn strict_range_contains(distance_squared: f64, reach: f64) -> Option<bool> {
    #[cfg(kani)]
    kani::cover!(true, "actual production strict range comparison executed");
    if !distance_squared.is_finite() || distance_squared < 0.0 || !reach.is_finite() || reach < 0.0
    {
        return None;
    }
    Some(distance_squared < reach * reach)
}

#[cfg(not(verus_keep_ghost))]
pub(crate) fn inclusive_range_contains(distance_squared: f64, reach: f64) -> Option<bool> {
    #[cfg(kani)]
    kani::cover!(
        true,
        "actual production inclusive sensor comparison executed"
    );
    if !distance_squared.is_finite() || distance_squared < 0.0 || !reach.is_finite() {
        return None;
    }
    Some(distance_squared <= reach * reach)
}

#[cfg(not(verus_keep_ghost))]
pub(crate) fn annular_range_contains(
    distance_squared: f64,
    hi: f64,
    notification_range: f64,
) -> Option<bool> {
    #[cfg(kani)]
    kani::cover!(
        true,
        "actual production follow notification annulus executed"
    );
    if !distance_squared.is_finite() || distance_squared < 0.0 {
        return None;
    }
    let lo = annulus_lower_radius(hi, notification_range)?;
    Some(distance_squared < hi * hi && distance_squared > lo * lo)
}

#[cfg(not(verus_keep_ghost))]
#[cfg_attr(kani,kani::ensures(|result:&Option<f64>|match result {
    None=>!hi.is_finite() || !range.is_finite() || range<0.0,
    Some(lo)=>hi.is_finite() && range.is_finite() && range>=0.0 && {
        let lower=hi-2.0*range;
        lo.to_bits()==(if lower<0.0 {0.0}else{lower}).to_bits()
    },
}))]
pub(crate) fn annulus_lower_radius(hi: f64, range: f64) -> Option<f64> {
    annulus_lower_radius_math(hi, range)
}

// Keep the observed math in one actual production body. Kani contract checking
// duplicates its wrapper's modes; a witness in that wrapper would also appear
// in the unreachable original mode. This unannotated helper is executed by
// both ordinary production and the checking mode, with all raw inputs.
#[cfg(not(verus_keep_ghost))]
fn annulus_lower_radius_math(hi: f64, range: f64) -> Option<f64> {
    #[cfg(kani)]
    kani::cover!(
        true,
        "actual production source annulus lower radius executed"
    );
    if !hi.is_finite() || !range.is_finite() || range < 0.0 {
        return None;
    }
    let mut lo = hi - 2.0 * range;
    if lo < 0.0 {
        lo = 0.0;
    }
    Some(lo)
}

/// Preserve the source's double addition/comparison and reset to zero, including
/// nonpositive periods. No epsilon or modulo remainder replaces those rules.
#[cfg(not(verus_keep_ghost))]
pub(crate) fn sensor_elapsed_step(prior: f64, dt: f64, period: f64) -> Option<(f64, bool)> {
    #[cfg(kani)]
    kani::cover!(true, "actual production sensor elapsed body executed");
    // The native caller validates finite JSON sensor state and positive tick
    // duration. Reject malformed/private caller inputs before floating-point
    // arithmetic as well; rejection is proved, not assumed out of the domain.
    if !prior.is_finite() || prior < 0.0 || !dt.is_finite() || dt <= 0.0 || !period.is_finite() {
        return None;
    }
    let elapsed = prior + dt;
    Some(if elapsed < period {
        (elapsed, false)
    } else {
        (0.0, true)
    })
}
