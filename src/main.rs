
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
const EXPLANATION: &str = r#"
This is the start menu. After you press `escape`, the simulation will start.
Each triangle represents a `boid`, which is a simulated bird.
They will move forward and adjust their direction depending on three parameters:

1) Separation: boids avoid crashing into each other
2) Alignment: boids rotate to face the same direction as nearby boids
3) Cohesion: boids try to flock together in groups

You can adjust how strong each of these parameters are by moving the sliders on the ride-hand side of the simulation.
Once the simulation is running, you can return to this menu by pressing `escape` again.
"#;

#[derive(Component)]
struct Boid;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Menu,
    Play,
    Pause
}

fn main() { App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "boids".into(),
                ..default()
            }),
            ..default()
        }))
    .add_plugins(LogDiagnosticsPlugin::default())
    .add_plugins(FrameTimeDiagnosticsPlugin::default())
    .add_state::<AppState>()
    .add_systems(Startup, setup)
    .add_systems(Update, menu.run_if(in_state(AppState::Menu)))
    .add_systems(Update, play.run_if(in_state(AppState::Play)))
    .add_systems(Update, pause.run_if(in_state(AppState::Pause)))
    .add_systems(OnEnter(AppState::Menu), transition)
    .add_systems(OnExit(AppState::Menu), transition)
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
                visibility: Visibility::Visible,
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
                EXPLANATION,
                TextStyle {
                    font_size: 32.,
                    ..default()
                }
            )
        ]),
        visibility: Visibility::Hidden,
        ..default()
    });

    commands.spawn(Camera2dBundle::default());
}

fn transition(mut visibilities: Query<&mut Visibility>) {
    visibilities.for_each_mut(|mut visibility| *visibility = match *visibility {
        Visibility::Visible => Visibility::Hidden,
        Visibility::Hidden => Visibility::Visible,
        Visibility::Inherited => Visibility::Inherited
    })
}

fn menu(mut next_state: ResMut<NextState<AppState>>, keys: Res<Input<KeyCode>>) {
    next_state.set(match keys.get_just_pressed().next() {
        Some(KeyCode::Escape) => AppState::Play,
        Some(KeyCode::Space) => AppState::Pause,
        _ => AppState::Menu
    });
}

fn rotate(transform: &mut Transform, x: Vec3, scale: f32) { if x != Vec3::ZERO {
    let normal = x.normalize();
    let forward_dot = transform.local_y().dot(normal);
    let right_dot = (transform.rotation * Vec3::X).dot(normal);
    transform.rotate_z(-f32::copysign(1.0, right_dot) * scale.min(forward_dot.clamp(-1.0, 1.0).acos()));
} }

fn play(
    mut next_state: ResMut<NextState<AppState>>,
    keys: Res<Input<KeyCode>>,
    mut transforms: Query<&mut Transform, With<Boid>>,
    time: Res<Time>
) {
    next_state.set(match keys.get_just_pressed().next() {
        Some(KeyCode::Escape) => AppState::Menu,
        Some(KeyCode::Space) => AppState::Pause,
        _ => AppState::Play
    });

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

fn pause(mut next_state: ResMut<NextState<AppState>>, keys: Res<Input<KeyCode>>) {
    next_state.set(match keys.get_just_pressed().next() {
        Some(KeyCode::Escape) => AppState::Menu,
        Some(KeyCode::Space) => AppState::Play,
        _ => AppState::Pause
    });
}
