use std::cmp::max;

use bevy::prelude::*;

use super::{helpers, Animating, Label, Marbles, UiAssets};
use crate::game::{Board, Slot};

const LABEL_SIZE: f32 = 64.0;
const LABEL_SLOT_GAP_X: f32 = 12.;
const LABEL_SLOT_GAP_Y: f32 = 208.0;
const LABEL_STORE_GAP_X: f32 = 103.0;

pub fn draw_labels(
    mut commands: Commands,
    label_query: Query<Entity, With<Label>>,
    marbles_query: Query<&Marbles>,
    container_query: Query<Entity, Added<Marbles>>,
    slot_query: Query<&Slot>,
    assets: Res<UiAssets>,
) {
    if container_query.iter().count() != Board::LENGTH {
        return;
    }

    clear_labels(&mut commands, &label_query);

    let screen = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .id();

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

    let mut store_labels: Vec<(Entity, usize)> = vec![];
    let mut slot_labels: Vec<(Entity, usize)> = vec![];

    for marbles in &marbles_query {
        let label = helpers::get_label(
            &mut commands,
            Label(marbles.0),
            assets.as_ref(),
            LABEL_SIZE,
            "0",
        );

        let index = slot_query.get(marbles.0).unwrap().index;

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
    changed_query: Query<
        (&Children, &Marbles, Option<&Animating>),
        (Changed<Children>, With<Marbles>),
    >,
) {
    for (children, marbles, animating) in &changed_query {
        let count = {
            let stack = if let Some(animating) = animating {
                animating.0 as i32
            } else {
                0
            };

            max((children.len() as i32) - stack, 0)
        };

        for (mut text, label) in &mut label_query {
            if label.0 == marbles.0 {
                text.sections[0].value = count.to_string();
            }
        }
    }
}

pub fn clear_labels(commands: &mut Commands, label_query: &Query<Entity, With<Label>>) {
    for entity in label_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
