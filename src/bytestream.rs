pub struct Bytestream {
    bytes: String, index: usize
}

impl Bytestream {
    pub fn new(bytes: &str) -> Bytestream {
        return Bytestream { bytes: bytes.to_string(), index: 0 }
    }

    // Consumes x bytes and returns them as a u8 vector
    pub fn get_bytes(&mut self, bytes: u64, convert_endian: bool) -> Vec<u8> {
        let (start, end) = (self.index, self.index+bytes as usize*2);
        self.index += bytes as usize*2;
        let mut string = self.bytes[start..end].to_string();
        if convert_endian { string = Bytestream::convert_endian(&string) }
        let byte_vector = hex::decode(string).unwrap();
        return byte_vector
    }

    // Consumes a variable number of bytes (max 1+8) and returns a u64
    pub fn get_varint(&mut self) -> u64 {
        let integer = self.get_bytes(1, false)[0]; // endian skippable if 1 byte
        let i = u8::MAX - integer;
        if i>2 { return integer as u64 }
        let bytes = if i==0 { 8 } else if i==1 { 4 } else { 2 };
        let byte_vector = self.get_bytes(bytes, true);
        return Bytestream::bytes_to_u64(&byte_vector)
    }

    // Converts big <-> little endian
    pub fn convert_endian(string: &str) -> String {
        let mut new_string = String::new();
        let mut prev_char = ' ';
        for (i, curr_char) in string.chars().rev().enumerate() {
            if i%2 == 0 { 
                prev_char = curr_char;
                continue
            }
            new_string.push(curr_char);
            new_string.push(prev_char);
        }
        return new_string
    }

    // Converts a u8 vector (max length = 8) to a single u64 value
    pub fn bytes_to_u64(byte_vector: &Vec<u8>) -> u64 {
        if byte_vector.len()>8 { panic!("Input exceeds u64 size") }
        let mut varint = 0;
        for byte in byte_vector {
            varint <<= 8;
            varint |= *byte as u64;
        }
        return varint
    }
}