// from fbernier/base62
const BASE: u128 = 62;
const ALPHABET: [u8; BASE as usize] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9',
    b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J',
    b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T',
    b'U', b'V', b'W', b'X', b'Y', b'Z', b'a', b'b', b'c', b'd',
    b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n',
    b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x',
    b'y', b'z'
];


pub fn encode(mut number: u128) -> String {
    let mut buffer: Vec<u8> = vec![];
    if number == 0 {
        return "0".to_string();
    }
    while number > 0 {
        buffer.push(ALPHABET[(number % BASE) as usize]);
        number /= BASE;
    }
    buffer.reverse();
    String::from_utf8(buffer).unwrap()
}

pub fn decode(string: &str) -> Option<u128> {
    let mut result: u128 = 0;
    if string.len() > 22 {
        return None;
    }
    for (i, c) in string.as_bytes().iter().rev().enumerate() {
        let num = BASE.pow(i as u32);
        result += ALPHABET
            .binary_search(c).ok()
            .and_then(|v| (v as u128).checked_mul(num))?;
    }
    Some(result)
}


#[cfg(test)]
mod tests {
    use super::{decode, encode};

    #[test]
    fn test_encode() {
        assert_eq!(encode(0), "0");
        assert_eq!(encode(34441886726), "base62");
    }

    #[test]
    fn test_decode() {
        assert_eq!(decode("0"), Some(0));
        assert_eq!(decode("1"), Some(1));
        assert_eq!(decode("10"), Some(62));
        assert_eq!(decode("zz"), Some(3843));
    }
}