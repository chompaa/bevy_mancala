use std::collections::{BTreeMap, VecDeque};

use bevy::{prelude::*, sprite::Material2dPlugin};

mod animations;
mod assets;
pub mod board;
mod helpers;
mod labels;
mod marbles;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<marbles::OutlineMaterial>::default())
            .add_event::<ReloadUiEvent>()
            .add_event::<AnimationWaitEvent>()
            .add_event::<MarbleEvent>()
            .add_event::<MarbleOutlineEvent>()
            .add_event::<SlotPressEvent>()
            .add_event::<SlotHoverEvent>()
            .init_resource::<MoveAnimations>()
            .add_systems(Startup, assets::load_assets)
            .add_systems(
                Update,
                (
                    (board::clear_ui, board::draw_board).run_if(on_event::<ReloadUiEvent>()),
                    marbles::draw_containers,
                    marbles::handle_marble_outline,
                    board::handle_action,
                    board::handle_hover,
                    animations::handle_move,
                ),
            )
            .add_systems(SpawnScene, (labels::draw_labels).chain())
            .add_systems(
                PostUpdate,
                (marbles::handle_marble_events, animations::animate_move).chain(),
            )
            .add_systems(Last, labels::update_labels);
    }
}

#[derive(Event)]
pub struct ReloadUiEvent;

impl Default for ReloadUiEvent {
    fn default() -> Self {
        Self
    }
}

#[derive(Event)]
pub struct SlotPressEvent(pub Entity);

#[derive(Event)]
pub struct SlotHoverEvent(Entity, bool);

pub enum MarbleEventKind {
    Add((Entity, u32)),
    Del((Entity, u32)),
}

#[derive(Event)]
pub struct MarbleEvent(pub MarbleEventKind);

#[derive(Event, Default)]
pub struct AnimationWaitEvent;

#[derive(Event)]
pub struct MarbleOutlineEvent(pub Entity, pub Visibility);

#[derive(Component)]
pub struct SlotButton;

#[derive(Component)]
pub struct SlotUi(Entity);

#[derive(Component)]
pub struct Marbles(Entity, Vec2, Vec2);

#[derive(Component)]
pub struct Marble;

#[derive(Component)]
pub struct MarbleOutline(Entity);

#[derive(Component)]
pub struct Label(Entity);

#[derive(Component)]
pub struct Stack;

#[derive(Clone)]
pub struct MoveAnimation {
    pub entity: Entity,
    pub origin: (Entity, u32, Transform),
    pub queue: VecDeque<Entity>,
}

#[derive(Resource)]
pub struct MoveAnimations(pub BTreeMap<u32, MoveAnimation>);

impl Default for MoveAnimations {
    fn default() -> Self {
        Self(BTreeMap::default())
    }
}

#[derive(Component)]
pub struct Animating(pub u32);

#[derive(Resource)]
pub struct UiAssets {
    pub font: Handle<Font>,
    pub marble: Handle<Image>,
}
