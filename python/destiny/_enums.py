"""Legacy numeric constants preserved for Carbon call sites."""

from enum import IntEnum


class DstBallMode(IntEnum):
    DSTBALL_GOTO = 0
    DSTBALL_FOLLOW = 1
    DSTBALL_STOP = 2
    DSTBALL_WARP = 3
    DSTBALL_ORBIT = 4
    DSTBALL_MISSILE = 5
    DSTBALL_MUSHROOM = 6
    DSTBALL_BOID = 7
    DSTBALL_TROLL = 8
    DSTBALL_MINIBALL = 9
    DSTBALL_FIELD = 10
    DSTBALL_RIGID = 11
    DSTBALL_FORMATION = 12


class DstEventType(IntEnum):
    DST_CREATE = 1
    DST_DESTROY = 2
    DST_PROXIMITY = 3
    DST_PRETICK = 4
    DST_POSTTICK = 5
    DST_COLLISION = 6
    DST_RANGE = 7
    DST_MODECHANGE = 8
    DST_PARTITION = 9
    DST_WARPACTIVATION = 10
    DST_WARPEXIT = 11


class DstConstants(IntEnum):
    DSTLOCALBALLS = -1073741824
    DSTNORMALCLOAK = 1
    DSTRESTORECLOAK = 2
    DSTGMCLOAK = 3


# The Blue extension exported every registered chooser member directly from
# ``destiny``.  The IntEnum classes are additive convenience APIs; these names
# are re-exported by destiny.__init__ for legacy Carbon imports.
LEGACY_CONSTANTS = {
    member.name: member
    for enum_type in (DstBallMode, DstEventType, DstConstants)
    for member in enum_type
}
