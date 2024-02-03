use bevy::{prelude::*, sprite::Material2dPlugin};

mod assets;
pub mod board;
mod helpers;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<board::OutlineMaterial>::default())
            .add_event::<ReloadUiEvent>()
            .add_event::<board::AnimationWaitEvent>()
            .add_event::<board::MarbleEvent>()
            .add_event::<board::SlotPressEvent>()
            .add_event::<board::SlotHoverEvent>()
            .init_resource::<board::MoveAnimations>()
            .add_systems(Startup, assets::load_assets)
            .add_systems(
                Update,
                (
                    (board::clear_ui, board::draw_board).run_if(on_event::<ReloadUiEvent>()),
                    board::draw_containers,
                    board::slot_action,
                    board::slot_hover,
                    board::handle_moves,
                )
                    .chain(),
            )
            .add_systems(SpawnScene, (board::draw_labels).chain())
            .add_systems(
                PostUpdate,
                (board::handle_marble_events, board::process_moves).chain(),
            )
            .add_systems(Last, (board::update_labels));
    }
}

#[derive(Event)]
pub struct ReloadUiEvent;

impl Default for ReloadUiEvent {
    fn default() -> Self {
        Self
    }
}

#[derive(Resource)]
pub struct UiAssets {
    pub font: Handle<Font>,
}
