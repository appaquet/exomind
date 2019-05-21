// TODO: Key has a fixed u8 size, which is max of all key size

pub struct PrivateKey {
    _key_type: KeyType,
}

pub struct PublicKey {}

pub enum KeyType {
    RSA,
    SECP256K1,
}
