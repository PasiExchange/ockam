use crate::channel::addresses::Addresses;
use crate::channel::common::SecureChannelVault;
use crate::channel::encryption_helper::EncryptionHelper;
use ockam_core::compat::boxed::Box;
use ockam_core::{async_trait, Address, Encodable, Route};
use ockam_core::{Any, Result, Routed, TransportMessage, Worker};
use ockam_node::Context;
use tracing::debug;

pub(crate) struct EncryptorWorker<V: SecureChannelVault> {
    addresses: Addresses,
    remote_route: Route,
    remote_backwards_compatibility_address: Address,
    encryption_helper: EncryptionHelper<V>,
}

impl<V: SecureChannelVault> EncryptorWorker<V> {
    pub fn new(
        addresses: Addresses,
        remote_route: Route,
        remote_backwards_compatibility_address: Address,
        encryption_helper: EncryptionHelper<V>,
    ) -> Self {
        Self {
            addresses,
            remote_route,
            remote_backwards_compatibility_address,
            encryption_helper,
        }
    }

    async fn handle_encrypt(
        &mut self,
        ctx: &mut <Self as Worker>::Context,
        msg: Routed<<Self as Worker>::Message>,
    ) -> Result<()> {
        debug!("SecureChannel received Encrypt");

        let mut onward_route = msg.onward_route();
        let mut return_route = msg.return_route();
        let transport_message = msg.into_transport_message();
        let payload = transport_message.payload;

        // Remove our address
        let _ = onward_route.step();
        // Add backwards compatibility address to simulate old behaviour
        onward_route
            .modify()
            .prepend(self.remote_backwards_compatibility_address.clone());

        // Add backwards compatibility address to simulate old behaviour
        return_route
            .modify()
            .prepend(self.addresses.decryptor_backwards_compatibility.clone());

        let msg = TransportMessage::v1(onward_route, return_route, payload.to_vec());
        let payload = msg.encode()?;

        let payload = self.encryption_helper.encrypt(payload.as_slice()).await?;

        ctx.send_from_address(
            self.remote_route.clone(),
            payload,
            self.addresses.encryptor.clone(),
        )
        .await
    }
}

#[async_trait]
impl<V: SecureChannelVault> Worker for EncryptorWorker<V> {
    type Message = Any;
    type Context = Context;

    async fn handle_message(
        &mut self,
        ctx: &mut Self::Context,
        msg: Routed<Self::Message>,
    ) -> Result<()> {
        self.handle_encrypt(ctx, msg).await
    }
}
