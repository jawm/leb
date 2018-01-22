use std::io::{
    Bytes,
    Error,
    Read,
    ErrorKind
};

const CONTINUE_MASK: u64 = 0x80;
const VALUE_MASK: u64 = 0x7F;
const SIGN_MASK: u64 = 0x40;

pub trait ReadLEB : Iterator {
    fn read_varuint(&mut self, max_bits: i8) -> Result<u64, Error>;

    fn read_varint(&mut self, max_bits: i8) -> Result<i64, Error>;
}

impl<R: Read> ReadLEB for Bytes<R> {
    fn read_varuint(&mut self, max_bits: i8) -> Result<u64, Error> {
        return Ok(vuN(self, max_bits as i64));
    }

    fn read_varint(&mut self, max_bits: i8) -> Result<i64, Error> {
        return Ok(vsN(self, max_bits as i64));
    }
}

pub fn vuN<R: Read>(buffer: &mut Bytes<R>, max_bits: i64) -> u64  {
    assert!(max_bits > 0);
    let byte = buffer.next().unwrap().unwrap() as u64;
    assert!(max_bits >=7 || byte & 0x7f < 0xff << max_bits);
    let result = byte & 0x7f;
    if byte & 0x80 == 0 {
        return result;
    } else {
        return result | (vuN(buffer, max_bits-7) << 7);
    }
}

pub fn vsN<R: Read>(buffer: &mut Bytes<R>, max_bits: i64) -> i64 {
    assert!(max_bits > 0);
    let byte = buffer.next().unwrap().unwrap() as i64;
    let mask = ((u64::pow(2, 64-max_bits as u32)-1 << max_bits%64) & 0x7f )as i64;
    assert!(max_bits >= 7 || byte & mask == 0 || byte & mask == mask);
    let result = byte & 0x7f as i64;
    if byte & 0x80 == 0 {
        return if byte & 0x40 == 0 {
            result
        } else {
            result | (-1i64 ^ 0x7fi64)
        };
    } else {
        return result | (vsN(buffer, max_bits-7) << 7);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Bytes, Cursor, Read};

    fn b(bytes: &[u8]) -> Bytes<Cursor<Vec<u8>>> {
        Cursor::new(bytes.to_vec()).bytes()
    }

    #[test]
    fn new() {
        assert!(vuN(&mut b(&[0]), 1) == 0);
        assert!(vuN(&mut b(&[0xE5, 0x8E, 0x26]), 32) == 624485);
        assert!(vuN(&mut b(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01]), 64) == 0xffff_ffff_ffff_ffff);
        assert!(vsN(&mut b(&[0x80, 0x7f]), 32) == -128);
        assert!(vsN(&mut b(&[0x80, 0x80, 0x80, 0x80, 0x78]), 32) == -2147483648);
        assert!(vsN(&mut b(&[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x7f]), 64) == i64::min_value());
    }

    #[test]
    fn test_unsigned_decode() {
        assert!(b(&[0]).read_varuint(1).unwrap() == 0);

        assert!(b(&[1]).read_varuint(1).unwrap() == 1);
        assert!(b(&[42]).read_varuint(7).unwrap() == 42);
        assert!(b(&[127]).read_varuint(7).unwrap() == 127);
        assert!(b(&[128, 1]).read_varuint(32).unwrap() == 128);
        assert!(b(&[255, 1]).read_varuint(32).unwrap() == 255);

        assert!(b(&[0]).read_varuint(32).unwrap() == 0);
        assert!(b(&[42]).read_varuint(32).unwrap() == 42);
        assert!(b(&[127]).read_varuint(32).unwrap() == 127);
        assert!(b(&[128, 1]).read_varuint(32).unwrap() == 128);
        assert!(b(&[255, 255, 3]).read_varuint(32).unwrap() == 0xffff);
        assert!(b(&[0xE5, 0x8E, 0x26]).read_varuint(32).unwrap() == 624485);
        assert!(b(&[0xff, 0xff, 0xff, 0xff, 0x0f]).read_varuint(32).unwrap() == 0xffff_ffff);
        assert!(b(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01]).read_varuint(64).unwrap() == 0xffff_ffff_ffff_ffff);
    }

    #[test]
    fn test_signed_decode() {
        assert!(b(&[0]).read_varint(1).unwrap() == 0);
        assert!(b(&[1]).read_varint(1).unwrap() == 1);
        assert!(b(&[2]).read_varint(7).unwrap() == 2);
        assert!(b(&[0x7e]).read_varint(7).unwrap() == -2);
        assert!(b(&[0xff, 0]).read_varint(16).unwrap() == 127);
        assert!(b(&[0x81, 0x7f]).read_varint(16).unwrap() == -127);
        assert!(b(&[0x80, 0x7f]).read_varint(16).unwrap() == -128);
    }

    // #[test]
    // #[should_panic]
    // fn test_decode_overflow_u1() {
    //     b(&[2]).read_varuint(1).unwrap();
    // }
    #[test]
    #[should_panic]
    fn test_decode_empty_buffer() {
        b(&[]).read_varuint(1).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_decode_overflow_u7() {
        b(&[128]).read_varuint(7).unwrap();
    }

    // #[test]
    // #[should_panic]
    // fn test_decode_overflow_u8() {
    //     b(&[128, 2]).read_varuint(8).unwrap();
    // }

    // #[test]
    // #[should_panic]
    // fn test_decode_overflow_u16() {
    //     b(&[128, 128, 4]).read_varuint(16).unwrap();
    // }

    // #[test]
    // #[should_panic]
    // fn test_decode_overflow_u32() {
    //     b(&[0x80, 0x80, 0x80, 0x80, 0x07]).read_varuint(32).unwrap();
    // }

    // #[test]
    // #[should_panic]
    // fn test_decode_overflow_u64() {
    //     b(&[128, 128, 128, 128, 128, 128, 128, 128, 128, 2]).read_varuint(64).unwrap();
    // }

    #[test]
    #[should_panic]
    fn test_decode_overflow_i8() {
        b(&[128, 2]).read_varint(8).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_decode_overflow_i16() {
        b(&[128, 128, 4]).read_varint(16).unwrap();
    }

    // #[test]
    // #[should_panic]
    // fn test_decode_overflow_i32() {
    //     b(&[0x80, 0x80, 0x80, 0x80, 0x07]).read_varint(32).unwrap();
    // }

    #[test]
    fn test_decode_i32() {
        // println!("dum dum {:?}", b(&[0x80, 0x80, 0x80, 0x80, 0x78]).read_varint(32).unwrap());
        // panic!();
        assert!(b(&[0x80, 0x80, 0x80, 0x80, 0x78]).read_varint(32).unwrap() == -2147483648);
    }

    #[test]
    #[should_panic]
    fn test_decode_overflow_i64() {
        b(&[128, 128, 128, 128, 128, 128, 128, 128, 128, 2]).read_varint(64).unwrap();
    }
}
