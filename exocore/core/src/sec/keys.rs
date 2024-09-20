use libp2p_identity::{
    ed25519 as libp2p_ed25519, secp256k1 as libp2p_secp256k1, Keypair as libp2p_Keypair,
    PublicKey as libp2p_PublicKey,
};
use petname::Generator;
use rand::SeedableRng;

const ENCODE_KEYPAIR_CODE: u8 = b'a';
const ENCODE_PUBLIC_KEY_CODE: u8 = b'p';

/// Private and public keypair used for nodes and cells.
#[derive(Clone)]
pub struct Keypair {
    keypair: libp2p_Keypair,
    keypair_type: KeypairType,
}

#[derive(Clone)]
enum KeypairType {
    Ed25519 { keypair: libp2p_ed25519::Keypair },
    Secp256k1 { _keypair: libp2p_secp256k1::Keypair },
}

impl Keypair {
    pub fn from_libp2p(keypair: libp2p_Keypair) -> Keypair {
        let keypair_type = match keypair.key_type() {
            libp2p_identity::KeyType::Ed25519 => KeypairType::Ed25519 {
                keypair: keypair.clone().try_into_ed25519().unwrap(),
            },
            libp2p_identity::KeyType::Secp256k1 => KeypairType::Secp256k1 {
                _keypair: keypair.clone().try_into_secp256k1().unwrap(),
            },
            _ => unimplemented!(),
        };

        Keypair {
            keypair,
            keypair_type,
        }
    }

    pub fn generate_ed25519() -> Keypair {
        Self::from_libp2p(libp2p_Keypair::generate_ed25519())
    }

    pub fn public(&self) -> PublicKey {
        PublicKey::from_libp2p(self.keypair.public())
    }

    pub fn algorithm(&self) -> Algorithm {
        match self.keypair_type {
            KeypairType::Ed25519 { keypair: _ } => Algorithm::Ed25519,
            KeypairType::Secp256k1 { _keypair: _ } => Algorithm::Secp256k1,
        }
    }

    pub fn to_libp2p(&self) -> &libp2p_Keypair {
        &self.keypair
    }

    /// Sign given message with the keypair.
    /// The `verify` method on the public key can be used to validate signature.
    pub fn sign(&self, msg: &[u8]) -> Result<Vec<u8>, Error> {
        self.keypair
            .sign(msg)
            .map_err(|err| Error::Libp2pSigning(err.to_string()))
    }

    /// Encode the keypair into a bytes representation.
    pub fn encode(&self) -> Vec<u8> {
        match &self.keypair_type {
            KeypairType::Ed25519 { keypair } => {
                let mut vec = vec![0; 66];
                vec[0] = ENCODE_KEYPAIR_CODE;
                vec[1] = Algorithm::Ed25519.to_code();
                vec[2..].copy_from_slice(&keypair.to_bytes());
                vec
            }
            _ => unimplemented!(),
        }
    }

    /// Encode the keypair into a base58 representation
    pub fn encode_base58_string(&self) -> String {
        encode_base58(&self.encode())
    }

    /// Decodes given bytes into a keypair.
    /// The method takes a mutable slice since libp2p zeroize it afterward.
    pub fn decode(bytes: &mut [u8]) -> Result<Keypair, Error> {
        if bytes.len() < 3 {
            return Err(Error::DecodeInvalidSize);
        }

        if bytes[0] != ENCODE_KEYPAIR_CODE {
            return Err(Error::DecodeExpectedPair);
        }

        match Algorithm::from_code(bytes[1])? {
            Algorithm::Ed25519 => {
                let keypair = libp2p_ed25519::Keypair::try_from_bytes(&mut bytes[2..])
                    .map_err(|err| Error::Libp2pDecode(err.to_string()))?;

                Ok(Self::from_libp2p(keypair.into()))
            }
            _ => unimplemented!(),
        }
    }

    /// Decode given a base58 represented string into a keypair.
    pub fn decode_base58_string(input: &str) -> Result<Keypair, Error> {
        let mut bytes = decode_base58(input)?;
        Self::decode(&mut bytes)
    }
}

