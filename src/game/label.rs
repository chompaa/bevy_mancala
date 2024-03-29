use super::{
    helpers,
    marble::{MarbleStack, MarbleStackEntity},
    Board, Slot,
};
use crate::{states::AppState, ui::UiAssets};
use bevy::prelude::*;

pub const LABEL_SIZE: f32 = 64.0;
pub const LABEL_SLOT_GAP_X: f32 = 12.;
pub const LABEL_SLOT_GAP_Y: f32 = 208.0;
pub const LABEL_STORE_GAP_X: f32 = 102.0;

pub struct LabelPlugin;

impl Plugin for LabelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SpawnScene, draw_labels.run_if(in_state(AppState::Game)))
            .add_systems(Last, update_labels.run_if(in_state(AppState::Game)))
            .add_systems(OnExit(AppState::Game), helpers::despawn::<LabelScreen>);
    }
}

#[derive(Component)]
struct LabelScreen;

#[derive(Component)]
pub struct Label(Entity);

pub fn draw_labels(
    mut commands: Commands,
    label_query: Query<Entity, With<Label>>,
    marble_stack_query: Query<&MarbleStack>,
    container_query: Query<Entity, Added<MarbleStack>>,
    slot_query: Query<&Slot>,
    assets: Res<UiAssets>,
) {
    if container_query.iter().count() != Board::LENGTH {
        return;
    }

    clear_labels(&mut commands, &label_query);

    let screen = helpers::get_screen(&mut commands);

    commands.entity(screen).insert(LabelScreen);

    let width = LABEL_SIZE * (Board::COLS as f32) + LABEL_SLOT_GAP_X * ((Board::COLS - 1) as f32);
    let height = LABEL_SIZE * (Board::ROWS as f32) + LABEL_SLOT_GAP_Y;

    let labels_container = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(width),
                height: Val::Px(height),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(LABEL_SLOT_GAP_X),
                row_gap: Val::Px(LABEL_SLOT_GAP_Y),
                ..default()
            },
            ..default()
        })
        .id();

    // TODO: allocate on the stack
    let mut store_labels: Vec<(Entity, usize)> = vec![];
    let mut slot_labels: Vec<(Entity, usize)> = vec![];

    for stack in &marble_stack_query {
        let label = helpers::get_label(
            &mut commands,
            Label(stack.0),
            assets.as_ref(),
            LABEL_SIZE,
            "0",
        );

        let index = slot_query.get(stack.0).unwrap().index;

        if Board::is_store(index) {
            store_labels.push((label, index));
        } else {
            slot_labels.push((label, index));
        }
    }

    store_labels.sort_by(|a, b| a.1.cmp(&b.1));

    slot_labels.sort_by(|a, b| {
        let ord_a = Board::slot_order().iter().position(|&x| x == a.1).unwrap();
        let ord_b = Board::slot_order().iter().position(|&x| x == b.1).unwrap();

        ord_a.cmp(&ord_b)
    });

    for (label, _) in slot_labels {
        commands.entity(labels_container).add_child(label);
    }

    let container = commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(LABEL_STORE_GAP_X),
                ..default()
            },
            ..default()
        })
        .id();

    assert_eq!(store_labels.len(), 2);

    commands.entity(container).push_children(&[
        store_labels[0].0,
        labels_container,
        store_labels[1].0,
    ]);
    commands.entity(screen).add_child(container);
}

pub fn update_labels(
    mut label_query: Query<(&mut Text, &Label)>,
    changed_query: Query<Entity, (Changed<Children>, With<MarbleStack>)>,
    marble_stack: MarbleStackEntity,
    children_query: Query<Option<&Children>>,
) {
    if changed_query.iter().count() == 0 {
        return;
    }

    for (mut text, label) in &mut label_query {
        if let Some((stack_container, _)) = marble_stack.get(label.0) {
            let count = children_query
                .get(stack_container)
                .unwrap_or(None)
                .map_or(0, |children| children.len());

            text.sections[0].value = count.to_string();
        }
    }
}

pub fn clear_labels(commands: &mut Commands, label_query: &Query<Entity, With<Label>>) {
    for entity in label_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
