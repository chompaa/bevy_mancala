use super::{board::SlotUi, helpers};
use crate::{game::Slot, states::AppState, ui::UiAssets};
use bevy::{
    ecs::system::SystemParam,
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle},
};

pub const MARBLE_SIZE: f32 = 48.0;

pub struct MarblePlugin;

impl Plugin for MarblePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<OutlineMaterial>::default())
            .add_event::<MarbleEvent>()
            .add_event::<MarbleOutlineEvent>()
            .add_systems(
                Update,
                (draw_containers, handle_marble_outline).run_if(in_state(AppState::Game)),
            )
            .add_systems(
                PostUpdate,
                handle_marble_events.run_if(in_state(AppState::Game)),
            )
            .add_systems(OnExit(AppState::Game), helpers::despawn::<MarbleContainer>);
    }
}

pub enum MarbleEventKind {
    Add((Entity, u32, Option<Vec2>)),
    Del((Entity, u32)),
}

#[derive(SystemParam)]
pub struct MarbleStackEntity<'w, 's> {
    marbles_query: Query<'w, 's, (Entity, &'static MarbleStack)>,
}

impl<'w, 's> MarbleStackEntity<'w, 's> {
    pub fn get(&self, slot_entity: Entity) -> Option<(Entity, &MarbleStack)> {
        self.marbles_query
            .iter()
            .find(|(_, marbles)| marbles.0 == slot_entity)
    }
}

#[derive(Event)]
pub struct MarbleEvent(pub MarbleEventKind);

#[derive(Event)]
pub struct MarbleOutlineEvent(pub Entity, pub Visibility);

#[derive(Component)]
pub struct Marble;

#[derive(Component)]
pub struct MarbleStack(pub Entity, pub Vec2, pub Vec2);

#[derive(Component)]
struct MarbleContainer;

#[derive(Component)]
pub struct MarbleOutline(Entity);

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

pub fn handle_marble_events(
    mut commands: Commands,
    mut marble_events: EventReader<MarbleEvent>,
    marble_stack: MarbleStackEntity,
    mut children_query: Query<&Children>,
    mut materials: ResMut<Assets<OutlineMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    ui_assets: Res<UiAssets>,
) {
    for MarbleEvent(event) in marble_events.read() {
        match event {
            MarbleEventKind::Add((slot, count, offset)) => {
                let (stack_container, stack) = marble_stack.get(*slot).unwrap();

                for _ in 0..*count {
                    let offset = offset
                        .as_ref()
                        .map_or_else(
                            || helpers::random_point_in_circle(stack.2),
                            |offset| offset.clamp_length_max(stack.2.y),
                        )
                        .extend(0.);

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
                            texture: ui_assets.marble.clone(),
                            transform: Transform::from_translation(Vec2::ZERO.extend(1.)),
                            ..default()
                        })
                        .id();

                    let mesh = Mesh2dHandle(meshes.add(Mesh::from(Rectangle {
                        half_size: Vec2::new(MARBLE_SIZE / 2., MARBLE_SIZE / 2.),
                    })));

                    let shader = commands
                        .spawn((
                            MaterialMesh2dBundle {
                                mesh,
                                visibility: Visibility::Hidden,
                                material: materials.add(OutlineMaterial {
                                    color: Color::WHITE,
                                    thickness: 0.04,
                                    texture: ui_assets.marble.clone(),
                                }),
                                ..default()
                            },
                            MarbleOutline(*slot),
                        ))
                        .id();

                    commands.entity(wrapper).push_children(&[sprite, shader]);
                    commands.entity(stack_container).add_child(wrapper);
                }
            }
            MarbleEventKind::Del((entity, count)) => {
                let Some((stack_container, _)) = marble_stack.get(*entity) else {
                    continue;
                };

                if let Ok(children) = children_query.get_mut(stack_container) {
                    for child in children.iter().take(*count as usize) {
                        commands.entity(*child).despawn_recursive();
                    }
                }
            }
        }
    }
}

pub fn handle_marble_outline(
    mut outline_query: Query<(&MarbleOutline, &mut Visibility)>,
    mut marble_outline_events: EventReader<MarbleOutlineEvent>,
) {
    for MarbleOutlineEvent(slot, visibility) in marble_outline_events.read() {
        for (outline, mut outline_visibility) in &mut outline_query {
            if *slot == outline.0 {
                *outline_visibility = *visibility;
            }
        }
    }
}

pub fn draw_containers(
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
            MarbleStack(slot_ui.0, transform, radius),
            MarbleContainer,
        ));

        let count = slot_query.get(slot_ui.0).unwrap().count;

        marble_events.send(MarbleEvent(MarbleEventKind::Add((slot_ui.0, count, None))));
    }
}
