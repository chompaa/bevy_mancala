use std::collections::{BTreeMap, VecDeque};

use crate::game::{MoveEvent, Slot};

use super::marble::{MarbleEvent, MarbleEventKind, MarbleOutlineEvent, Marbles};

use bevy::prelude::*;

pub const MOVE_SPEED: f32 = 175.;
pub const MOVE_TOLERANCE: f32 = 5.;
pub const MOVE_OFFSET: f32 = 5.;
pub const MOVE_SCALE: f32 = 0.1;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AnimationWaitEvent>()
            .add_event::<AnimationEndEvent>()
            .init_resource::<MoveAnimations>()
            .add_systems(Update, handle_move)
            .add_systems(FixedUpdate, animate_move);
    }
}

#[derive(Component)]
pub struct Animating(pub u32);

#[derive(Component)]
pub struct Stack;

#[derive(Event, Default)]
pub struct AnimationWaitEvent;

#[derive(Event, Default)]
pub struct AnimationEndEvent;

#[derive(Clone)]
pub struct MoveAnimation {
    pub entity: Entity,
    pub original: Entity,
    pub slot: Entity,
    pub previous: Vec2,
    pub queue: VecDeque<Entity>,
}

#[derive(Resource)]
pub struct MoveAnimations {
    pub map: BTreeMap<u32, MoveAnimation>,
    pub delay_timer: Timer,
}

impl Default for MoveAnimations {
    fn default() -> Self {
        Self {
            map: BTreeMap::default(),
            delay_timer: Timer::from_seconds(0.5, TimerMode::Once),
        }
    }
}

pub fn handle_move(
    mut commands: Commands,
    mut move_events: EventReader<MoveEvent>,
    mut wait_events: EventWriter<AnimationWaitEvent>,
    marbles_query: Query<(Entity, &Marbles, &Transform)>,
    mut animations: ResMut<MoveAnimations>,
) {
    for MoveEvent(start_move, start_stack, moves) in move_events.read() {
        let mut slots = moves.clone();
        let start = slots.pop_front().unwrap();

        let mut queue: VecDeque<Entity> = VecDeque::new();

        for slot in slots {
            if let Some((entity, _, _)) = marbles_query.iter().find(|(_, m, _)| m.0 == slot) {
                queue.push_back(entity);
            }
        }

        let (original, marbles, transform) =
            marbles_query.iter().find(|(_, m, _)| m.0 == start).unwrap();

        let container = commands
            .spawn((
                SpatialBundle {
                    transform: Transform::from_translation(marbles.1.extend(100.)),
                    ..default()
                },
                Stack,
            ))
            .id();

        commands
            .entity(container)
            .insert(Marbles(container, marbles.1, marbles.2));

        animations.map.insert(
            *start_move,
            MoveAnimation {
                entity: container,
                original,
                slot: start,
                previous: transform.translation.xy(),
                queue,
            },
        );

        wait_events.send_default();
    }
}

pub fn animate_move(
    mut commands: Commands,
    mut wait_events: EventWriter<AnimationWaitEvent>,
    mut end_events: EventWriter<AnimationEndEvent>,
    mut marble_events: EventWriter<MarbleEvent>,
    mut marble_outline_events: EventWriter<MarbleOutlineEvent>,
    mut transform_query: Query<&mut Transform>,
    slot_query: Query<&Slot>,
    marbles_query: Query<&Marbles>,
    mut children_query: Query<&Children>,
    time: Res<Time>,
    mut animations: ResMut<MoveAnimations>,
) {
    if animations.map.is_empty() {
        return;
    }

    wait_events.send_default();

    if !animations.delay_timer.finished() {
        animations.delay_timer.tick(time.delta());
        return;
    }

    let (move_start, mut animator) = animations.map.clone().into_iter().next().unwrap();
    let mut transform = transform_query.get_mut(animator.entity).unwrap();

    // check if there are no more moves to process
    if animator.queue.is_empty() {
        animations.map.remove(&move_start);

        commands.entity(animator.entity).despawn_recursive();

        // turn off the outline
        marble_outline_events.send(MarbleOutlineEvent(animator.slot, Visibility::Hidden));

        if let Some((_, next)) = animations.map.iter().next() {
            // there is another move to animate, enable its outline
            marble_outline_events.send(MarbleOutlineEvent(next.slot, Visibility::Visible));
        } else {
            // there are no more moves to animate, send the end event
            end_events.send_default();
        }

        animations.delay_timer.reset();

        return;
    }

    if !children_query.get(animator.entity).is_ok() {
        if let Ok(children) = children_query.get_mut(animator.original) {
            for child in children {
                // reassign all marbles to the stack
                commands.entity(*child).set_parent(animator.entity);
            }
        }
    }

    let (slot, target) = {
        let next = animator.queue.get(0).unwrap();
        let marbles = marbles_query.get(*next).unwrap();
        let slot = slot_query.get(marbles.0).unwrap();

        let direction = {
            if slot.index % 2 == 0 {
                1.
            } else {
                -1.
            }
        };

        let mut target = marbles.1;
        target.y += MOVE_OFFSET * direction;

        (marbles.0, target)
    };

    let distance = (target - transform.translation.xy()).length();

    if distance < MOVE_TOLERANCE {
        animator.queue.pop_front();
        animator.previous = target;

        let marble_transform = {
            let children = children_query.get(animator.entity).unwrap();
            // the `Del` event will always take the first child
            transform_query.get(children[0]).unwrap().clone()
        };

        marble_events.send_batch(vec![
            MarbleEvent(MarbleEventKind::Del((animator.entity, 1))),
            MarbleEvent(MarbleEventKind::Add((
                slot,
                1,
                Some(marble_transform.translation.xy()),
            ))),
        ]);

        // be sure to update the animation map
        animations.map.insert(move_start, animator);

        return;
    }

    let difference = target - transform.translation.xy();

    transform.translation += difference.extend(0.) / distance * MOVE_SPEED * time.delta_seconds();

    let total_distance = (target - animator.previous).length();
    let travelled = total_distance - difference.length();

    // elapsed can be negative if the stack overshoots the target, so clamp it to 0.
    // note: since elapsed depends on the transform, we don't need to worry about delta time
    let elapsed = f32::max(travelled / total_distance, 0.);
    let curve = bezier_blend(elapsed);

    let scale = if elapsed > 0.3 {
        // remove from stack scale based on curve
        (1. + MOVE_SCALE) - MOVE_SCALE * curve
    } else {
        // add to stack scale based on curve
        1. + (MOVE_SCALE / 0.3) * curve
    };

    transform.scale = Vec3::splat(scale);
}

fn bezier_blend(time: f32) -> f32 {
    time.powi(2) * (3. - 2. * time)
}
