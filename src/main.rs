
use std::{iter::zip, f32::consts::PI};
use rand::{random, thread_rng, Rng};
use bevy::{
    prelude::*,
    render::mesh,
    window::PrimaryWindow,
    sprite::MaterialMesh2dBundle,
    text::Text2dBounds,
    diagnostic::{LogDiagnosticsPlugin, FrameTimeDiagnosticsPlugin}
};

const MARGIN: f32 = 32.;
const WIDTH: f32 = 256. + 2. * MARGIN;
const BOIDS: usize = 128;
const BOID_COLOR: Color = Color::TEAL;
const SCALE: f32 = 4.;
const SPEED: f32 = 128.;
const ALIGNMENT: f32 = 8.;
const COHESION: f32 = 1.;
const SEPARATION: f32 = 64.;

#[derive(Component)]
struct Boid;

#[derive(Component)]
struct Parameter;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum State {
    #[default]
    Menu,
    Play,
    Pause
}

fn main() { App::new()
    .add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "boids".into(),
                ..default()
            }),
            ..default()
        }),
        LogDiagnosticsPlugin::default(),
        FrameTimeDiagnosticsPlugin::default()
    ))
    .add_state::<State>()
    .add_systems(Startup, setup)
    .add_systems(Update, (
        menu.run_if(in_state(State::Menu)),
        play.run_if(in_state(State::Play)),
        pause.run_if(in_state(State::Pause)),
        buttons.run_if(in_state(State::Play).or_else(in_state(State::Pause)))
    ))
    .add_systems(OnEnter(State::Menu), transition)
    .add_systems(OnExit(State::Menu), transition)
    .run();
}

fn resolution(window: Query<&Window, With<PrimaryWindow>>) -> Vec3 {
    let resolution = &window.single().resolution;
    Vec3::new(resolution.width() - WIDTH, resolution.height(), 1.)
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let resolution = resolution(window);

    commands.spawn(Camera2dBundle::default());

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
                r#"
This is the start menu. Press `escape` to start the simulation or `space` to exit the menu. Each triangle represents a `boid`, which is a simulated bird. They will move forward and adjust their direction depending on three parameters:

1) Separation: boids avoid crashing into each other
2) Alignment: boids rotate to face the same direction as nearby boids
3) Cohesion: boids try to flock together in groups

You can adjust how strong each of these parameters are by moving the sliders on the right-hand side of the simulation. Once the simulation is running, you can return to this menu by pressing `escape` again.
                "#,
                TextStyle {
                    font_size: 32.,
                    ..default()
                }
            )
        ]),
        visibility: Visibility::Hidden,
        text_2d_bounds: Text2dBounds { size: resolution.xy() },
        ..default()
    });

    for _ in 1..BOIDS + 1 {
        let mut mesh = Mesh::new(mesh::PrimitiveTopology::TriangleList);
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![[-1., 0., 0.], [1., 0., 0.], [0., 3., 0.]],
        );

        commands.spawn((Boid, MaterialMesh2dBundle {
            mesh: meshes.add(mesh).into(),
            transform: Transform {
                translation: Vec3::new(
                    resolution.x * random::<f32>() - (resolution.x - WIDTH) / 2.,
                    resolution.y * random::<f32>() - resolution.y / 2.,
                0.),
                rotation: Quat::from_rotation_z(2.0 * PI * random::<f32>()),
                scale: Vec3::splat(SCALE)
            },
            material: materials.add(ColorMaterial::from(BOID_COLOR)),
            visibility: Visibility::Visible,
            ..default()
        }));
    }

    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Px(WIDTH),
            height: Val::Percent(100.),
            ..default()
        },
        background_color: Color::GRAY.into(),
        visibility: Visibility::Visible,
        ..default()
    })
    .with_children(|parent| {
        parent.spawn(TextBundle {
            text: Text::from_sections([
                TextSection::new("Controls\n\n", TextStyle { color: Color::BLACK, font_size: 32., ..default() }),
                TextSection::new("Press `escape` to see\nthe start menu\n\n", TextStyle { color: Color::BLACK, font_size: 24., ..default() }),
                TextSection::new("Press `space` to\npause or unpause\nthe simulation\n\n", TextStyle { color: Color::BLACK, font_size: 24., ..default() }),
                TextSection::new("Press the `-` and `+` buttons to adjust\neach parameter\n\n", TextStyle { color: Color::BLACK, font_size: 24., ..default() }),
                TextSection::new("Separation:\n\n", TextStyle { color: Color::BLACK, font_size: 24., ..default() }),
                TextSection::new("Alignment:\n\n", TextStyle { color: Color::BLACK, font_size: 24., ..default() }),
                TextSection::new("Cohesion:", TextStyle { color: Color::BLACK, font_size: 24., ..default() }),
            ]),
            style: Style {
                top: Val::Px(MARGIN),
                left: Val::Px(MARGIN),
                ..default()
            },
            ..default()
        });
    });

    for top in [360., 409., 458.] {
        commands.spawn(NodeBundle {
            style: Style {
                left: Val::Px(210.),
                top: Val::Px(top),
                ..default()
            },
            z_index: ZIndex::Local(1),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((Parameter, TextBundle {
                text: Text::from_section("X", TextStyle { color: Color::BLACK, font_size: 24., ..default() }),
                ..default()
            }))
            .with_children(|parent| {
                for (offset, value) in [(-40., "-"), (10., "+")] {
                    parent.spawn(ButtonBundle {
                        style: Style {
                            width: Val::Px(24.0),
                            height: Val::Px(24.0),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            left: Val::Px(offset),
                            ..default()
                        },
                        background_color: BOID_COLOR.into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section(
                            value,
                            TextStyle {
                                font_size: 24.,
                                color: Color::BLACK,
                                ..default()
                            }
                        ));
                    });
                }
            });
        });
    }
}

