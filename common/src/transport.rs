#[derive(Copy, Clone, Debug)]
pub enum TransportLayer {
    Meta = 1,
    Common = 2,
    Data = 3,
}

impl TransportLayer {
    pub fn from_code(code: u8) -> Option<TransportLayer> {
        match code {
            1 => Some(TransportLayer::Meta),
            2 => Some(TransportLayer::Common),
            3 => Some(TransportLayer::Data),
            _ => None,
        }
    }

    pub fn to_code(self) -> u8 {
        self as u8
    }
}

impl Into<u8> for TransportLayer {
    fn into(self) -> u8 {
        self.to_code()
    }
}
