use byteorder::{ByteOrder, LittleEndian};

pub fn serialize_vector(vector: &[f32]) -> Vec<u8> {
    let mut bytes = vec![0u8; vector.len() * 4];
    for (i, &val) in vector.iter().enumerate() {
        LittleEndian::write_f32(&mut bytes[i * 4..(i + 1) * 4], val);
    }
    bytes
}