use crate::software_vault::SoftwareVault;
use crate::VaultError;
use ockam_core::hex::encode;
use ockam_vault_core::{Hasher, KeyId, KeyIdVault, PublicKey, Secret};

impl KeyIdVault for SoftwareVault {
    fn get_secret_by_key_id(&mut self, key_id: &str) -> ockam_core::Result<Secret> {
        let index = self
            .entries
            .iter()
            .find(|(_, entry)| {
                if let Some(e_key_id) = entry.key_id() {
                    e_key_id == key_id
                } else {
                    false
                }
            })
            .ok_or_else(|| Into::<ockam_core::Error>::into(VaultError::SecretNotFound))?
            .0;

        println!("fjfj");

        Ok(Secret::new(*index))
    }

    fn compute_key_id_for_public_key(
        &mut self,
        public_key: &PublicKey,
    ) -> ockam_core::Result<KeyId> {
        let key_id = self.sha256(public_key.as_ref())?;
        Ok(encode(key_id))
    }
}

#[cfg(test)]
mod tests {
    use crate::SoftwareVault;
    use ockam_vault_test_attribute::*;

    fn new_vault() -> SoftwareVault {
        SoftwareVault::default()
    }

    #[vault_test]
    fn compute_key_id_for_public_key() {}

    #[vault_test]
    fn get_secret_by_key_id() {}
}
