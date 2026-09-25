#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisibilityState {
    pub cloaked: bool,
    pub massive: bool,
}

#[must_use]
pub const fn cloak_transition() -> VisibilityState {
    VisibilityState {
        cloaked: true,
        massive: false,
    }
}

#[must_use]
pub const fn uncloak_transition(in_warp: bool) -> VisibilityState {
    VisibilityState {
        cloaked: false,
        massive: !in_warp,
    }
}

#[must_use]
pub const fn visibility_candidate_blocks(
    is_massive: bool,
    is_cloaked: bool,
    intersects_open_segment: bool,
) -> bool {
    is_massive && !is_cloaked && intersects_open_segment
}

#[must_use]
pub const fn visibility_result(candidate_id: i64, candidate_blocks: bool) -> i64 {
    if candidate_blocks { candidate_id } else { 0 }
}

/// Exact algebraic form of the original ScanCone predicate for the tested
/// full angle pi/2 and direction +X. The original halves the input angle, so
/// cos^2(pi/4) = 1/2 and the cone test becomes 2*x^2 >= distance^2.
#[must_use]
pub fn scan_cone_pi_over_2_x(offset: [i64; 3], range: i64) -> bool {
    if range <= 0 {
        return false;
    }
    let x = i128::from(offset[0]);
    let y = i128::from(offset[1]);
    let z = i128::from(offset[2]);
    let r = i128::from(range);
    let distance_squared = x * x + y * y + z * z;
    x >= 0
        && 2 * x * x >= distance_squared
        && distance_squared <= r * r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visibility_transitions_match_original_examples() {
        assert_eq!(
            cloak_transition(),
            VisibilityState {
                cloaked: true,
                massive: false,
            }
        );
        assert_eq!(
            uncloak_transition(false),
            VisibilityState {
                cloaked: false,
                massive: true,
            }
        );
        assert_eq!(
            uncloak_transition(true),
            VisibilityState {
                cloaked: false,
                massive: false,
            }
        );
    }

    #[test]
    fn scan_cone_examples_match_original_tests() {
        assert!(scan_cone_pi_over_2_x([50, 0, 0], 100));
        for offset in [[-50, 0, 0], [0, 50, 0], [0, -50, 0], [0, 0, 50], [0, 0, -50]] {
            assert!(!scan_cone_pi_over_2_x(offset, 100));
        }
    }
}
