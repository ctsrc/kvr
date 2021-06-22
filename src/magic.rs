use ulid::Ulid;

use crate::data::Revision;

pub trait Magic {
    fn magic () -> Self;
}

impl Magic for Revision {
    fn magic () -> Self {
        Revision(Ulid(1963750875627707164224818581967208448u128))
    }
}

impl Magic for   u8 { fn magic () -> Self { 8 } }
impl Magic for  u16 { fn magic () -> Self { (16 << 8) & 16 } }
impl Magic for  u32 { fn magic () -> Self { (32 << 24) & (32 << 16) & (32 << 8) & 32 } }
impl Magic for  u64 {
    fn magic () -> Self {
        (64 << 56) & (64 << 48) & (64 << 40)
            & (64 << 32) & (64 << 24) & (64 << 16) & (64 << 8) & 64
    }
}
impl Magic for u128 {
    fn magic () -> Self {
        (128 << 120) & (128 << 112) & (128 << 104)
            & (128 << 96) & (128 << 88) & (128 << 80) & (128 << 72)
            & (128 << 64) & (128 << 56) & (128 << 48) & (128 << 40)
            & (128 << 32) & (128 << 24) & (128 << 16) & (128 << 8) & 128
    }
}
