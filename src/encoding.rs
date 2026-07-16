pub fn decode(bytes: &[u8], encoding: &str) -> Result<String, String> {
    match encoding.to_lowercase().replace("-", "").replace("_", "") {
        e if e == "utf8" || e == "utf-8" => {
            String::from_utf8(bytes.to_vec()).map_err(|e| {
                format!("UnicodeDecodeError: 'utf-8' codec cannot decode byte {:x?} in position {}: invalid start byte", bytes[e.utf8_error().valid_up_to()..].first().unwrap_or(&0), e.utf8_error().valid_up_to())
            })
        }
        e if e == "ascii" => {
            if bytes.iter().all(|&b| b < 128) {
                unsafe { Ok(String::from_utf8_unchecked(bytes.to_vec())) }
            } else {
                let pos = bytes.iter().position(|&b| b >= 128).unwrap_or(0);
                Err(format!("UnicodeDecodeError: 'ascii' codec cannot decode byte {:x?} in position {}: ordinal not in range(128)", bytes[pos], pos))
            }
        }
        e if e == "latin1" || e == "latin-1" || e == "iso8859-1" || e == "iso-8859-1" => {
            Ok(bytes.iter().map(|&b| b as char).collect())
        }
        e if e == "utf16" || e == "utf-16" => {
            if bytes.len() < 2 {
                return Err("UnicodeDecodeError: 'utf-16' codec cannot decode 0 bytes in position 0: truncated data".to_string());
            }
            let (bom, little_endian) = if bytes[0] == 0xFF && bytes[1] == 0xFE {
                (2, true)
            } else if bytes[0] == 0xFE && bytes[1] == 0xFF {
                (2, false)
            } else {
                (0, false)
            };
            let mut chars = Vec::new();
            let mut i = bom;
            while i + 1 < bytes.len() {
                let code_unit = if little_endian {
                    bytes[i] as u16 | (bytes[i + 1] as u16) << 8
                } else {
                    (bytes[i] as u16) << 8 | bytes[i + 1] as u16
                };
                i += 2;
                if code_unit >= 0xD800 && code_unit <= 0xDBFF {
                    if i + 1 < bytes.len() {
                        let low = if little_endian {
                            bytes[i] as u16 | (bytes[i + 1] as u16) << 8
                        } else {
                            (bytes[i] as u16) << 8 | bytes[i + 1] as u16
                        };
                        i += 2;
                        let cp = 0x10000 + ((code_unit - 0xD800) as u32) * 0x400 + (low - 0xDC00) as u32;
                        if let Some(c) = char::from_u32(cp) {
                            chars.push(c);
                        }
                    }
                } else if code_unit >= 0xDC00 && code_unit <= 0xDFFF {
                    return Err(format!("UnicodeDecodeError: 'utf-16' codec cannot decode unexpected low surrogate at position {}", i - 2));
                } else {
                    chars.push(char::from_u32(code_unit as u32).unwrap_or('\u{FFFD}'));
                }
            }
            Ok(chars.into_iter().collect())
        }
        e if e == "utf16be" || e == "utf-16be" => {
            decode(bytes, "utf-16")
        }
        e if e == "utf16le" || e == "utf-16le" => {
            decode(bytes, "utf-16")
        }
        _ => {
            Err(format!("LookupError: unknown encoding: '{}'", encoding))
        }
    }
}

pub fn encode(text: &str, encoding: &str) -> Result<Vec<u8>, String> {
    match encoding.to_lowercase().replace("-", "").replace("_", "") {
        e if e == "utf8" || e == "utf-8" => {
            Ok(text.as_bytes().to_vec())
        }
        e if e == "ascii" => {
            if text.is_ascii() {
                Ok(text.as_bytes().to_vec())
            } else {
                let pos = text.chars().position(|c| !c.is_ascii()).unwrap_or(0);
                Err(format!("UnicodeEncodeError: 'ascii' codec cannot encode character '{}' in position {}: ordinal not in range(128)", text.chars().nth(pos).unwrap(), pos))
            }
        }
        e if e == "latin1" || e == "latin-1" || e == "iso8859-1" || e == "iso-8859-1" => {
            Ok(text.chars().map(|c| if c as u32 <= 0xFF { c as u8 } else { b'?' }).collect())
        }
        _ => {
            Err(format!("LookupError: unknown encoding: '{}'", encoding))
        }
    }
}
