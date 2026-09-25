use vstd::prelude::*;

verus! {

pub const MAX_FORMATION_SLOTS: int = 16;
pub const TEN_BILLION: int = 10_000_000_000;

pub struct OriginalBallDefaults {
    pub id: int,
    pub mass: int,
    pub radius: int,
    pub max_velocity: int,
    pub is_free: bool,
    pub is_global: bool,
    pub is_massive: bool,
    pub is_interactive: bool,
    pub is_cloaked: bool,
    pub is_moribund: bool,
    pub harmonic: int,
    pub corporation_id: int,
    pub alliance_id: int,
    pub x: int,
    pub y: int,
    pub z: int,
    pub agility: int,
    pub speed_fraction: int,
    pub mode: int,
    pub follow_range: int,
    pub new_bubble_id: int,
    pub old_bubble_id: int,
    pub formation_id: int,
    pub mini_ball_count: int,
    pub has_ballpark: bool,
}

pub open spec fn original_ball_defaults() -> OriginalBallDefaults {
    OriginalBallDefaults {
        id: 0,
        mass: 0,
        radius: 0,
        max_velocity: 0,
        is_free: false,
        is_global: false,
        is_massive: false,
        is_interactive: false,
        is_cloaked: false,
        is_moribund: false,
        harmonic: -1,
        corporation_id: -1,
        alliance_id: -1,
        x: TEN_BILLION,
        y: TEN_BILLION,
        z: TEN_BILLION,
        agility: 1,
        speed_fraction: 1,
        mode: 0,
        follow_range: 10,
        new_bubble_id: -1,
        old_bubble_id: -1,
        formation_id: 255,
        mini_ball_count: 0,
        has_ballpark: false,
    }
}

// original-test: python/destiny/test/test_ball.py::test_can_construct
pub proof fn original_ball_can_be_constructed()
    ensures original_ball_defaults().id == 0,
{
}

// original-test: python/destiny/test/test_ball.py::test_default_values
pub proof fn original_ball_defaults_are_exact()
    ensures
        original_ball_defaults().id == 0,
        original_ball_defaults().mass == 0,
        original_ball_defaults().radius == 0,
        original_ball_defaults().max_velocity == 0,
        !original_ball_defaults().is_free,
        !original_ball_defaults().is_global,
        !original_ball_defaults().is_massive,
        !original_ball_defaults().is_interactive,
        !original_ball_defaults().is_cloaked,
        !original_ball_defaults().is_moribund,
        original_ball_defaults().harmonic == -1,
        original_ball_defaults().corporation_id == -1,
        original_ball_defaults().alliance_id == -1,
        original_ball_defaults().x == TEN_BILLION,
        original_ball_defaults().y == TEN_BILLION,
        original_ball_defaults().z == TEN_BILLION,
        original_ball_defaults().agility == 1,
        original_ball_defaults().speed_fraction == 1,
        original_ball_defaults().mode == 0,
        original_ball_defaults().follow_range == 10,
        original_ball_defaults().new_bubble_id == -1,
        original_ball_defaults().old_bubble_id == -1,
        original_ball_defaults().formation_id == 255,
        original_ball_defaults().mini_ball_count == 0,
        !original_ball_defaults().has_ballpark,
{
}

pub open spec fn child_count_after_successful_add(current: int) -> int {
    current + 1
}

// original-test: python/destiny/test/test_ball.py::test_can_add_miniball
pub proof fn miniball_add_increments_count(current: int)
    ensures child_count_after_successful_add(current) == current + 1,
{
}

pub open spec fn mini_capsule_radius_accepted(radius: int) -> bool {
    radius > 0
}

pub open spec fn child_count_after_capsule_attempt(current: int, radius: int) -> int {
    if mini_capsule_radius_accepted(radius) { current + 1 } else { current }
}

// original-test: python/destiny/test/test_ball.py::test_can_add_minicapsule
pub proof fn positive_minicapsule_add_increments_count(current: int, radius: int)
    requires radius > 0,
    ensures
        mini_capsule_radius_accepted(radius),
        child_count_after_capsule_attempt(current, radius) == current + 1,
{
}

// original-test: python/destiny/test/test_ball.py::test_can_not_add_minicapsule_with_negative_radius
pub proof fn nonpositive_minicapsule_is_rejected(current: int, radius: int)
    requires radius <= 0,
    ensures
        !mini_capsule_radius_accepted(radius),
        child_count_after_capsule_attempt(current, radius) == current,
{
}

pub open spec fn identity_rotated_vector(vector: Seq<int>) -> Seq<int> {
    vector
}

// original-test: python/destiny/test/test_ball.py::test_get_rotated_vector_returns_original_vector_if_there_is_no_rotation
pub proof fn identity_rotation_preserves_vector(vector: Seq<int>)
    ensures identity_rotated_vector(vector) == vector,
{
}

pub open spec fn proximity_sensor_accepted(ball_radius: int, range: int) -> bool {
    ball_radius + range >= 0
}

// original-test: python/destiny/test/test_ball.py::test_add_proximity_sensor
pub proof fn original_proximity_example_is_accepted()
    ensures proximity_sensor_accepted(0, 5),
{
}

pub open spec fn next_sequential_formation_slot(
    reserved_prefix: int,
    slot_count: int,
    formation_assigned: bool,
    park_present: bool,
    formation_valid: bool,
) -> int {
    if !formation_assigned
        || !park_present
        || !formation_valid
        || slot_count > MAX_FORMATION_SLOTS
        || reserved_prefix >= slot_count
    {
        -1
    } else {
        reserved_prefix
    }
}

// original-test: python/destiny/test/test_ball.py::test_reserve_formation_slot_returns_negative_one_when_ball_is_not_set_up_properly
pub proof fn unconfigured_ball_has_no_formation_slot(reserved: int, slots: int)
    ensures next_sequential_formation_slot(reserved, slots, false, false, false) == -1,
{
}

// original-test: python/destiny/test/test_ball.py::test_reserve_formation_slot
pub proof fn first_valid_formation_slot_is_zero(slots: int)
    requires 0 < slots <= MAX_FORMATION_SLOTS,
    ensures next_sequential_formation_slot(0, slots, true, true, true) == 0,
{
}

// original-test: python/destiny/test/test_ball.py::test_slots_are_reserved_in_incremental_order
pub proof fn valid_formation_slots_are_incremental(reserved: int, slots: int)
    requires
        0 <= reserved < slots,
        slots <= MAX_FORMATION_SLOTS,
    ensures
        next_sequential_formation_slot(reserved, slots, true, true, true) == reserved,
{
}

// original-test: python/destiny/test/test_ball.py::test_reserving_too_many_formation_slots_fails
pub proof fn full_formation_has_no_free_slot(slots: int)
    requires 0 <= slots <= MAX_FORMATION_SLOTS,
    ensures next_sequential_formation_slot(slots, slots, true, true, true) == -1,
{
}

pub open spec fn slot_reused_after_free(
    freed_slot: int,
    slot_count: int,
    formation_assigned: bool,
    park_present: bool,
    formation_valid: bool,
) -> int {
    if !formation_assigned
        || !park_present
        || !formation_valid
        || slot_count <= 0
        || slot_count > MAX_FORMATION_SLOTS
        || freed_slot < 0
        || freed_slot >= slot_count
    {
        -1
    } else {
        freed_slot
    }
}

// original-test: python/destiny/test/test_ball.py::test_free_formation_slot
pub proof fn freed_formation_slot_is_reused(freed: int, slots: int)
    requires
        0 <= freed < slots,
        slots <= MAX_FORMATION_SLOTS,
    ensures slot_reused_after_free(freed, slots, true, true, true) == freed,
{
}

} // verus!
