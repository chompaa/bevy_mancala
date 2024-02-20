use super::{
    helpers,
    marble::{MarbleEvent, MarbleEventKind, MarbleOutlineEvent, MarbleStack, MarbleStackEntity},
    Board, CaptureEvent, GameOverEvent, MoveEvent, Slot,
};
use crate::{states::AppState, ui::UiAssets};
use bevy::{ecs::system::SystemState, prelude::*};
use rand::Rng;
use std::{any::Any, collections::VecDeque};

pub const MOVE_SPEED: f32 = 175.;
pub const MOVE_SLOT_OFFSET: f32 = 4.;
pub const MOVE_STORE_OFFSET: f32 = 25.;
pub const MOVE_SCALE: f32 = 0.1;
pub const MOVE_DELAY: f32 = 0.75;

pub const CAPTURE_SPEED: f32 = 225.;
pub const CAPTURE_OFFSET_X: f32 = 4.;
pub const CAPTURE_OFFSET_Y: f32 = 25.;
pub const CAPTURE_DELAY: f32 = 0.25;

pub const FADE_SPEED: f32 = 5.;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AnimationState>()
            .init_resource::<AnimationQueue>()
            .add_systems(
                Update,
                (
                    update_state,
                    handle_capture,
                    handle_game_over.after(handle_capture),
                )
                    .run_if(in_state(AppState::Game)),
            )
            .add_systems(PostUpdate,
                handle_move
            )
            .add_systems(FixedUpdate, animation_tick.run_if(in_state(AppState::Game)));
    }
}

pub trait Animation: Send + Sync {
    fn init(&mut self, _: &mut World) {}
    fn tick(&mut self, world: &mut World);
    fn cleanup(&mut self, world: &mut World);
    fn is_started(&self) -> bool {
        true
    }
    fn is_finished(&self) -> bool;
    fn as_any(&self) -> &dyn Any;
}

pub struct MoveAnimation {
    pub started: bool,
    pub entity: Entity,
    pub stack_container: Entity,
    pub slot: Entity,
    pub previous: Vec2,
    pub moves: VecDeque<(Entity, Vec2)>,
}

impl Animation for MoveAnimation {
    fn init(&mut self, world: &mut World) {
        let children: Vec<Entity> = world
            .get::<Children>(self.stack_container)
            .map_or_else(Vec::new, |entities| entities.iter().copied().collect());

        world.entity_mut(self.entity).push_children(&children);
        world.send_event(MarbleOutlineEvent(self.slot, Visibility::Visible));

        self.started = true;
    }

