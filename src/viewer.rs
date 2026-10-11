use std::collections::HashMap;
use std::env;

use bevy::app::AppExit;
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use cellworld::{Coord3, World};

const SIMULATION_STEP_SECONDS: f64 = 0.5;
const WORLD_SPACING: f32 = 1.5;
const ORBIT_SENSITIVITY: f32 = 0.005;
const PAN_SENSITIVITY: f32 = 0.002;

#[derive(Resource)]
struct Simulation {
    world: World,
    feed_per_tick: f32,
    ticks: u64,
    revision: u64,
    paused: bool,
}

#[derive(Resource)]
struct VisualAssets {
    cell_mesh: Handle<Mesh>,
    feed_mesh: Handle<Mesh>,
    cell_material: Handle<StandardMaterial>,
    feed_material: Handle<StandardMaterial>,
}

#[derive(Component)]
struct CellVisual;

#[derive(Component)]
struct FeedVisual;

#[derive(Component)]
struct StatusText;

#[derive(Component)]
struct FeedRateText;

#[derive(Component, Clone, Copy)]
enum FeedAdjustment {
    Decrease,
    Increase,
}

#[derive(Component)]
struct FieldCamera {
    focus: Vec3,
    radius: f32,
    yaw: f32,
    pitch: f32,
}

pub fn run() {
    let feed_per_tick = env::args()
        .nth(1)
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or(1.5);

    let mut world = World::new();
    world.add_feed(Coord3::new(0, 0, 0), 2.8);
    world.add_cell(Coord3::new(0, 0, 0));
    world.add_cell(Coord3::new(1, 0, 0));
    world.add_cell(Coord3::new(-1, 0, 0));

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Cellworld".to_owned(),
                resolution: (1280, 800).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Time::<Fixed>::from_seconds(SIMULATION_STEP_SECONDS))
        .insert_resource(Simulation {
            world,
            feed_per_tick,
            ticks: 0,
            revision: 0,
            paused: false,
        })
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            step_simulation.run_if(|simulation: Res<Simulation>| {
                !simulation.paused && simulation.world.cell_count() > 0
            }),
        )
        .add_systems(
            Update,
            (
                toggle_pause,
                adjust_feed_rate,
                control_camera,
                sync_scene,
                exit_on_extinction,
                update_feed_rate_text,
            )
                .chain(),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    simulation: Res<Simulation>,
) {
    let initial_position = Vec3::new(9.0, 8.0, 12.0);
    let radius = initial_position.length();
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(initial_position).looking_at(Vec3::ZERO, Vec3::Y),
        FieldCamera {
            focus: Vec3::ZERO,
            radius,
            yaw: initial_position.x.atan2(initial_position.z),
            pitch: (initial_position.y / radius).asin(),
        },
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::FRAC_PI_4,
            std::f32::consts::FRAC_PI_8,
            0.0,
        )),
    ));

    let ground_mesh = meshes.add(Plane3d::default().mesh().size(80.0, 80.0));
    let ground_material = materials.add(Color::srgb(0.055, 0.075, 0.09));
    commands.spawn((
        Mesh3d(ground_mesh.clone()),
        MeshMaterial3d(ground_material.clone()),
        Transform::from_xyz(0.0, -1.0, 0.0),
    ));

    commands.insert_resource(VisualAssets {
        cell_mesh: meshes.add(Sphere::new(0.32)),
        feed_mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        cell_material: materials.add(Color::srgb(0.2, 0.85, 0.55)),
        feed_material: materials.add(Color::srgb(1.0, 0.68, 0.16)),
    });

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(16.0),
            left: Val::Px(18.0),
            width: Val::Px(340.0),
            padding: UiRect::all(Val::Px(16.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(14.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.035, 0.055, 0.07, 0.92)),
        children![
            (
                Text::new("CELLWORLD"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                StatusText,
            ),
            (
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    ..default()
                },
                children![
                    (
                        Button,
                        Node {
                            width: Val::Px(42.0),
                            height: Val::Px(38.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.16, 0.22, 0.27)),
                        FeedAdjustment::Decrease,
                        children![(
                            Text::new("-"),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        )],
                    ),
                    (
                        Text::new(format!("Feed / tick: {:.2}", simulation.feed_per_tick)),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            flex_grow: 1.0,
                            ..default()
                        },
                        FeedRateText,
                    ),
                    (
                        Button,
                        Node {
                            width: Val::Px(42.0),
                            height: Val::Px(38.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.16, 0.22, 0.27)),
                        FeedAdjustment::Increase,
                        children![(
                            Text::new("+"),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        )],
                    )
                ],
            )
        ],
    ));
}

