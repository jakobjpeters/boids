
use std::{iter::zip, f32::consts::PI, cmp::Ordering};
use rand::random;
use itertools::Itertools;
use bevy::{
    prelude::*,
    sprite::{Mesh2dHandle, MaterialMesh2dBundle},
    render::mesh::PrimitiveTopology::TriangleList,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin}
};

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
        .add_systems(Update, update)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    let points = [17., 19.]
        .map(|x| {
                let theta = PI / 12. * x;
                16. * (theta.cos() * Vec2::X + theta.sin() * Vec2::Y)
            });
    let vertices = (1..100)
        .flat_map(|_| {
                let position = 512. * (Vec2::new(random(), random()) - 0.5);
                let rotation = Vec2::from_angle(2. * PI * random::<f32>());
                [rotation.rotate(points[0]), rotation.rotate(points[1]), Vec2::ZERO]
                    .map(|point| (point + position).extend(0.).to_array())
            })
        .collect::<Vec<[f32; 3]>>();

    let mut mesh = Mesh::new(TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);

    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(mesh).into(),
        material: materials.add(ColorMaterial::from(Color::BLUE)),
        ..default()
    });
}

fn update(query: Query<&Mesh2dHandle>, assets: ResMut<Assets<Mesh>>, _time: Res<Time>) {
    let mesh = assets
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
                let vertices = chunk
                    .map(|vertex| *vertex)
                    .collect::<Vec<[f32; 3]>>();

                vertices
                    .iter()
                    .map(|vertex| [
                            vertex[0] + (0.05 * (vertices[0][1] - vertices[1][1])),
                            vertex[1] + (-0.05 * (vertices[0][0] - vertices[1][0])),
                            0.
                        ])
                    .collect::<Vec<[f32; 3]>>()
            })
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
