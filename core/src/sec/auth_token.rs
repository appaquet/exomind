use crate::protos::core::{AuthToken as AuthTokenProto, AuthTokenData as AutoTokenDataProto};
use crate::protos::prost::{ProstDateTimeExt, ProstMessageExt, ProstTimestampExt};
use crate::{cell::Cell, cell::CellId, cell::NodeId, time::Clock, time::ConsistentTimestamp};
use prost::Message;

/// Authentication token that can be used as an alternative authentication
/// method for a node of a cell when using a transport authentified transport
/// like `libp2p`. Since not all clients can use a `libp2p` based transport, a
/// token can be used by a fat client to authentify further calls by a thi
/// client.
///
/// Ex: * a iOS client (fat) can create a token to be used by an app extension.
///     * a web extension can get a token from a running instance of exocore and
///       keep it in a storage for further calls.
pub struct AuthToken {
    cell_id: CellId,
    node_id: NodeId,
    signature_date: ConsistentTimestamp,
    expiration_date: Option<ConsistentTimestamp>,
    signed: AuthTokenProto,
}

impl AuthToken {
    /// Creates a new authentication token using the given cell's local node to
    /// sign the token. An optional expiration date can be given to enhance
    /// security to prevent long term usage of a potentially leaked token.
    pub fn new(
        cell: &Cell,
        clock: &Clock,
        expiration_date: Option<ConsistentTimestamp>,
    ) -> Result<AuthToken, Error> {
        let now = clock.consistent_time(cell.local_node());

        let data = AutoTokenDataProto {
            cell_id: cell.id().as_bytes().to_vec(),
            node_id: cell.local_node().id().as_bytes().to_vec(),
            signature_date: Some(now.to_proto_timestamp()),
            expiration_date: expiration_date.map(|d| d.to_proto_timestamp()),
        };

        let token_proto = data.encode_to_vec()?;
        let signature = cell.local_node().keypair().sign(&token_proto)?;

        let signed = AuthTokenProto {
            data: token_proto,
            signature,
        };

        Ok(AuthToken {
            cell_id: cell.id().to_owned(),
            node_id: cell.local_node().id().to_owned(),
            signature_date: clock.consistent_time(cell.local_node().node()),
            expiration_date,
            signed,
        })
    }

    /// Unmarshal a token from the given protobuf message.
    pub fn from_proto(token: AuthTokenProto) -> Result<AuthToken, Error> {
        let token_data: AutoTokenDataProto = AutoTokenDataProto::decode(token.data.as_slice())
            .map_err(crate::protos::Error::ProstDecodeError)?;

        let cell_id = CellId::from_bytes(&token_data.cell_id);
        let node_id = NodeId::from_bytes(token_data.node_id)
            .map_err(|err| Error::Invalid(format!("Invalid node id: {}", err)))?;
        let signature_date = token_data
            .signature_date
            .ok_or_else(|| Error::Invalid("Invalid token signature".to_string()))?
            .to_consistent_timestamp();
        let expiration_date = token_data
            .expiration_date
            .map(|d| d.to_consistent_timestamp());

        Ok(AuthToken {
            cell_id,
            node_id,
            signature_date,
            expiration_date,
            signed: token,
        })
    }

    /// Unmarshals a token from the given base58 encoded protobuf message.
    pub fn decode_base58_string(token: &str) -> Result<AuthToken, Error> {
        let token_proto_bytes = bs58::decode(token).into_vec()?;
        let token_proto = AuthTokenProto::decode(token_proto_bytes.as_slice())
            .map_err(crate::protos::Error::ProstDecodeError)?;

        Self::from_proto(token_proto)
    }

    /// Returns cell identifier from the token.
    pub fn cell_id(&self) -> &CellId {
        &self.cell_id
    }

    /// Returns node identifier from which the otken was signed.
    pub fn node_id(&self) -> &NodeId {
        &self.node_id
    }

    /// Returns protocol buffer representation of the token.
    pub fn as_proto(&self) -> &AuthTokenProto {
        &self.signed
    }

