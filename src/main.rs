use bevy::{
    math::bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume},
    prelude::*,
    window::*,
};

const WINDOW_WIDTH: f32 = 900.;
const WINDOW_HEIGHT: f32 = 600.;
const BACHGROUND_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);
const SQUARE_SIDE: f32 = 64.;
const SQUARE_COLOR: Color = Color::srgb(0.1, 0.1, 1.0);
const BALL_COLOR: Color = Color::srgb(0.1, 1.0, 0.2);
const SQUARE_SPEED: f32 = 300.0;
const BALL_DIAMETER: f32 = 32.0;
const BALL_DIRECTION: Vec2 = Vec2::new(0.5, -0.5);
const BALL_SPEED: f32 = 500.0;

#[derive(Component)]
struct Square;

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Collider;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Event, Default)]
struct CollisionEvent;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Collision {
    Left,
    Right,
    Top,
    Bottom,
}

// Returns `Some` if `ball` collides with `bounding_box`.
// The returned `Collision` is the side of `bounding_box` that `ball` hit.
fn ball_collision(ball: BoundingCircle, bounding_box: Aabb2d) -> Option<Collision> {
    if !ball.intersects(&bounding_box) {
        return None;
    }

    let closest = bounding_box.closest_point(ball.center());
    let offset = ball.center() - closest;
    let side = if offset.x.abs() > offset.y.abs() {
        if offset.x < 0. {
            Collision::Left
        } else {
            Collision::Right
        }
    } else if offset.y > 0. {
        Collision::Top
    } else {
        Collision::Bottom
    };

    Some(side)
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let square_y = 100.0;

    commands.spawn(Camera2d);

    commands.spawn((
        Sprite::from_color(SQUARE_COLOR, Vec2::ONE),
        Transform {
            translation: Vec3::new(0.0, square_y, 0.0),
            scale: Vec2::new(SQUARE_SIDE, SQUARE_SIDE).extend(1.0),
            ..default()
        },
        Square,
        Collider,
    ));

    commands.spawn((
        Mesh2d(meshes.add(Circle::default())),
        MeshMaterial2d(materials.add(BALL_COLOR)),
        Transform {
            translation: Vec3::new(0.0, square_y + 100.0, 0.0),
            scale: Vec2::splat(BALL_DIAMETER).extend(1.0),
            ..default()
        },
        Ball,
        Velocity(BALL_DIRECTION.normalize() * BALL_SPEED),
    ));
}

fn check_for_collisions(
    mut commands: Commands,
    ball_query: Single<(&mut Velocity, &Transform), With<Ball>>,
    collider_query: Query<(Entity, &Transform, Option<&Square>), With<Collider>>,
    //mut collision_events: EventWriter<CollisionEvent>,
) {
    let (mut ball_velocity, ball_transform) = ball_query.into_inner();
    for (collider_entity, collider_transform, maybe_square) in &collider_query {
        let collision = ball_collision(
            BoundingCircle::new(ball_transform.translation.truncate(), BALL_DIAMETER / 2.),
            Aabb2d::new(
                collider_transform.translation.truncate(),
                collider_transform.scale.truncate() / 2.,
            ),
        );
        if let Some(collision) = collision {
            if maybe_square.is_some() {
                match collision {
                    Collision::Left | Collision::Right => ball_velocity.x *= -1.,
                    Collision::Top | Collision::Bottom => ball_velocity.y *= -1.,
                }
                println!("Hit!!");
            }
        }
    }
}

fn move_square(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut square_transform: Single<&mut Transform, With<Square>>,
    time: Res<Time>,
) {
    let mut direction = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        direction.x = 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }

    square_transform.translation.x += direction.x * SQUARE_SPEED * time.delta_secs();
    square_transform.translation.y += direction.y * SQUARE_SPEED * time.delta_secs();
}

fn apply_velocity(mut query: Query<(&mut Transform, &mut Velocity)>, time: Res<Time>) {
    for (mut transform, mut velocity) in &mut query {
        let x = transform.translation.x + velocity.x * time.delta_secs();
        let y = transform.translation.y + velocity.y * time.delta_secs();
        if x > WINDOW_WIDTH / 2. || x < -WINDOW_WIDTH / 2. {
            velocity.x *= -1.;
        }
        if y > WINDOW_HEIGHT / 2. || y < -WINDOW_HEIGHT / 2. {
            velocity.y *= -1.;
        }
        transform.translation.x = x.clamp(-450.0, 450.0);
        transform.translation.y = y.clamp(-300.0, 300.0);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(BACHGROUND_COLOR))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (apply_velocity, move_square, check_for_collisions).chain(),
        )
        .run();
}
