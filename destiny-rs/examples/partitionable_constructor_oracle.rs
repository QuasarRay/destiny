// Differential counterpart of the actual original C++ constructor.
use destiny_rs::Partitionable;
fn main() {
    let p = Partitionable::new();
    println!("state {} {} {} {} {} {} {} {} {} {}",
        p.mId, p.mHandled as u8, p.mNewBubble, p.mOldBubble,
        p.isInteractive as u8, p.mBoxes.len(),
        p.mMyBox.is_none() as u8, p.mPark.is_none() as u8,
        p.mEffectStamp, p.isMoribund as u8);
    println!("vectors {} {} {} {} {} {}",
        p.mNewPos.x, p.mNewPos.y, p.mNewPos.z,
        p.mOldPos.x, p.mOldPos.y, p.mOldPos.z);
}
