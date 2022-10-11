use super::BulletForceHax;

impl BulletForceHax {
    /// Creates the webrequest proxy handler thread. Panics if one has already been created.
    pub fn start_webrequest_proxy(&mut self) {
        if self.webrequest_proxy.is_some() {
            panic!("webrequest proxy is already enabled");
        }

        let state = self.state.clone();
        tokio::spawn(async move {
            crate::proxy::webrequest_proxy::block_on_server(state).await;
        });

        self.webrequest_proxy = Some(());
    }
}
