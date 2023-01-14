use crate::channel::common::SecureChannelKeyExchanger;
use crate::channel::encryption_helper::{DecryptionHelper, EncryptionHelper};
use crate::{IdentityIdentifier, IdentityVault};

pub(crate) struct KeyExchange<K: SecureChannelKeyExchanger> {
    pub key_exchanger: K,
}

pub(crate) struct ExchangeIdentity<V: IdentityVault> {
    pub encryption_helper: EncryptionHelper<V>,
    pub decryption_helper: DecryptionHelper<V>,
    pub auth_hash: [u8; 32],
    pub identity_sent: bool,
    pub received_identity_id: Option<IdentityIdentifier>,
}

pub(crate) struct Initialized<V: IdentityVault> {
    pub decryption_helper: DecryptionHelper<V>,
    pub their_identity_id: IdentityIdentifier,
}

pub(crate) enum State {
    KeyExchange,
    ExchangeIdentity,
    Initialized,
}
