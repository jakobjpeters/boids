
use std::{f32::{consts::PI, EPSILON}, iter::zip};
use rand::random;
use bevy::{
    prelude::*,
    render::mesh,
    sprite::MaterialMesh2dBundle,
    diagnostic::{LogDiagnosticsPlugin, FrameTimeDiagnosticsPlugin}
};

const BORDER: f32 = 512.;
const BOIDS: usize = 32;
const SCALE: f32 = 8.;
const SPEED: f32 = 100.;
const ALIGNMENT: f32 = 1.;

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

fn boids(mut transforms: Query<&mut Transform, With<Boid>>, time: Res<Time>) {
    let velocities = transforms
        .iter()
        .map(|&a| transforms
                .iter()
                .filter(|b| a != **b)
                .map(|b| b.local_y() / a.translation.distance(b.translation))
                .sum::<Vec3>()
                .normalize_or_zero()
            )
        .collect::<Vec<Vec3>>();

    for (mut transform, velocity) in zip(&mut transforms, velocities) {
        let a = if velocity == Vec3::ZERO { transform.local_y() } else {
            velocity
        };

        let forward_dot = transform.local_y().dot(a);
        let right = transform.rotation * Vec3::X;
        let right_dot = right.dot(a);
        let sign = -f32::copysign(1.0, right_dot);
        let max_angle = forward_dot.clamp(-1.0, 1.0).acos();
        let rotation_angle = sign * (ALIGNMENT * time.delta_seconds()).min(max_angle);
        transform.rotate_z(rotation_angle);

        transform.translation = (
            transform.translation + SPEED * transform.local_y() * time.delta_seconds() + BORDER
        ).rem_euclid(2. * BORDER * Vec3::ONE) - BORDER;
    }
}
