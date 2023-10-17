
use std::f32::consts::PI;
use rand::random;
use bevy::{prelude::*, render::mesh, sprite::MaterialMesh2dBundle};

const BORDER: f32 = 256.;
const BOIDS: usize = 10;

#[derive(Component)]
struct Boid;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for _ in 1..BOIDS {
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
                    scale: Vec3::splat(8.)
                },
                material: materials.add(ColorMaterial::from(Color::BLUE)),
                ..default()
            }
        ));
    }

    commands.spawn(Camera2dBundle::default());
}

fn update(mut transforms: Query<&mut Transform, With<Boid>>, time: Res<Time>) {
    for mut transform in &mut transforms {
        transform.translation = (
            transform.translation + transform.local_y() * 100. * time.delta_seconds() + BORDER
        ).rem_euclid(2. * BORDER * Vec3::ONE) - BORDER;
    }
}
