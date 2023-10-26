
use std::f32::consts::PI;
use rand::random;
use itertools::Itertools;
use bevy::{
    prelude::*,
    reflect::{TypeUuid, TypePath},
    sprite::Mesh2dHandle,
    asset::{Asset, Handle},
    render::mesh::PrimitiveTopology::TriangleList,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin}
};

#[derive(Component)]
// struct Boids<M> where M: TypeUuid + TypePath {
struct Boids<M> where M: Asset {
    mesh: Handle<Mesh>,
    material: Handle<M>
}

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
    let vertices: Vec<[f32; 3]> = (1..100)
        .flat_map(|_| {
            let position = 512. * (Vec2::new(random(), random()) - 0.5);
            let rotation = Vec2::from_angle(2. * PI * random::<f32>());
            [rotation.rotate(points[0]), rotation.rotate(points[1]), Vec2::ZERO]
                .map(|point| (point + position).extend(0.).to_array())
        })
        .collect();

    let mut mesh = Mesh::new(TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);

    commands.spawn(Boids {
        mesh: meshes.add(mesh),
        material: materials.add(ColorMaterial::from(Color::BLUE))
    });

    // commands.spawn((
    //     materials.add(ColorMaterial::from(Color::BLUE)),
    //     meshes.add(mesh)
    // ));

    // commands.spawn(MaterialMesh2dBundle {
    //     material: materials.add(ColorMaterial::from(Color::BLUE)),
    //     mesh: meshes.add(mesh).into(),
    //     ..default()
    // });
}

// fn update(boids: ResMu/t<Assets<Mesh>>, _time: Res<Time>) {
fn update<M: Asset>(boids: Boids<M>, _time: Res<Time>) {
    // let mesh = boids
    //     .into_inner()
    //     .iter_mut()
    //     .at_most_one()
    //     .unwrap_or_else(|_| None)
    //     .unwrap()
    //     .1;

    let vertices = boids.mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .unwrap() 
        .as_float3()
        .unwrap()
        .into_iter()
        .chunks(3)
        .into_iter()
        .flat_map(|triangle| {
            triangle.map(|vertex| *vertex)
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
