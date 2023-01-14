use crate::error::IdentityError;
use ockam_core::compat::vec::Vec;
use ockam_core::vault::{KeyId, SymmetricVault};
use ockam_core::Result;

pub(crate) struct EncryptionHelper<V: SymmetricVault> {
    key: KeyId,
    nonce: u64,
    vault: V,
}

impl<V: SymmetricVault> EncryptionHelper<V> {
    /// We use u64 nonce since it's convenient to work with it (e.g. increment)
    /// But we use 8-byte be format to send it over to the other side (according to noise spec)
    /// And we use 12-byte be format for encryption, since AES-GCM wants 12 bytes
    pub(crate) fn convert_nonce_from_u64(nonce: u64) -> ([u8; 8], [u8; 12]) {
        let mut n: [u8; 12] = [0; 12];
        let b: [u8; 8] = nonce.to_be_bytes();

        n[4..].copy_from_slice(&b);

        (b, n)
    }

    /// Restore 12-byte nonce needed for AES GCM from 8 byte that we use for noise
    fn convert_nonce_from_small(b: &[u8]) -> Result<[u8; 12]> {
        let bytes: [u8; 8] = b.try_into().map_err(|_| IdentityError::InvalidNonce)?;

        let nonce = u64::from_be_bytes(bytes);

        Ok(Self::convert_nonce_from_u64(nonce).1)
    }

    pub async fn encrypt(&mut self, payload: &[u8]) -> Result<Vec<u8>> {
        let old_nonce = self.nonce;
        if old_nonce == u64::MAX {
            return Err(IdentityError::InvalidNonce.into());
        }

        self.nonce += 1;

        let (small_nonce, nonce) = Self::convert_nonce_from_u64(old_nonce);

        let mut cipher_text = self
            .vault
            .aead_aes_gcm_encrypt(&self.key, payload, &nonce, &[])
            .await?;

        let mut res = Vec::new();
        res.extend_from_slice(&small_nonce);
        res.append(&mut cipher_text);

        Ok(res)
    }

    pub fn new(key: KeyId, nonce: u64, vault: V) -> Self {
        Self { key, nonce, vault }
    }
}

#[derive(Clone)]
pub(crate) struct DecryptionHelper<V: SymmetricVault> {
    key: KeyId,
    vault: V,
}

impl<V: SymmetricVault> DecryptionHelper<V> {
    pub async fn decrypt(&self, payload: &[u8]) -> Result<Vec<u8>> {
        if payload.len() < 8 {
            return Err(IdentityError::InvalidNonce.into());
        }

        let nonce = EncryptionHelper::<V>::convert_nonce_from_small(&payload[..8])?;

        self.vault
            .aead_aes_gcm_decrypt(&self.key, &payload[8..], &nonce, &[])
            .await
    }
    pub fn new(key: KeyId, vault: V) -> Self {
        Self { key, vault }
    }
}