/// Public key
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicKey {
    key: libp2p_PublicKey,
    key_type: PublicKeyType,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PublicKeyType {
    Ed25519(libp2p_ed25519::PublicKey),
    Secp256k1(libp2p_secp256k1::PublicKey),
}

impl PublicKey {
    pub fn from_libp2p(key: libp2p_PublicKey) -> PublicKey {
        let key_type = match key.key_type() {
            libp2p_identity::KeyType::Ed25519 => {
                PublicKeyType::Ed25519(key.clone().try_into_ed25519().unwrap())
            }
            libp2p_identity::KeyType::Secp256k1 => {
                PublicKeyType::Secp256k1(key.clone().try_into_secp256k1().unwrap())
            }
            _ => unimplemented!(),
        };

        PublicKey { key, key_type }
    }

    pub fn to_libp2p(&self) -> &libp2p_PublicKey {
        &self.key
    }

    /// Verify the message for authenticity (signed by key) and integrity (not
    /// tampered with).
    pub fn verify(&self, msg: &[u8], sig: &[u8]) -> bool {
        self.key.verify(msg, sig)
    }

    /// Encode the public key into a bytes representation.
    pub fn encode(&self) -> Vec<u8> {
        match &self.key_type {
            PublicKeyType::Ed25519(pk) => {
                let mut vec = vec![0; 34];
                vec[0] = ENCODE_PUBLIC_KEY_CODE;
                vec[1] = Algorithm::Ed25519.to_code();
                vec[2..].copy_from_slice(&pk.to_bytes());
                vec
            }
            _ => unimplemented!(),
        }
    }

    /// Encode the public key into a base58 representation
    pub fn encode_base58_string(&self) -> String {
        encode_base58(&self.encode())
    }

    /// Decodes given bytes into a public key.
    pub fn decode(bytes: &[u8]) -> Result<PublicKey, Error> {
        if bytes.len() < 3 {
            return Err(Error::DecodeInvalidSize);
        }

        if bytes[0] != ENCODE_PUBLIC_KEY_CODE {
            return Err(Error::DecodeExpectedPublic);
        }

        match Algorithm::from_code(bytes[1])? {
            Algorithm::Ed25519 => {
                let pk = libp2p_ed25519::PublicKey::try_from_bytes(&bytes[2..])
                    .map_err(|err| Error::Libp2pDecode(err.to_string()))?;

                Ok(PublicKey::from_libp2p(pk.into()))
            }
            _ => unimplemented!(),
        }
    }

    /// Decode given a base58 represented string into a public key.
    pub fn decode_base58_string(input: &str) -> Result<PublicKey, Error> {
        let bytes = decode_base58(input)?;
        Self::decode(&bytes)
    }

    /// Generates a deterministic random name from this public key
    pub fn generate_name(&self) -> String {
        let bytes = self.encode();
        let bytes_len = bytes.len();

        let mut rng = rand::prelude::StdRng::seed_from_u64(u64::from_le_bytes([
            bytes[bytes_len - 1],
            bytes[bytes_len - 2],
            bytes[bytes_len - 3],
            bytes[bytes_len - 4],
            bytes[bytes_len - 5],
            bytes[bytes_len - 6],
            bytes[bytes_len - 7],
            bytes[bytes_len - 8],
        ]));
        petname::Petnames::small()
            .generate(&mut rng, 3, "-")
            .expect("a petname to be generated")
    }
}

/// Convert key to base58 representation
fn encode_base58(bytes: &[u8]) -> String {
    format!(
        "{}{}{}",
        char::from(bytes[0]),
        char::from(bytes[1]),
        bs58::encode(&bytes[2..]).into_string()
    )
}

/// Convert base58 key representation to bytes
fn decode_base58(input: &str) -> Result<Vec<u8>, Error> {
    let input_bytes = input.as_bytes();
    if input_bytes.len() < 3 {
        return Err(Error::DecodeInvalidSize);
    }

    // see `bs58::decode::into_vec()`
    let mut output = vec![0; (input_bytes.len() / 8 + 1) * 6];
    output[0..2].copy_from_slice(&input_bytes[0..2]);

    let len = bs58::decode(&input[2..]).onto(&mut output[2..])?;

    output.truncate(len + 2);

    Ok(output)
}

/// Encryption / signature algorithm type
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Algorithm {
    Ed25519,
    Secp256k1,
}

impl Algorithm {
    fn to_code(self) -> u8 {
        match self {
            Algorithm::Ed25519 => b'e',
            Algorithm::Secp256k1 => b'c',
        }
    }

    fn from_code(code: u8) -> Result<Algorithm, Error> {
        match code {
            b'e' => Ok(Algorithm::Ed25519),
            b'c' => Ok(Algorithm::Secp256k1),
            _ => Err(Error::InvalidAlgorithmCode(code)),
        }
    }
}

/// Cryptographic keys related error
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Given bytes to decode doesn't have the right size")]
    DecodeInvalidSize,

    #[error("Given bytes to decode wasn't a keypair")]
    DecodeExpectedPair,

    #[error("Given bytes to decode wasn't a public key")]
    DecodeExpectedPublic,

    #[error("Given bytes couldn't be decoded by libp2p: {0}")]
    Libp2pDecode(String),

    #[error("Couldn't decode base58 string into bytes: {0}")]
    Base58Decode(#[from] bs58::decode::Error),

    #[error("Algorithm code is invalid: {0}")]
    InvalidAlgorithmCode(u8),

    #[error("Libp2p signing error: {0}")]
    Libp2pSigning(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn ed25519_keypair_encode_decode() -> anyhow::Result<()> {
        let keypair = Keypair::generate_ed25519();

        let mut encoded = keypair.encode();
        let keypair_decoded = Keypair::decode(&mut encoded)?;
        assert_eq!(keypair.public(), keypair_decoded.public());

        assert!(Keypair::decode(&mut []).is_err());
        assert!(Keypair::decode(&mut [0]).is_err());
        assert!(Keypair::decode(&mut [0, 0]).is_err());
        assert!(Keypair::decode(&mut [0, 0, 0]).is_err());

        Ok(())
    }

    #[test]
    pub fn ed25519_keypair_base58_encode_decode() -> anyhow::Result<()> {
        let keypair = Keypair::generate_ed25519();

        let encoded_bytes = keypair.encode();
        let encoded_base58 = encode_base58(&encoded_bytes);
        let decoded_base58 = decode_base58(&encoded_base58)?;
        assert_eq!(encoded_bytes, decoded_base58);

        let encoded = keypair.encode_base58_string();
        let keypair_decoded = Keypair::decode_base58_string(&encoded)?;
        assert_eq!(keypair.public(), keypair_decoded.public());

        assert!(Keypair::decode_base58_string("").is_err());
        assert!(Keypair::decode_base58_string("a").is_err());
        assert!(Keypair::decode_base58_string("ae").is_err());
        assert!(Keypair::decode_base58_string("aeb").is_err());

        Ok(())
    }

    #[test]
    pub fn ed25519_public_key_encode_decode() -> anyhow::Result<()> {
        let keypair = Keypair::generate_ed25519();

        let encoded = keypair.public().encode();
        let public_decoded = PublicKey::decode(&encoded)?;

        assert_eq!(keypair.public(), public_decoded);

        assert!(PublicKey::decode(&[]).is_err());
        assert!(PublicKey::decode(&[0]).is_err());
        assert!(PublicKey::decode(&[0, 0]).is_err());
        assert!(PublicKey::decode(&[0, 0, 0]).is_err());

        Ok(())
    }

    #[test]
    pub fn ed25519_public_key_base58_encode_decode() -> anyhow::Result<()> {
        let keypair = Keypair::generate_ed25519();

        let encoded_bytes = keypair.public().encode();
        let encoded_base58 = encode_base58(&encoded_bytes);
        let decoded_base58 = decode_base58(&encoded_base58)?;
        assert_eq!(encoded_bytes, decoded_base58);

        let encoded = keypair.public().encode_base58_string();
        let public_decoded = PublicKey::decode_base58_string(&encoded)?;
        assert_eq!(keypair.public(), public_decoded);

        assert!(PublicKey::decode_base58_string("").is_err());
        assert!(PublicKey::decode_base58_string("p").is_err());
        assert!(PublicKey::decode_base58_string("pe").is_err());
        assert!(PublicKey::decode_base58_string("peb").is_err());

        Ok(())
    }

    #[test]
    pub fn signature_and_verification() -> anyhow::Result<()> {
        let keypair = Keypair::generate_ed25519();

        let msg = String::from("hello world").into_bytes();
        let sig = keypair.sign(&msg)?;

        let pk = keypair.public();
        assert!(pk.verify(&msg, &sig));

        let mut invalid_sig = sig.clone();
        invalid_sig[5] = b'c';
        invalid_sig[6] = b'd';
        invalid_sig[7] = b'd';

        assert!(!pk.verify(&msg, &invalid_sig));

        let tampered_msg = String::from("h4x0r").into_bytes();
        assert!(!pk.verify(&tampered_msg, &sig));
        Ok(())
    }
}
