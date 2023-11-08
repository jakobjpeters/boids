
use std::{iter::zip, f32::consts::PI};
use rand::random;
use bevy::{
    prelude::*,
    render::mesh,
    sprite::MaterialMesh2dBundle,
    diagnostic::{LogDiagnosticsPlugin, FrameTimeDiagnosticsPlugin}
};

const BORDER: f32 = 512.;
const BOIDS: usize = 128;
const SCALE: f32 = 4.;
const SPEED: f32 = 128.;
const ALIGNMENT: f32 = 8.;
const COHESION: f32 = 1.;
const SEPARATION: f32 = 64.;

#[derive(Component)]
struct Boid;

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
    for _ in 1..BOIDS + 1 {
        let mut mesh = Mesh::new(mesh::PrimitiveTopology::TriangleList);
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![[-1., 0., 0.], [1., 0., 0.], [0., 3., 0.]],
        );

        commands.spawn((
            Boid,
            MaterialMesh2dBundle {
                mesh: meshes.add(mesh).into(),
                transform: Transform {
                    translation: Vec3::new(
                        2. * BORDER * random::<f32>() - BORDER,
                        2. * BORDER * random::<f32>() - BORDER,
                    0.),
                    rotation: Quat::from_rotation_z(2.0 * PI * random::<f32>()),
                    scale: Vec3::splat(SCALE)
                },
                material: materials.add(ColorMaterial::from(Color::BLUE)),
                visibility: Visibility::Hidden,
                ..default()
            }
        ));
    }

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

fn rotate(transform: &mut Transform, x: Vec3, scale: f32) { if x != Vec3::ZERO {
    let normal = x.normalize();
    let forward_dot = transform.local_y().dot(normal);
    let right_dot = (transform.rotation * Vec3::X).dot(normal);
    transform.rotate_z(-f32::copysign(1.0, right_dot) * scale.min(forward_dot.clamp(-1.0, 1.0).acos()));
} }

fn boids(mut transforms: Query<&mut Transform, With<Boid>>, time: Res<Time>) {
    let xs = transforms
        .iter()
        .map(|&a| transforms
                .iter()
                .filter(|b| a.translation != b.translation)
                .map(|b| {
                        let distance = a.translation.distance(b.translation);
                        (b.local_y() / distance, b.translation / distance, (distance, b.translation))
                    })
                .reduce(|xs, ys| (xs.0 + ys.0, xs.1 + ys.1, if xs.2.0 < ys.2.0 { xs.2 } else { ys.2 }))
                .map(|xs| [xs.0, xs.1, xs.2.1])
                .unwrap()
            )
        .collect::<Vec<[Vec3; 3]>>();

    for (mut transform, x) in zip(&mut transforms, xs) {
        let delta_seconds = time.delta_seconds();
        let [centroid, nearest] = [1, 2].map(|i| x[i] - transform.translation);

        rotate(&mut transform, x[0], ALIGNMENT * delta_seconds * x[0].length());
        rotate(&mut transform, centroid, COHESION * delta_seconds * centroid.length() / BOIDS as f32);
        rotate(&mut transform, -nearest, SEPARATION * delta_seconds / nearest.length());

        transform.translation = (
            transform.translation + SPEED * transform.local_y() * time.delta_seconds() + BORDER
        ).rem_euclid(2. * BORDER * Vec3::ONE) - BORDER;
    }
}
