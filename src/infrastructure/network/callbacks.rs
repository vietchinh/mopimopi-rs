//! How the network layer hands events to the application.

use super::connection_status::ConnectionStatus;
use crate::models::act_data::{ActEvent, CombatDataMessage};
use std::rc::Rc;

/// Functions the application provides; they are called from browser event handlers.
#[derive(Clone)]
pub struct ActEventCallbacks {
    pub on_combat_data: Rc<dyn Fn(CombatDataMessage)>,
    pub on_local_player_name: Rc<dyn Fn(String)>,
    pub on_status_change: Rc<dyn Fn(ConnectionStatus)>,
}

impl ActEventCallbacks {
    pub(super) fn dispatch(&self, event: ActEvent) {
        match event {
            ActEvent::CombatData(message) => (self.on_combat_data)(message),
            ActEvent::LocalPlayerName(name) => (self.on_local_player_name)(name),
        }
    }

    pub(super) fn report_status(&self, status: ConnectionStatus) {
        (self.on_status_change)(status);
    }
}