fn transition(mut visibilities: Query<&mut Visibility>, mut styles: Query<&mut Style>) {
    visibilities.for_each_mut(|mut visibility| *visibility = match *visibility {
        Visibility::Visible => Visibility::Hidden,
        Visibility::Hidden => Visibility::Visible,
        Visibility::Inherited => Visibility::Inherited
    });

    styles.for_each_mut(|mut style| style.display = match style.display {
        Display::None => Display::DEFAULT,
        _ => Display::None
    });
}

fn menu(mut next_state: ResMut<NextState<State>>, keys: Res<Input<KeyCode>>) {
    next_state.set(match keys.get_just_pressed().next() {
        Some(KeyCode::Escape) => State::Play,
        Some(KeyCode::Space) => State::Pause,
        _ => State::Menu
    });
}

fn rotate(transform: &mut Transform, x: Vec3, scale: f32) { if x != Vec3::ZERO {
    let normal = x.normalize();
    let forward_dot = transform.local_y().dot(normal);
    let right_dot = (transform.rotation * Vec3::X).dot(normal);
    transform.rotate_z(-f32::copysign(1.0, right_dot) * scale.min(forward_dot.clamp(-1.0, 1.0).acos()));
} }

fn play(
    mut next_state: ResMut<NextState<State>>,
    keys: Res<Input<KeyCode>>,
    mut transforms: Query<&mut Transform, With<Boid>>,
    time: Res<Time>,
    window: Query<&Window, With<PrimaryWindow>>
) {
    next_state.set(match keys.get_just_pressed().next() {
        Some(KeyCode::Escape) => State::Menu,
        Some(KeyCode::Space) => State::Pause,
        _ => State::Play
    });

    let delta_seconds = time.delta_seconds();
    let resolution = resolution(window);
    let offset = (resolution - (WIDTH * Vec3::X)) / 2.;
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
        let [centroid, nearest] = [1, 2].map(|i| x[i] - transform.translation);

        rotate(&mut transform, x[0], ALIGNMENT * delta_seconds * x[0].length());
        rotate(&mut transform, centroid, COHESION * delta_seconds * centroid.length() / BOIDS as f32);
        rotate(&mut transform, -nearest, SEPARATION * delta_seconds / nearest.length());

        transform.translation = (
            transform.translation + SPEED * transform.local_y() * time.delta_seconds() + offset
        ).rem_euclid(resolution) - offset;
    }
}

fn pause(mut next_state: ResMut<NextState<State>>, keys: Res<Input<KeyCode>>) {
    next_state.set(match keys.get_just_pressed().next() {
        Some(KeyCode::Escape) => State::Menu,
        Some(KeyCode::Space) => State::Play,
        _ => State::Pause
    });
}

fn buttons(
    mut query: Query<(&mut Text, &Children), With<Parameter>>,
    mut interactions: Query<(&Interaction, &mut BorderColor), (Changed<Interaction>, With<Button>)>
) {
    for (mut text, children) in &mut query {
        for child in children {
            let s = thread_rng().gen_range(1..11).to_string();
            match interactions.get_mut(*child) {
                Ok((interaction, mut border_color)) => {
                    match *interaction {
                        Interaction::Pressed => {
                            border_color.0 = Color::BLUE;
                            *text = Text::from_section(s, TextStyle {
                                font_size: 24.,
                                color: Color::BLACK,
                                ..default()
                            });
                        },
                        Interaction::Hovered => border_color.0 = Color::WHITE,
                        Interaction::None => border_color.0 = Color::BLACK
                    }
                },
                _ => ()
            }
        }
    }
}

fn _buttons(mut interactions: Query<(&Interaction, &mut BorderColor), (Changed<Interaction>, With<Button>)>) {
    for (interaction, mut border_color) in &mut interactions {
        match *interaction {
            Interaction::Pressed => border_color.0 = Color::BLUE,
            Interaction::Hovered => border_color.0 = Color::WHITE,
            Interaction::None => border_color.0 = Color::BLACK
        }
    }
}
