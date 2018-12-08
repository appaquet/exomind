const MAX_SIGNATURE_SIZE: usize = 1024;

pub struct Message {}

pub struct Signature {
    sig: [u8; MAX_SIGNATURE_SIZE],
}

impl Signature {
    fn validate(&self, _message: &Message) -> bool {
        unimplemented!()
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_signature() {}
}
