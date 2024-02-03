use crate::game::Slot;

use super::{helpers, MarbleEvent, MarbleEventKind, Marbles, Outline, SlotUi, UiAssets};

use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, MaterialMesh2dBundle, Mesh2dHandle},
};

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

const MARBLE_SIZE: f32 = 48.0;

pub fn handle_marble_events(
    mut commands: Commands,
    mut marble_events: EventReader<MarbleEvent>,
    mut children_query: Query<&Children>,
    marbles_query: Query<(Entity, &Marbles)>,
    mut materials: ResMut<Assets<OutlineMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    ui_assets: Res<UiAssets>,
) {
    for MarbleEvent(event) in marble_events.read() {
        match event {
            MarbleEventKind::Add((entity, count)) => {
                let (container, marbles) = marbles_query
                    .iter()
                    .find(|(_, marbles)| marbles.0 == *entity)
                    .unwrap();

                for _ in 0..*count {
                    let offset = helpers::random_point_in_circle(marbles.2).extend(0.);

                    let wrapper = commands
                        .spawn(SpatialBundle::from_transform(Transform::from_translation(
                            offset,
                        )))
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
                                    texture: ui_assets.marble.clone(),
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
            Marbles(slot_ui.0, transform, radius),
        ));

        let count = slot_query.get(slot_ui.0).unwrap().count;

        marble_events.send(MarbleEvent(MarbleEventKind::Add((slot_ui.0, count))));
    }
}
