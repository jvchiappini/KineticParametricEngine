use bevy::prelude::*;
use crate::app;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(app::AppState::new())
            .add_systems(Update, app::ui_system);
    }
}
