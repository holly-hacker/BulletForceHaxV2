use std::sync::Arc;

use futures_util::lock::Mutex;
use photon_lib::photon_message::PhotonMessage;
use tracing::debug;

use crate::hax::HaxState;

impl HaxState {
    #[allow(clippy::ptr_arg)]
    pub fn webrequest_hook_onrequest(
        _hax: Arc<Mutex<Self>>,
        _url: &hyper::Uri,
        _bytes: &mut Vec<u8>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    #[allow(clippy::ptr_arg)]
    pub fn webrequest_hook_onresponse(
        _hax: Arc<Mutex<Self>>,
        _url: &hyper::Uri,
        _bytes: &mut Vec<u8>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    #[allow(clippy::ptr_arg)]
    pub fn websocket_hook(_hax: Arc<Mutex<Self>>, data: &mut Vec<u8>, direction: &'static str) {
        let mut bytes = bytes::Bytes::copy_from_slice(data);
        let deserialized = PhotonMessage::from_websocket_bytes(&mut bytes);
        debug!("{direction} data: {deserialized:?}");
    }
}