    /// Encodes the signed token into a base58 representation.
    pub fn encode_base58_string(&self) -> String {
        let signed_encoded = self
            .signed
            .encode_to_vec()
            .expect("Couldn't encore signed token");
        bs58::encode(&signed_encoded).into_string()
    }

    /// Validates the token signature & expiration date.
    pub fn is_valid(&self, cell: &Cell, clock: &Clock) -> Result<(), Error> {
        let cell_nodes = cell.nodes();
        let cell_node = cell_nodes
            .nodes
            .get(&self.node_id)
            .ok_or(Error::UnknownNode)?;
        let node = cell_node.node();

        if let Some(expiration_date) = self.expiration_date {
            let now = clock.consistent_time(node);
            if expiration_date <= now {
                return Err(Error::Expired);
            }
        }

        let signature_valid = node
            .public_key()
            .verify(&self.signed.data, &self.signed.signature);

        if signature_valid {
            Ok(())
        } else {
            Err(Error::InvalidSignature)
        }
    }
}

impl PartialEq for AuthToken {
    fn eq(&self, other: &Self) -> bool {
        self.signed.eq(&other.signed)
    }
}

impl std::fmt::Debug for AuthToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthToken")
            .field("cell_id", &self.cell_id)
            .field("node_id", &self.node_id)
            .field("signature_date", &self.signature_date)
            .field("expiration_date", &self.expiration_date)
            .finish()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Signature error: {0}")]
    KeyError(#[from] crate::sec::keys::Error),

    #[error("Invalid information in token: {0}")]
    Invalid(String),

    #[error("Proto serialization error: {0}")]
    ProtoSerialization(#[from] crate::protos::Error),

    #[error("Base58 decoding error: {0}")]
    Base58Decoding(#[from] bs58::decode::Error),

    #[error("Token was signed by an unknown node")]
    UnknownNode,

    #[error("Token signatute is invalid")]
    InvalidSignature,

    #[error("Token has expired")]
    Expired,
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::cell::{FullCell, LocalNode};
    use crate::time::Instant;

    #[test]
    fn token_validity() -> anyhow::Result<()> {
        let node = LocalNode::generate();
        let cell = FullCell::generate(node);
        let clock = Clock::new();

        let mut token = AuthToken::new(cell.cell(), &clock, None)?;
        assert!(token.is_valid(cell.cell(), &clock).is_ok());

        // modify signature, shouldn't be valid anymore
        token.signed.signature = vec![0, 1, 2, 3];
        assert!(token.is_valid(cell.cell(), &clock).is_err());

        // clear signature, shouldn't be valid anymore
        token.signed.signature.clear();
        assert!(token.is_valid(cell.cell(), &clock).is_err());

        Ok(())
    }

    #[test]
    fn token_expiration() -> anyhow::Result<()> {
        let node = LocalNode::generate();
        let cell = FullCell::generate(node.clone());

        let now = Instant::now();
        let clock = Clock::new_fixed_mocked(now);

        let expiry = clock.consistent_time(node.node()) + Duration::from_millis(100);

        let token = AuthToken::new(cell.cell(), &clock, Some(expiry))?;

        // shouldn't be expired
        assert!(token.is_valid(cell.cell(), &clock).is_ok());

        // should now be expired
        clock.set_fixed_instant(now + Duration::from_millis(100));
        assert!(token.is_valid(cell.cell(), &clock).is_err());

        Ok(())
    }

    #[test]
    fn token_encoding() -> anyhow::Result<()> {
        let node = LocalNode::generate();
        let cell = FullCell::generate(node.clone());

        let clock = Clock::new();
        let expiry = clock.consistent_time(node.node());

        let token = AuthToken::new(cell.cell(), &clock, Some(expiry))?;
        let bs58_encoded = token.encode_base58_string();
        let token_decoded = AuthToken::decode_base58_string(&bs58_encoded)?;
        assert_eq!(token, token_decoded);

        Ok(())
    }
}
