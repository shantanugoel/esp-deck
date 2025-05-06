use anyhow::Result;
use serde::{Deserialize, Serialize};
const MAGIC_WORD: u32 = 0xE59DECC0;
// Magic Word + Payload length
const HEADER_SIZE: usize = 8;

pub struct DeFramedMessage<'a> {
    pub payload_length: u32,
    pub payload: &'a [u8],
}

pub fn frame_message<T: Serialize>(payload: &T) -> Result<Vec<u8>> {
    let json_payload = serde_json::to_vec(payload)?;
    let payload_bytes = json_payload.as_slice();
    let payload_length = payload_bytes.len();

    let mut frame = Vec::with_capacity(HEADER_SIZE + payload_length);
    frame.extend_from_slice(&MAGIC_WORD.to_le_bytes());
    frame.extend_from_slice(&payload_length.to_le_bytes());
    frame.extend_from_slice(payload_bytes);
    Ok(frame)
}

pub fn deframe_message(frame_bytes: &[u8]) -> Result<DeFramedMessage> {
    if frame_bytes.len() < HEADER_SIZE {
        return Err(anyhow::anyhow!("Frame is too short"));
    }

    let magic_word = u32::from_le_bytes(frame_bytes[0..4].try_into().unwrap());

    if magic_word != MAGIC_WORD {
        return Err(anyhow::anyhow!("Invalid magic word"));
    }

    let payload_length = usize::from_le_bytes(frame_bytes[4..8].try_into().unwrap());

    if frame_bytes.len() < HEADER_SIZE + payload_length {
        return Err(anyhow::anyhow!("Frame is too short"));
    }

    let payload = &frame_bytes[HEADER_SIZE..HEADER_SIZE + payload_length];
    Ok(DeFramedMessage {
        payload_length: payload_length as u32,
        payload,
    })
}

impl<'a> DeFramedMessage<'a> {
    pub fn deserialize<T: Deserialize<'a>>(&'a self) -> Result<T> {
        let payload = serde_json::from_slice(self.payload)?;
        Ok(payload)
    }
}