fn step_simulation(mut simulation: ResMut<Simulation>) {
    let feed_per_tick = simulation.feed_per_tick;
    simulation.world.step_with_feed(feed_per_tick);
    simulation.ticks += 1;
    simulation.revision += 1;
}

fn toggle_pause(keys: Res<ButtonInput<KeyCode>>, mut simulation: ResMut<Simulation>) {
    if keys.just_pressed(KeyCode::Space) {
        simulation.paused = !simulation.paused;
        simulation.revision += 1;
    }
}

fn adjust_feed_rate(
    buttons: Query<(&Interaction, &FeedAdjustment), Changed<Interaction>>,
    mut simulation: ResMut<Simulation>,
) {
    for (interaction, adjustment) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let next_rate = match adjustment {
            FeedAdjustment::Decrease => (simulation.feed_per_tick - 0.1).max(0.0),
            FeedAdjustment::Increase => simulation.feed_per_tick + 0.1,
        };
        if next_rate != simulation.feed_per_tick {
            simulation.feed_per_tick = next_rate;
            simulation.revision += 1;
        }
    }
}

fn control_camera(
    buttons: Res<ButtonInput<MouseButton>>,
    mut motion: MessageReader<MouseMotion>,
    mut wheel: MessageReader<MouseWheel>,
    mut cameras: Query<(&mut Transform, &mut FieldCamera)>,
) {
    let mut mouse_delta = Vec2::ZERO;
    for event in motion.read() {
        mouse_delta += event.delta;
    }

    let mut scroll_delta = 0.0;
    for event in wheel.read() {
        scroll_delta += match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / MouseScrollUnit::SCROLL_UNIT_CONVERSION_FACTOR,
        };
    }

    for (mut transform, mut camera) in &mut cameras {
        if buttons.pressed(MouseButton::Left) {
            camera.yaw -= mouse_delta.x * ORBIT_SENSITIVITY;
            camera.pitch = (camera.pitch - mouse_delta.y * ORBIT_SENSITIVITY).clamp(-1.5, 1.5);
        }

        if buttons.pressed(MouseButton::Right) {
            let pan_scale = camera.radius * PAN_SENSITIVITY;
            let right = transform.rotation * Vec3::X;
            let up = transform.rotation * Vec3::Y;
            camera.focus += right * (-mouse_delta.x * pan_scale) + up * (mouse_delta.y * pan_scale);
        }

        if scroll_delta != 0.0 {
            camera.radius = (camera.radius * (-scroll_delta * 0.12).exp()).clamp(2.0, 120.0);
        }

        let horizontal_radius = camera.radius * camera.pitch.cos();
        let offset = Vec3::new(
            camera.yaw.sin() * horizontal_radius,
            camera.pitch.sin() * camera.radius,
            camera.yaw.cos() * horizontal_radius,
        );
        transform.translation = camera.focus + offset;
        transform.look_at(camera.focus, Vec3::Y);
    }
}

