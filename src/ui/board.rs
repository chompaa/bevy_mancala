use std::cmp::max;
use std::collections::BTreeMap;
use std::collections::VecDeque;

use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, MaterialMesh2dBundle, Mesh2dHandle},
};

use super::UiAssets;
use super::{helpers, ReloadUiEvent};
use crate::game::{Board, MoveEvent, Slot};

const ROWS: usize = 2;
const COLS: usize = 6;

const MARBLE_SIZE: f32 = 48.0;

const SLOT_SIZE: f32 = 64.0;
const SLOT_GAP: f32 = 12.0;

const STORE_WIDTH: f32 = 64.0 + 8.0;
const STORE_HEIGHT: f32 = 128.0 + 28.0;
const STORE_GAP: f32 = 0.0;

const LABEL_SIZE: f32 = 64.0;
const LABEL_SLOT_GAP_X: f32 = 12.;
const LABEL_SLOT_GAP_Y: f32 = 208.0;
const LABEL_STORE_GAP_X: f32 = 103.0;

const MOVE_SPEED: f32 = 5.;
const MOVE_TOLERANCE: f32 = 1.;

#[derive(AsBindGroup, TypePath, Asset, Debug, Clone)]
pub struct OutlineMaterial {
    #[uniform(0)]
    color: Color,
    #[uniform(0)]
    thickness: f32,
    #[texture(1)]
    #[sampler(2)]
    texture: Handle<Image>,
}

impl Material2d for OutlineMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/outline.wgsl".into()
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

#[derive(Component)]
pub struct SlotButton;

#[derive(Component)]
pub struct SlotUi(Entity);

#[derive(Component)]
pub struct Marbles(Entity, Vec2, Vec2);

#[derive(Component)]
pub struct Marble;

#[derive(Component)]
pub struct Outline(Entity);

#[derive(Component)]
pub struct Label(Entity);

#[derive(Component)]
pub struct Stack;

#[derive(Component)]
pub struct MoveAnimation {
    pub origin: (Entity, u32, Transform),
    pub queue: VecDeque<Entity>,
    pub animating: bool,
}

#[derive(Resource)]
pub struct MoveAnimations(pub BTreeMap<u32, Entity>);

impl Default for MoveAnimations {
    fn default() -> Self {
        Self(BTreeMap::default())
    }
}

