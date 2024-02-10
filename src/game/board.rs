use super::{
    animation::AnimationState,
    helpers,
    marble::{MarbleOutlineEvent, Marbles},
};
use crate::game::{Board, CurrentPlayer, Slot};
use crate::ui::ReloadUiEvent;
use bevy::prelude::*;

pub const SLOT_SIZE: f32 = 64.0;
pub const SLOT_GAP: f32 = 12.0;

pub const STORE_WIDTH: f32 = 64.0 + 8.0;
pub const STORE_HEIGHT: f32 = 128.0 + 28.0;
pub const STORE_GAP: f32 = 0.0;

pub const BOARD_WIDTH: f32 = SLOT_SIZE * (Board::COLS as f32)
    + SLOT_GAP * ((Board::COLS - 1) as f32)
    + 2. * STORE_WIDTH
    + 2. * STORE_GAP;
pub const BOARD_HEIGHT: f32 =
    SLOT_SIZE * (Board::ROWS as f32) + SLOT_GAP * ((Board::ROWS - 1) as f32);

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SlotPressEvent>()
            .add_event::<SlotHoverEvent>()
            .add_systems(
                Update,
                (
                    (clear_ui, draw_board).run_if(on_event::<ReloadUiEvent>()),
                    handle_action.run_if(in_state(AnimationState::Idle)),
                    handle_hover,
                ),
            );
    }
}

#[derive(Event)]
pub struct SlotPressEvent(pub Entity);

#[derive(Event)]
pub struct SlotHoverEvent(Entity, bool);

#[derive(Component)]
pub struct SlotButton;

#[derive(Component)]
pub struct SlotUi(pub Entity);

pub fn draw_board(mut commands: Commands, board: Res<Board>) {
    let screen = helpers::get_screen(&mut commands);

    let slot_container = commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(SLOT_GAP),
                row_gap: Val::Px(SLOT_GAP),
                ..default()
            },
            ..default()
        })
        .id();

    let mut stores: Vec<Entity> = vec![];

    for slot in Board::slot_order() {
        let slot_entity = board.slots[slot];

        if Board::is_store(slot) {
            // slot is a store, so we need to create a store node

            let node = helpers::get_node(&mut commands, STORE_WIDTH, STORE_HEIGHT);
            commands.entity(node).insert(SlotUi(slot_entity));
            stores.push(node);

            continue;
        }

        let button = helpers::get_button(&mut commands, SLOT_SIZE, SLOT_SIZE);

        commands
            .entity(button)
            .insert((SlotButton, SlotUi(slot_entity)));
        commands.entity(slot_container).add_child(button);
    }

    assert_eq!(stores.len(), 2);

    let board_container = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(BOARD_WIDTH),
                height: Val::Px(BOARD_HEIGHT),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(STORE_GAP),
                ..default()
            },
            ..default()
        })
        .id();

    commands
        .entity(board_container)
        .push_children(&[stores[0], slot_container, stores[1]]);

    commands.entity(screen).add_child(board_container);
}

pub fn handle_hover(
    mut slot_hover_events: EventReader<SlotHoverEvent>,
    mut marble_outline_events: EventWriter<MarbleOutlineEvent>,
    slot_query: Query<&Slot>,
    current_player: Res<CurrentPlayer>,
) {
    for event in slot_hover_events.read() {
        let slot = slot_query.get(event.0).unwrap();

        if Board::owner(slot.index) != current_player.0 {
            continue;
        }

        let visibility = if event.1 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        marble_outline_events.send(MarbleOutlineEvent(event.0, visibility));
    }
}

pub fn handle_action(
    mut changed_interaction_query: Query<
        (&Interaction, &SlotUi),
        (Changed<Interaction>, With<Button>, With<SlotButton>),
    >,
    mut interaction_query: Query<(&Interaction, &SlotUi), (With<Button>, With<SlotButton>)>,
    mut slot_press_events: EventWriter<SlotPressEvent>,
    mut slot_hover_events: EventWriter<SlotHoverEvent>,
) {
    // leave events
    for (interaction, slot_ui) in &mut changed_interaction_query {
        match *interaction {
            Interaction::Pressed => {
                slot_press_events.send(SlotPressEvent(slot_ui.0));
            }
            Interaction::Hovered => {}
            Interaction::None => {
                slot_hover_events.send(SlotHoverEvent(slot_ui.0, false));
            }
        }
    }

    // enter events
    for (interaction, slot_ui) in &mut interaction_query {
        match *interaction {
            Interaction::Hovered => {
                slot_hover_events.send(SlotHoverEvent(slot_ui.0, true));
            }
            _ => {}
        }
    }
}

pub fn clear_ui(
    mut commands: Commands,
    mut reload_ui_events: EventReader<ReloadUiEvent>,
    slot_ui_query: Query<Entity, With<SlotUi>>,
    marbles_query: Query<Entity, With<Marbles>>,
) {
    for _ in reload_ui_events.read() {
        for entity in slot_ui_query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        for entity in marbles_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
