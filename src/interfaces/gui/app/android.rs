//! App для Android — точка входа android-activity.
//! Будет реализована при сборке под Android.

use std::sync::mpsc;

use crate::data_layer::EditorCommand;
use crate::interfaces::gui::intent::Intent;
use crate::interfaces::gui::model::Model;

/// Заглушка для Android — будет содержать android-activity логику.
pub struct GCodeApp {
    pub model: Model,
}

impl GCodeApp {
    pub fn new(
        _cmd_tx: mpsc::Sender<EditorCommand>,
        _evt_rx: mpsc::Receiver<crate::data_layer::EditorEvent>,
    ) -> Self {
        Self {
            model: Model::default(),
        }
    }

    pub fn handle_intent(&mut self, intent: &Intent) {
        self.model.apply(intent);
    }
}
