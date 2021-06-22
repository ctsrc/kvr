use serde::{Serialize, Deserialize};
use ulid::Ulid;

use kvr::KeyValueRevision;
//use kvr::data::Revision;
use kvr::magic::Magic;
use kvr::error::KVRInitializationError;

#[derive(Clone, Copy, Ord, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExampleKey(Ulid);

impl Magic for ExampleKey {
    fn magic() -> Self {
        Self(Ulid(u128::from_be_bytes([
            0x00, 0x01, 0x02, 0x03,
            0x10, 0x11, 0x12, 0x13,
            0b10101010, 0b01010101, 0b00110011, 0b11001100,
            0b11100111, 0b10100101, 0b00000001, 0b10000001,
        ])))
    }
}

#[derive(Debug)]
enum KVRMainError {
    InitializationError(KVRInitializationError),
}

#[tokio::main]
async fn main() -> Result<(), KVRMainError> {
    //let magic_rev = Revision::magic();
    //println!("Revision magic number: {}", magic_rev);

    let x = KeyValueRevision::<ExampleKey, u32>::try_init("example.db", true)
        .await.map_err(|e| KVRMainError::InitializationError(e))?;

    Ok(())
}