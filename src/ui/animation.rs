use std::collections::{BTreeMap, VecDeque};

use crate::game::{MoveEvent, Slot};

use super::marble::{MarbleEvent, MarbleEventKind, MarbleOutlineEvent, Marbles};

use bevy::prelude::*;

pub const MOVE_SPEED: f32 = 150.;
pub const MOVE_TOLERANCE: f32 = 2.;
pub const MOVE_OFFSET: f32 = 10.;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AnimationWaitEvent>()
            .add_event::<AnimationEndEvent>()
            .init_resource::<MoveAnimations>()
            .add_systems(Update, handle_move)
            .add_systems(PostUpdate, animate_move);
    }
}

#[derive(Component)]
pub struct Animating(pub u32);

#[derive(Event, Default)]
pub struct AnimationWaitEvent;

#[derive(Event, Default)]
pub struct AnimationEndEvent;

#[derive(Clone)]
pub struct MoveAnimation {
    pub entity: Entity,
    pub origin: (Entity, u32, Transform),
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

        let (marbles, _, transform) = marbles_query.iter().find(|(_, m, _)| m.0 == start).unwrap();

        animations.map.insert(
            *start_move,
            MoveAnimation {
                entity: marbles,
                origin: (start, *start_stack, *transform),
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
    mut entity_query: Query<(Entity, &mut Transform)>,
    slot_query: Query<&Slot>,
    marbles_query: Query<&Marbles>,
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
    // let (entity, mut animator, mut transform) = animator_query.get_mut(entity).unwrap();
    let (entity, mut transform) = entity_query.get_mut(animator.entity).unwrap();

    // check if there are no more moves to process
    if animator.queue.is_empty() {
        animations.map.remove(&move_start);

        commands.entity(entity).remove::<Animating>();

        // reset the transform to the origin
        transform.translation = animator.origin.2.translation;

        // turn off the outline
        marble_outline_events.send(MarbleOutlineEvent(animator.origin.0, Visibility::Hidden));

        if let Some((_, next)) = animations.map.clone().into_iter().next() {
            // if there is another move to animate, enable its outline
            marble_outline_events.send(MarbleOutlineEvent(next.origin.0, Visibility::Visible));
        } else {
            // if there are no more moves to animate, send the end event
            end_events.send_default();
        }

        animations.delay_timer.reset();

        return;
    }

    if animations.delay_timer.just_finished() {
        commands.entity(entity).insert(Animating(animator.origin.1));
        transform.translation.z = 100.;
    }

    let (slot, target) = {
        let next = animator.queue.get(0).unwrap();
        let marble = marbles_query.get(*next).unwrap();
        let slot = slot_query.get(marble.0).unwrap();

        let direction = {
            if slot.index % 2 == 0 {
                1.
            } else {
                -1.
            }
        };

        let mut target = marble.1;
        target.y += MOVE_OFFSET * direction;

        (marble.0, target)
    };

    let distance = (target - transform.translation.xy()).length();

    if distance < MOVE_TOLERANCE {
        animator.queue.pop_front();

        marble_events.send_batch(vec![
            MarbleEvent(MarbleEventKind::Del((animator.origin.0, 1))),
            MarbleEvent(MarbleEventKind::Add((slot, 1))),
        ]);
    } else {
        let difference = target - transform.translation.xy();

        transform.translation +=
            difference.extend(0.) / distance * MOVE_SPEED * time.delta_seconds();
    }

    // be sure to update the animation map
    animations.map.insert(move_start, animator);
}