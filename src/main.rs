
use std::f32::consts::PI;
use rand::random;
use itertools::Itertools;
use bevy::{
    prelude::*,
    sprite::{Mesh2dHandle, MaterialMesh2dBundle},
    render::mesh::PrimitiveTopology::TriangleList,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
};

const BOIDS: usize = 100;
const BORDER: f32 = 256.;
const SPEED: f32 = 4.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "boids".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, popup)
        .add_systems(Update, boids)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let tail = [17., 19.]
        .map(|x| {
                let theta = PI / 12. * x;
                16. * (theta.cos() * Vec2::X + theta.sin() * Vec2::Y)
            });
    let vertices = (1..BOIDS)
        .flat_map(|_| {
                let position = BORDER * (Vec2::new(random(), random()) - 0.5);
                let rotation = Vec2::from_angle(2. * PI * random::<f32>());

                [rotation.rotate(tail[0]), rotation.rotate(tail[1]), Vec2::ZERO]
                    .map(|point| (point + position).extend(0.).to_array())
            })
        .collect::<Vec<[f32; 3]>>();

    let mut mesh = Mesh::new(TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);

    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(mesh).into(),
        material: materials.add(ColorMaterial::from(Color::BLUE)),
        visibility: Visibility::Hidden,
        ..default()
    });

    commands.spawn(Text2dBundle {
        text: Text::from_sections([
            TextSection::new(
                "Boids\n",
                TextStyle {
                    font_size: 64.,
                    ..default()
                }
            ),
            TextSection::new(
                "Press `escape` to start the simulation\n\n",
                TextStyle {
                    font_size: 48.,
                    ..default()
                }
            ),
            TextSection::new(
                "Separation: boids should avoid crowding nearby boids.\nAlignment: boids should fly in the same direction as nearby boids.\nCohesion: boids should be near other boids.",
                TextStyle {
                    font_size: 32.,
                    ..default()
                }
            )
        ]),
        visibility: Visibility::Visible,
        ..default()
    });

    commands.spawn(Camera2dBundle::default());
}

fn popup(mut visibilities: Query<&mut Visibility>, keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::Escape) {
        visibilities
            .for_each_mut(|mut visibility| {
                *visibility = match *visibility {
                Visibility::Visible => Visibility::Hidden,
                Visibility::Hidden => Visibility::Visible,
                Visibility::Inherited => Visibility::Inherited
            };
        })
    }
}

fn boids(query: Query<&Mesh2dHandle>, meshes: ResMut<Assets<Mesh>>, time: Res<Time>) {
    let delta = SPEED * time.delta_seconds();

    let mesh = meshes
        .into_inner()
        .get_mut(&query.single().0)
        .unwrap();

    let vertices = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .unwrap() 
        .as_float3()
        .unwrap()
        .into_iter()
        .chunks(3)
        .into_iter()
        .flat_map(|chunk| {
                let boid = chunk
                    .map(|[x, y, _]| [*x, *y])
                    .collect::<Vec<[f32; 2]>>();

                let wrapped_head = boid[2].map(|n| (n + BORDER).rem_euclid(2. * BORDER) - BORDER);
                let [delta_x, delta_y] = [boid[2][0] - wrapped_head[0], boid[2][1] - wrapped_head[1]];
                let wrapped_boid = [
                    [boid[0][0] - delta_x, boid[0][1] - delta_y],
                    [boid[1][0] - delta_x, boid[1][1] - delta_y],
                    wrapped_head
                ];

                wrapped_boid
                    .iter()
                    .map(|[x, y]| [
                            x + (delta * (wrapped_boid[0][1] - wrapped_boid[1][1])),
                            y + (-delta * (wrapped_boid[0][0] - wrapped_boid[1][0]))
                        ])
                    .collect::<Vec<[f32; 2]>>()
            })
        .map(|[x, y]| [x, y, 0.])
        .collect::<Vec<[f32; 3]>>();

    mesh.remove_attribute(Mesh::ATTRIBUTE_POSITION);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
}

// fn update(mut transforms: Query<&mut Transform, With<Boid>>, time: Res<Time>) {
//     let delta = 100. * time.delta_seconds();
//     let x = 2. * BORDER * Vec3::ONE;

//     for mut transform in &mut transforms {
//         println!("{}\n", transform.translation);
//         transform.translation = (
//             transform.translation + transform.local_y() * delta + BORDER
//         ).rem_euclid(x) - BORDER;
//     }
// }
