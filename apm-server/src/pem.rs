use anyhow::Result;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};

const fn make_decode_table() -> [u8; 256] {
    let mut t = [0xFFu8; 256];
    let alpha = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut i = 0usize;
    while i < 64 {
        t[alpha[i] as usize] = i as u8;
        i += 1;
    }
    t
}
const DECODE_TABLE: [u8; 256] = make_decode_table();

fn base64_decode(s: &str) -> Result<Vec<u8>> {
    let chars: Vec<u8> = s.bytes().filter(|&b| !b.is_ascii_whitespace()).collect();
    let len = chars.len();
    if len % 4 != 0 {
        return Err(anyhow::anyhow!("invalid base64 length"));
    }
    let mut out = Vec::with_capacity(len / 4 * 3);
    let mut i = 0;
    while i < len {
        let a = chars[i];
        let b = chars[i + 1];
        let c = chars[i + 2];
        let d = chars[i + 3];

        let va = DECODE_TABLE[a as usize];
        let vb = DECODE_TABLE[b as usize];
        if va == 0xFF || vb == 0xFF {
            return Err(anyhow::anyhow!("invalid base64 character"));
        }
        out.push((va << 2) | (vb >> 4));

        if c != b'=' {
            let vc = DECODE_TABLE[c as usize];
            if vc == 0xFF {
                return Err(anyhow::anyhow!("invalid base64 character"));
            }
            out.push((vb << 4) | (vc >> 2));

            if d != b'=' {
                let vd = DECODE_TABLE[d as usize];
                if vd == 0xFF {
                    return Err(anyhow::anyhow!("invalid base64 character"));
                }
                out.push((vc << 6) | vd);
            }
        }

        i += 4;
    }
    Ok(out)
}

fn pem_blocks(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let text = String::from_utf8_lossy(bytes);
    let mut blocks = Vec::new();
    let mut current_label: Option<String> = None;
    let mut body = String::new();

    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("-----BEGIN ") {
            if let Some(lbl) = rest.strip_suffix("-----") {
                current_label = Some(lbl.to_string());
                body.clear();
            }
        } else if let Some(rest) = line.strip_prefix("-----END ") {
            if let Some(lbl) = rest.strip_suffix("-----") {
                if current_label.as_deref() == Some(lbl) {
                    if let Ok(decoded) = base64_decode(&body) {
                        blocks.push((lbl.to_string(), decoded));
                    }
                    current_label = None;
                    body.clear();
                }
            }
        } else if current_label.is_some() {
            body.push_str(line);
        }
    }

    blocks
}

pub fn parse_certs(bytes: &[u8]) -> Result<Vec<CertificateDer<'static>>> {
    let blocks = pem_blocks(bytes);
    let certs: Vec<CertificateDer<'static>> = blocks
        .into_iter()
        .filter(|(label, _)| label == "CERTIFICATE")
        .map(|(_, der)| CertificateDer::from(der))
        .collect();
    Ok(certs)
}

pub fn parse_private_key(bytes: &[u8]) -> Result<Option<PrivateKeyDer<'static>>> {
    let blocks = pem_blocks(bytes);
    for (label, der) in blocks {
        match label.as_str() {
            "PRIVATE KEY" => return Ok(Some(PrivateKeyDer::Pkcs8(der.into()))),
            "RSA PRIVATE KEY" => return Ok(Some(PrivateKeyDer::Pkcs1(der.into()))),
            "EC PRIVATE KEY" => return Ok(Some(PrivateKeyDer::Sec1(der.into()))),
            _ => {}
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_decode_hello_world() {
        // "hello" -> "aGVsbG8="
        let decoded = base64_decode("aGVsbG8=").unwrap();
        assert_eq!(decoded, b"hello");
    }

    #[test]
    fn base64_decode_padding_two() {
        // "he" -> "aGU="
        let decoded = base64_decode("aGU=").unwrap();
        assert_eq!(decoded, b"he");
    }

    #[test]
    fn base64_decode_no_padding() {
        // "hel" -> "aGVs"
        let decoded = base64_decode("aGVs").unwrap();
        assert_eq!(decoded, b"hel");
    }

    #[test]
    fn base64_decode_skips_whitespace() {
        let decoded = base64_decode("aGVs\nbG8=").unwrap();
        assert_eq!(decoded, b"hello");
    }

    #[test]
    fn base64_decode_invalid_char_errors() {
        assert!(base64_decode("aGVs!G8=").is_err());
    }

    #[test]
    fn pem_blocks_certificate() {
        // Minimal fake PEM block (base64 of 3 zero bytes = "AAAA")
        let pem = "-----BEGIN CERTIFICATE-----\nAAAA\n-----END CERTIFICATE-----\n";
        let blocks = pem_blocks(pem.as_bytes());
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].0, "CERTIFICATE");
        assert_eq!(blocks[0].1, &[0, 0, 0]);
    }

    #[test]
    fn pem_blocks_multiple_types() {
        let pem = concat!(
            "-----BEGIN CERTIFICATE-----\nAAAA\n-----END CERTIFICATE-----\n",
            "-----BEGIN PRIVATE KEY-----\nAAAA\n-----END PRIVATE KEY-----\n",
        );
        let blocks = pem_blocks(pem.as_bytes());
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].0, "CERTIFICATE");
        assert_eq!(blocks[1].0, "PRIVATE KEY");
    }

    #[test]
    fn pem_blocks_ignores_garbage_lines() {
        let pem = "not a pem line\n-----BEGIN CERTIFICATE-----\nAAAA\n-----END CERTIFICATE-----\n";
        let blocks = pem_blocks(pem.as_bytes());
        assert_eq!(blocks.len(), 1);
    }

    #[test]
    fn parse_certs_empty_for_no_certificate_blocks() {
        let pem = b"-----BEGIN PRIVATE KEY-----\nAAAA\n-----END PRIVATE KEY-----\n";
        let certs = parse_certs(pem).unwrap();
        assert!(certs.is_empty());
    }

    #[test]
    fn parse_private_key_none_when_absent() {
        let pem = b"-----BEGIN CERTIFICATE-----\nAAAA\n-----END CERTIFICATE-----\n";
        let key = parse_private_key(pem).unwrap();
        assert!(key.is_none());
    }
}