fn sync_scene(
    mut commands: Commands,
    simulation: Res<Simulation>,
    assets: Res<VisualAssets>,
    cell_entities: Query<Entity, With<CellVisual>>,
    feed_entities: Query<Entity, With<FeedVisual>>,
    mut status_texts: Query<&mut Text, With<StatusText>>,
    mut rendered_revision: Local<Option<u64>>,
) {
    if *rendered_revision == Some(simulation.revision) {
        return;
    }
    *rendered_revision = Some(simulation.revision);

    for entity in &cell_entities {
        commands.entity(entity).despawn();
    }
    for entity in &feed_entities {
        commands.entity(entity).despawn();
    }

    let mut stacked_cells: HashMap<Coord3, usize> = HashMap::new();
    for cell in &simulation.world.cells {
        let slot = stacked_cells.entry(cell.position).or_default();
        let scatter = if *slot == 0 {
            Vec3::ZERO
        } else {
            let ring_slot = *slot - 1;
            let layer = ring_slot / 8;
            let angle = (ring_slot % 8) as f32 * std::f32::consts::TAU / 8.0;
            let radius = 0.75 + layer as f32 * 0.45;
            Vec3::new(
                angle.cos() * radius,
                layer as f32 * 0.45,
                angle.sin() * radius,
            )
        };
        *slot += 1;

        let energy_scale = (0.7 + cell.energy * 0.12).clamp(0.65, 1.6);
        commands.spawn((
            Mesh3d(assets.cell_mesh.clone()),
            MeshMaterial3d(assets.cell_material.clone()),
            Transform::from_translation(world_position(cell.position) + scatter)
                .with_scale(Vec3::splat(energy_scale)),
            CellVisual,
        ));
    }

    for (&position, &amount) in simulation.world.feed.iter() {
        if amount <= 0.01 {
            continue;
        }
        let marker_size = (0.22 + amount * 0.025).clamp(0.22, 0.55);
        commands.spawn((
            Mesh3d(assets.feed_mesh.clone()),
            MeshMaterial3d(assets.feed_material.clone()),
            Transform::from_translation(world_position(position) + Vec3::Y * 0.48)
                .with_scale(Vec3::splat(marker_size)),
            FeedVisual,
        ));
    }

    let total_feed: f32 = simulation.world.feed.values().copied().sum();
    let state = if simulation.world.cell_count() == 0 {
        "Extinct"
    } else if simulation.paused {
        "Paused"
    } else {
        "Running"
    };
    let status = format!(
        "CELLWORLD\n\
         {state}  |  Tick: {}\n\
         Cells: {}  |  Total energy: {:.2}\n\
         Feed: {:.2}\n\
         \n\
         Green spheres: cells\n\
         Amber cubes: feed\n\
         \n\
         [Space] Pause / resume\n\
         [Esc] Quit",
        simulation.ticks,
        simulation.world.cell_count(),
        simulation.world.total_energy(),
        total_feed,
    );
    for mut text in &mut status_texts {
        text.0.clone_from(&status);
    }
}

fn exit_on_extinction(
    simulation: Res<Simulation>,
    mut app_exit: MessageWriter<AppExit>,
    mut has_exited: Local<bool>,
) {
    if *has_exited || !should_close_on_extinction(simulation.world.cell_count()) {
        return;
    }

    *has_exited = true;
    app_exit.write(AppExit::Success);
}

fn should_close_on_extinction(cell_count: usize) -> bool {
    cell_count == 0
}

fn update_feed_rate_text(
    simulation: Res<Simulation>,
    mut feed_rate_texts: Query<&mut Text, With<FeedRateText>>,
    mut displayed_rate: Local<Option<f32>>,
) {
    if *displayed_rate == Some(simulation.feed_per_tick) {
        return;
    }
    *displayed_rate = Some(simulation.feed_per_tick);

    for mut text in &mut feed_rate_texts {
        text.0 = format!("Feed / tick: {:.2}", simulation.feed_per_tick);
    }
}

fn world_position(position: Coord3) -> Vec3 {
    Vec3::new(
        position.x as f32 * WORLD_SPACING,
        position.y as f32 * WORLD_SPACING,
        position.z as f32 * WORLD_SPACING,
    )
}

#[cfg(test)]
mod tests {
    use super::should_close_on_extinction;

    #[test]
    fn extinct_population_requests_exit() {
        assert!(should_close_on_extinction(0));
        assert!(!should_close_on_extinction(1));
    }
}
