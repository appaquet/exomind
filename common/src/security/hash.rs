pub struct Hash {
    hash_type: HashType,
    data: Vec<u8>,
}

pub enum HashType {
    Sha256,
}

trait Digest {
    fn consume(&mut self, data: &[u8]);
    fn result(&mut self) -> HashType;
}

struct Sha256Digest {}
