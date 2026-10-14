//! Actual native detached-object constructor fields. The source-backed
//! executable specification supplies the inherited scalar defaults; native
//! allocation, class registration and the snapshot adapter are separate callers.
pub(crate) fn detached_fields(client: bool) -> destiny_original_spec::OriginalParkBall {
    let ball = destiny_original_spec::OriginalParkBall {
        client_ball: client,
        ..destiny_original_spec::OriginalParkBall::default()
    };
    #[cfg(kani)]
    kani::cover!(
        true,
        "actual native detached constructor fields initialized"
    );
    ball
}
