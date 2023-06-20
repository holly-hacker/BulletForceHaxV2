use shared::{FeatureSettings, HaxStateUpdate};

pub struct UiContext<'ipc> {
    pub state: &'ipc HaxStateUpdate,
    pub settings: FeatureSettings,
    settings_is_dirty: bool,
}

impl<'ipc> UiContext<'ipc> {
    pub fn mark_settings_dirty(&mut self) {
        self.settings_is_dirty = true;
    }

    pub fn is_settings_dirty(&mut self) -> bool {
        self.settings_is_dirty
    }

    pub(crate) fn new(
        state: &'ipc HaxStateUpdate,
        settings: &'ipc FeatureSettings,
    ) -> UiContext<'ipc> {
        Self {
            state,
            settings: settings.clone(),
            settings_is_dirty: false,
        }
    }
}