    fn tick(&mut self, world: &mut World) {
        let mut system_state: SystemState<(
            EventWriter<MarbleEvent>,
            Query<&mut Transform>,
            Query<&MarbleStack>,
            Query<&Children>,
            Res<Time>,
        )> = SystemState::new(world);

        let (mut marble_events, mut transform_query, marble_stack_query, children_query, time) =
            system_state.get_mut(world);

        let mut transform = transform_query.get_mut(self.entity).unwrap();

        let (slot, target, offset) = {
            let (entity, offset) = *self.moves.front().unwrap();
            let stack = marble_stack_query.get(entity).unwrap();

            (stack.0, stack.1 + offset, offset)
        };

        if transform.translation.xy() == target {
            self.moves.pop_front();
            self.previous = target;

            let marble_transform = {
                let children = children_query.get(self.entity).unwrap();
                // TODO: we can't rely on entity order from Bevy 0.13.0, so this needs to be fixed
                transform_query.get(children[0]).unwrap()
            };

            let location = marble_transform.translation.xy() + offset;

            marble_events.send_batch(vec![
                MarbleEvent(MarbleEventKind::Del((self.entity, 1))),
                MarbleEvent(MarbleEventKind::Add((slot, 1, Some(location)))),
            ]);

            return;
        }

        transform
            .translation
            .move_towards(target, MOVE_SPEED * time.delta_seconds());

        let delta = target - transform.translation.xy();
        let total_distance = (target - self.previous).length();
        let travelled = total_distance - delta.length();

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

    fn cleanup(&mut self, world: &mut World) {
        world.send_event(MarbleOutlineEvent(self.slot, Visibility::Hidden));
        world.entity_mut(self.entity).despawn_recursive();
    }

    fn is_started(&self) -> bool {
        self.started
    }

    fn is_finished(&self) -> bool {
        self.moves.is_empty()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct CaptureAnimation {
    started: bool,
    target: Entity,
    moves: Vec<(Entity, Entity)>,
    finished: Vec<Entity>,
}

impl Animation for CaptureAnimation {
    fn init(&mut self, world: &mut World) {
        for (stack_container, container) in &mut self.moves {
            let mut children: Vec<Entity> = world
                .get::<Children>(*stack_container)
                .map_or_else(Vec::new, |entities| entities.iter().copied().collect());

            let translation: Vec2 = world.get::<Transform>(*container).unwrap().translation.xy();

            let mut rng = rand::thread_rng();

            for child in &mut children {
                let offset = (
                    rng.gen_range(-CAPTURE_OFFSET_X..=CAPTURE_OFFSET_X),
                    rng.gen_range(-CAPTURE_OFFSET_Y..=CAPTURE_OFFSET_Y),
                );
                world
                    .entity_mut(*child)
                    .insert(Offset(Vec2::new(-translation.x + offset.0, offset.1)));
            }

            world.entity_mut(*container).push_children(&children);
        }

        self.started = true;
    }

    fn tick(&mut self, world: &mut World) {
        let mut system_state: SystemState<(
            EventWriter<MarbleEvent>,
            Query<&mut Transform>,
            Query<&Offset>,
            Query<&MarbleStack>,
            Query<&Children>,
            Res<Time>,
        )> = SystemState::new(world);

        let (
            mut marble_events,
            mut transform_query,
            offset_query,
            marble_stack_query,
            children_query,
            time,
        ) = system_state.get_mut(world);

        for (_, container) in &mut self.moves {
            let children = children_query.get(*container).unwrap();
            let stack = marble_stack_query.get(self.target).unwrap();
            let mut moving = false;

            for child in children {
                let mut transform = transform_query.get_mut(*child).unwrap();
                let offset = offset_query.get(*child).unwrap();

                let target = stack.1 + offset.0;

                if transform.translation.xy() == target {
                    continue;
                }

                moving = true;

                transform
                    .translation
                    .move_towards(target, CAPTURE_SPEED * time.delta_seconds());
            }

            if !moving {
                let transform = transform_query.get(*container).unwrap();

                for child in children {
                    let offset = offset_query.get(*child).unwrap();
                    let relative = transform.translation.xy() + offset.0;

                    marble_events.send(MarbleEvent(MarbleEventKind::Add((
                        stack.0,
                        1,
                        Some(relative),
                    ))));
                }

                self.finished.push(*container);
            }
        }

        // remove finished moves
        self.moves
            .retain(|(_, entity)| !self.finished.contains(entity));
    }

    fn cleanup(&mut self, world: &mut World) {
        for entity in &self.finished {
            world.entity_mut(*entity).despawn_recursive();
        }
    }

    fn is_started(&self) -> bool {
        self.started
    }

    fn is_finished(&self) -> bool {
        self.moves.is_empty()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Default)]
struct GameOverAnimation {
    elapsed: f32,
    alpha: f32,
}

impl Animation for GameOverAnimation {
    fn tick(&mut self, world: &mut World) {
        let time = world.get_resource::<Time>().unwrap().delta_seconds();

        self.elapsed += time;
        self.alpha = FADE_SPEED * 0.5 * bezier_blend(self.elapsed);

        let mut system_state: SystemState<
            Query<&mut BackgroundColor, (With<Alpha>, Without<Text>)>,
        > = SystemState::new(world);

        let mut background_color_query = system_state.get_mut(world);

        for mut color in &mut background_color_query {
            color.0 = color.0.with_a(self.alpha);
        }
    }

    fn cleanup(&mut self, world: &mut World) {
        let mut system_state: SystemState<Query<&mut Text, With<Alpha>>> = SystemState::new(world);

        let mut text_query = system_state.get_mut(world);

        for mut text in &mut text_query {
            text.sections[0].style.color = text.sections[0].style.color.with_a(1.);
        }
    }

    fn is_finished(&self) -> bool {
        self.alpha >= 0.5
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Component)]
pub struct Stack;

#[derive(Component)]
pub struct Offset(pub Vec2);

#[derive(Component)]
pub struct Alpha;

#[derive(Resource, Default)]
pub struct AnimationQueue(VecDeque<(Box<dyn Animation>, Option<Timer>)>);

#[derive(States, SystemSet, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub enum AnimationState {
    #[default]
    Idle,
    Animating,
}

fn update_state(mut state: ResMut<NextState<AnimationState>>, queue: Res<AnimationQueue>) {
    if !queue.is_changed() {
        return;
    }

    if queue.0.is_empty() {
        state.set(AnimationState::Idle);
    } else {
        state.set(AnimationState::Animating);
    }
}

fn animation_tick(world: &mut World) {
    world.resource_scope(|world, mut queue: Mut<AnimationQueue>| {
        let Some((animation, timer)) = queue.0.front_mut() else {
            return;
        };

        if let Some(timer) = timer {
            let time = world.get_resource::<Time>().unwrap();

            if !timer.tick(time.delta()).finished() {
                return;
            }
        };

        if animation.is_started() {
            animation.tick(world);
        } else {
            animation.init(world);
        }

        if animation.is_finished() {
            animation.cleanup(world);
            queue.0.pop_front();
        }
    });
}

pub fn handle_move(
    mut commands: Commands,
    mut move_events: EventReader<MoveEvent>,
    marble_stack: MarbleStackEntity,
    transform_query: Query<&Transform>,
    slot_query: Query<&Slot>,
    mut animations: ResMut<AnimationQueue>,
) {
    for MoveEvent(_, moves) in move_events.read() {
        let mut slots = moves.clone();
        let start = slots.pop_front().unwrap();

        let mut queue: VecDeque<(Entity, Vec2)> = VecDeque::new();

        let mut rng = rand::thread_rng();

        for slot in slots {
            if let Some((entity, _)) = marble_stack.get(slot) {
                if let Ok(component) = slot_query.get(slot) {
                    let offset = if Board::is_store(component.index) {
                        rng.gen_range(-MOVE_STORE_OFFSET..=MOVE_STORE_OFFSET)
                    } else {
                        match component.index % 2 {
                            0 => MOVE_SLOT_OFFSET,
                            _ => -MOVE_SLOT_OFFSET,
                        }
                    };

                    queue.push_back((entity, Vec2::new(0., offset)));
                }
            }
        }

        let (stack_container, stack) = marble_stack.get(start).unwrap();
        let transform = transform_query.get(stack_container).unwrap();

        let container = commands
            .spawn((
                SpatialBundle {
                    transform: Transform::from_translation(stack.1.extend(100.)),
                    ..default()
                },
                Stack,
            ))
            .id();

        commands
            .entity(container)
            .insert(MarbleStack(container, stack.1, stack.2));

        let animation = MoveAnimation {
            started: false,
            entity: container,
            stack_container,
            slot: start,
            previous: transform.translation.xy(),
            moves: queue,
        };

        let mut timer = None;

        if let Some(current_animation) = animations.0.front() {
            // if there is another move, delay the transition
            if current_animation
                .0
                .as_any()
                .downcast_ref::<MoveAnimation>()
                .is_some()
            {
                timer = Some(Timer::from_seconds(MOVE_DELAY, TimerMode::Once));
            }
        }

        animations.0.push_back((Box::new(animation), timer));
    }
}

fn handle_capture(
    mut commands: Commands,
    mut capture_events: EventReader<CaptureEvent>,
    marble_stack: MarbleStackEntity,
    mut animations: ResMut<AnimationQueue>,
) {
    for event in capture_events.read() {
        let (target, _) = marble_stack.get(event.store).unwrap();

        // TODO: allocate on the stack instead
        let mut moves: Vec<(Entity, Entity)> = vec![];

        for slot in &event.slots {
            if let Some((stack_container, stack)) = marble_stack.get(*slot) {
                let container = commands
                    .spawn((SpatialBundle {
                        transform: Transform::from_translation(stack.1.extend(100.)),
                        ..default()
                    },))
                    .id();

                commands
                    .entity(container)
                    .insert(MarbleStack(container, stack.1, stack.2));

                moves.push((stack_container, container));
            }
        }

        let animation = CaptureAnimation {
            started: false,
            target,
            moves,
            finished: vec![],
        };

        let timer = Timer::from_seconds(CAPTURE_DELAY, TimerMode::Once);

        animations.0.push_back((Box::new(animation), Some(timer)));
    }
}

pub fn handle_game_over(
    mut commands: Commands,
    mut game_over_events: EventReader<GameOverEvent>,
    mut animations: ResMut<AnimationQueue>,
    ui_assets: Res<UiAssets>,
) {
    for event in game_over_events.read() {
        let screen = helpers::get_screen(&mut commands);

        let value = event.0.as_ref().map_or_else(
            || "Draw!".to_string(),
            |player| format!("{} wins!", player.to_string()),
        );

        let container = commands
            .spawn((
                NodeBundle {
                    style: Style {
                        display: Display::Flex,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        width: Val::Percent(100.),
                        flex_grow: 1.,
                        ..default()
                    },
                    background_color: Color::rgba(0.0, 0.0, 0.0, 0.0).into(),
                    ..default()
                },
                Alpha,
            ))
            .id();

        let text = commands
            .spawn((
                TextBundle {
                    text: Text::from_section(
                        value,
                        TextStyle {
                            font: ui_assets.font.clone(),
                            font_size: 40.0,
                            color: Color::WHITE.with_a(0.),
                        },
                    ),
                    ..default()
                },
                Alpha,
            ))
            .id();

        commands.entity(container).add_child(text);
        commands.entity(screen).add_child(container);

        animations
            .0
            .push_back((Box::<GameOverAnimation>::default(), None));
    }
}

fn bezier_blend(time: f32) -> f32 {
    time.powi(2) * 2.0f32.mul_add(-time, 3.)
}

pub trait MoveTowards {
    const THRESHOLD: f32 = 5.;

    fn move_towards(&mut self, target: Vec2, max_velocity: f32);
}

impl MoveTowards for Vec3 {
    fn move_towards(&mut self, target: Vec2, max_velocity: f32) {
        let desired = target - self.xy();
        let distance = desired.length();

        let velocity = if distance < Self::THRESHOLD {
            desired
        } else {
            desired / distance * max_velocity
        };

        self.x += velocity.x;
        self.y += velocity.y;
    }
}
