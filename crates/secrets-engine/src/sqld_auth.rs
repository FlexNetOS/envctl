//! Rust-native key material for sqld's JWT authentication mode.
//!
//! This is crypto-only engine code: it returns bytes and never prints or performs filesystem I/O.
//! The hidden `secretctl internal-sqld-auth-bootstrap` command owns the atomic 0600 write edge.

use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::Engine as _;
use ring::rand::{SecureRandom, SystemRandom};
use ring::signature::{Ed25519KeyPair, KeyPair};
use zeroize::{Zeroize, Zeroizing};

const ED25519_SPKI_PREFIX: [u8; 12] = [
    0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00,
];
const JWT_HEADER: &[u8] = br#"{"alg":"EdDSA","typ":"JWT"}"#;
const JWT_CLAIMS: &[u8] = b"{}";

/// Public verification key plus its matching bearer. The bearer zeroizes its allocation on drop.
pub struct SqldAuthMaterial {
    pub public_key_pem: Vec<u8>,
    pub client_jwt: Zeroizing<Vec<u8>>,
}

/// Generate a fresh Ed25519 key in memory and a matching EdDSA JWT over empty claims.
pub fn generate_sqld_auth_material() -> anyhow::Result<SqldAuthMaterial> {
    let rng = SystemRandom::new();
    let mut seed = Zeroizing::new([0_u8; 32]);
    rng.fill(seed.as_mut())
        .map_err(|_| anyhow::anyhow!("generating the sqld Ed25519 seed failed"))?;
    material_from_seed(&seed)
}

fn material_from_seed(seed: &[u8; 32]) -> anyhow::Result<SqldAuthMaterial> {
    let key_pair = Ed25519KeyPair::from_seed_unchecked(seed)
        .map_err(|_| anyhow::anyhow!("deriving the sqld Ed25519 key failed"))?;

    let mut spki =
        Vec::with_capacity(ED25519_SPKI_PREFIX.len() + key_pair.public_key().as_ref().len());
    spki.extend_from_slice(&ED25519_SPKI_PREFIX);
    spki.extend_from_slice(key_pair.public_key().as_ref());
    let public_key_pem = pem_public_key(&spki);

    let encoded_header = URL_SAFE_NO_PAD.encode(JWT_HEADER);
    let encoded_claims = URL_SAFE_NO_PAD.encode(JWT_CLAIMS);
    let mut signing_input = Zeroizing::new(format!("{encoded_header}.{encoded_claims}"));
    let signature = key_pair.sign(signing_input.as_bytes());
    let mut client_jwt = Zeroizing::new(signing_input.as_bytes().to_vec());
    client_jwt.push(b'.');
    client_jwt.extend_from_slice(URL_SAFE_NO_PAD.encode(signature.as_ref()).as_bytes());
    signing_input.zeroize();

    Ok(SqldAuthMaterial {
        public_key_pem,
        client_jwt,
    })
}

fn pem_public_key(spki: &[u8]) -> Vec<u8> {
    let encoded = STANDARD.encode(spki);
    let mut pem = Vec::with_capacity(encoded.len() + 64);
    pem.extend_from_slice(b"-----BEGIN PUBLIC KEY-----\n");
    for line in encoded.as_bytes().chunks(64) {
        pem.extend_from_slice(line);
        pem.push(b'\n');
    }
    pem.extend_from_slice(b"-----END PUBLIC KEY-----\n");
    pem
}

#[cfg(test)]
mod tests {
    use super::*;
    use ring::signature::{UnparsedPublicKey, ED25519};

    #[test]
    fn deterministic_seed_yields_spki_pem_and_verifiable_eddsa_jwt() {
        let first = material_from_seed(&[7_u8; 32]).expect("material");
        let second = material_from_seed(&[7_u8; 32]).expect("material");
        assert_eq!(first.public_key_pem, second.public_key_pem);
        assert_eq!(first.client_jwt.as_slice(), second.client_jwt.as_slice());

        let pem = std::str::from_utf8(&first.public_key_pem).expect("PEM UTF-8");
        assert!(pem.starts_with("-----BEGIN PUBLIC KEY-----\n"));
        assert!(pem.ends_with("-----END PUBLIC KEY-----\n"));
        let encoded: String = pem
            .lines()
            .filter(|line| !line.starts_with("-----"))
            .collect();
        let spki = STANDARD.decode(encoded).expect("SPKI base64");
        assert_eq!(&spki[..ED25519_SPKI_PREFIX.len()], &ED25519_SPKI_PREFIX);

        let jwt = std::str::from_utf8(&first.client_jwt).expect("JWT ASCII");
        let parts: Vec<_> = jwt.split('.').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(URL_SAFE_NO_PAD.decode(parts[0]).unwrap(), JWT_HEADER);
        assert_eq!(URL_SAFE_NO_PAD.decode(parts[1]).unwrap(), JWT_CLAIMS);
        let signature = URL_SAFE_NO_PAD.decode(parts[2]).expect("signature");
        let public_key = &spki[ED25519_SPKI_PREFIX.len()..];
        UnparsedPublicKey::new(&ED25519, public_key)
            .verify(format!("{}.{}", parts[0], parts[1]).as_bytes(), &signature)
            .expect("matching EdDSA signature");
    }
}