pub fn draw_board(mut commands: Commands, board: Res<Board>) {
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

    let board_width = SLOT_SIZE * (COLS as f32)
        + SLOT_GAP * ((COLS - 1) as f32)
        + 2. * STORE_WIDTH
        + 2. * STORE_GAP;
    let board_height = SLOT_SIZE * (ROWS as f32) + SLOT_GAP * ((ROWS - 1) as f32);

    let board_container = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(board_width),
                height: Val::Px(board_height),
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

    let width = LABEL_SIZE * (COLS as f32) + LABEL_SLOT_GAP_X * ((COLS - 1) as f32);
    let height = LABEL_SIZE * (ROWS as f32) + LABEL_SLOT_GAP_Y;

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
        (&Children, &Marbles, Option<&MoveAnimation>),
        (Changed<Children>, With<Marbles>),
    >,
) {
    for (children, marbles, move_animation) in &changed_query {
        let count = {
            let stack = if let Some(move_animation) = move_animation {
                if move_animation.animating {
                    move_animation.origin.1 as i32
                } else {
                    0
                }
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

pub fn handle_marble_events(
    mut commands: Commands,
    mut marble_events: EventReader<MarbleEvent>,
    mut children_query: Query<&Children>,
    marbles_query: Query<(Entity, &Marbles)>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<OutlineMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for MarbleEvent(event) in marble_events.read() {
        match event {
            MarbleEventKind::Add((entity, count)) => {
                let image: Handle<Image> = asset_server.load("textures/marble.png");

                let (container, marbles) = marbles_query
                    .iter()
                    .find(|(_, marbles)| marbles.0 == *entity)
                    .unwrap();

                for _ in 0..*count {
                    let offset = helpers::random_point_in_circle(marbles.2).extend(0.);

                    let wrapper = commands
                        .spawn((
                            SpatialBundle::from_transform(Transform::from_translation(offset)),
                            Marble,
                        ))
                        .id();

                    let sprite = commands
                        .spawn(SpriteBundle {
                            sprite: Sprite {
                                custom_size: Some((MARBLE_SIZE, MARBLE_SIZE).into()),
                                ..default()
                            },
                            texture: image.clone(),
                            transform: Transform::from_translation(Vec2::ZERO.extend(1.)),
                            ..default()
                        })
                        .id();

                    let mesh = Mesh2dHandle(meshes.add(Mesh::from(shape::Quad {
                        size: Vec2::new(MARBLE_SIZE, MARBLE_SIZE),
                        flip: false,
                    })));

                    let shader = commands
                        .spawn((
                            MaterialMesh2dBundle {
                                mesh,
                                visibility: Visibility::Hidden,
                                material: materials.add(OutlineMaterial {
                                    color: Color::WHITE,
                                    thickness: 0.04,
                                    texture: image.clone(),
                                }),
                                ..default()
                            },
                            Outline(*entity),
                        ))
                        .id();

                    commands.entity(wrapper).push_children(&[sprite, shader]);
                    commands.entity(container).add_child(wrapper);
                }
            }
            MarbleEventKind::Del((entity, count)) => {
                let (container, _) = marbles_query
                    .iter()
                    .find(|(_, marbles)| marbles.0 == *entity)
                    .unwrap();

                if let Ok(children) = children_query.get_mut(container) {
                    // let take = children.len() - (*count as usize);

                    for child in children.iter().take(*count as usize) {
                        commands.entity(*child).despawn_recursive();
                    }
                } else {
                    println!("No children found for {:?}", container);
                }
            }
        }
    }
}

pub fn spawn_marble_containers(
    mut commands: Commands,
    global_transform_query: Query<(&Style, &GlobalTransform, &SlotUi), Added<SlotUi>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    slot_query: Query<&Slot>,
    mut marble_events: EventWriter<MarbleEvent>,
) {
    for (style, global_transform, slot_ui) in global_transform_query.iter() {
        let (camera, camera_transform) = camera_query.get_single().unwrap();

        let transform = camera
            .viewport_to_world_2d(camera_transform, global_transform.translation().xy())
            .unwrap();

        let radius = {
            // width and height are guaranteed to be Val::Px here
            let width = style.width.resolve(0., Vec2::ZERO).unwrap();
            let height = style.height.resolve(0., Vec2::ZERO).unwrap();

            Vec2::new(
                width / 2. - MARBLE_SIZE / 3.,
                height / 2. - MARBLE_SIZE / 3.,
            )
        };

        commands.spawn((
            SpatialBundle {
                transform: Transform::from_translation(transform.extend(1.)),
                ..default()
            },
            Marbles(slot_ui.0, transform, radius),
        ));

        let count = slot_query.get(slot_ui.0).unwrap().count;

        marble_events.send(MarbleEvent(MarbleEventKind::Add((slot_ui.0, count))));
    }
}

pub fn slot_hover(
    mut slot_hover_events: EventReader<SlotHoverEvent>,
    mut outline_query: Query<(&Outline, &mut Visibility)>,
) {
    for event in slot_hover_events.read() {
        for (outline, mut visibility) in outline_query.iter_mut() {
            if outline.0 != event.0 {
                continue;
            }

            *visibility = if event.1 {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}

pub fn slot_action(
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
            _ => {
                slot_hover_events.send(SlotHoverEvent(slot_ui.0, false));
            }
        }
    }

    // enter events
    for (interaction, slot_ui) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {}
            Interaction::Hovered => {
                slot_hover_events.send(SlotHoverEvent(slot_ui.0, true));
            }
            _ => {}
        }
    }
}

pub fn handle_moves(
    mut commands: Commands,
    mut move_events: EventReader<MoveEvent>,
    marbles_query: Query<(Entity, &Marbles, &Transform)>,
    mut animations: ResMut<MoveAnimations>,
) {
    for event in move_events.read() {
        let MoveEvent(start_move, start_stack, moves) = event;
        let mut slots = moves.clone();
        let start = slots.pop_front().unwrap();

        let mut queue: VecDeque<Entity> = VecDeque::new();

        for slot in slots {
            if let Some((entity, _, _)) = marbles_query.iter().find(|(_, m, _)| m.0 == slot) {
                queue.push_back(entity);
            }
        }

        let (marbles, _, transform) = marbles_query.iter().find(|(_, m, _)| m.0 == start).unwrap();

        commands.entity(marbles).insert(MoveAnimation {
            origin: (start, *start_stack, *transform),
            queue,
            animating: false,
        });
        animations.0.insert(*start_move, marbles);
    }
}

pub fn process_moves(
    mut commands: Commands,
    mut wait_events: EventWriter<AnimationWaitEvent>,
    mut marble_events: EventWriter<MarbleEvent>,
    mut animator_query: Query<(Entity, &mut MoveAnimation, &mut Transform)>,
    marbles_query: Query<&Marbles>,
    time: Res<Time>,
    mut animations: ResMut<MoveAnimations>,
) {
    if animations.0.is_empty() {
        return;
    }

    wait_events.send_default();

    let (move_start, entity) = animations.0.clone().into_iter().next().unwrap();
    let (entity, mut animator, mut transform) = animator_query.get_mut(entity).unwrap();

    // check if there are no more moves to process
    if animator.queue.is_empty() {
        animations.0.remove(&move_start);
        commands.entity(entity).remove::<MoveAnimation>();

        // reset the transform to the origin
        transform.translation = animator.origin.2.translation;

        return;
    }

    animator.animating = true;

    let (slot, target) = {
        let next = animator.queue.get(0).unwrap();
        let marble = marbles_query.get(*next).unwrap();

        (marble.0, marble.1)
    };

    let distance = (target - transform.translation.xy()).length();

    if distance < MOVE_TOLERANCE {
        transform.translation = target.extend(1.);
        animator.queue.pop_front();

        marble_events.send_batch(vec![
            MarbleEvent(MarbleEventKind::Del((animator.origin.0, 1))),
            MarbleEvent(MarbleEventKind::Add((slot, 1))),
        ]);
    } else {
        transform.translation = transform
            .translation
            .lerp(target.extend(100.), MOVE_SPEED * time.delta_seconds());
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

pub fn clear_labels(commands: &mut Commands, label_query: &Query<Entity, With<Label>>) {
    for entity in label_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
