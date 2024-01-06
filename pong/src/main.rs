use bevy::{prelude::*, window::close_on_esc, sprite::collide_aabb::{collide, Collision}};

use rand::Rng;

const PLAYER_START_POSITION: f32 = 1920. / 4. + 1920. / 5.;
const PLAYER_SPEED: f32 = 550.;

const COMPUTER_SPEED: f32 = 500.;

const BALL_SPEED: f32 = 700.;

const WALL_HEIGHT: f32 = 1080. / 2.;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Computer;

#[derive(Component)]
pub struct Ball;

#[derive(Component)]
pub struct Collider(Vec2);

#[derive(Event)]
pub struct BallDestroyed {
    pub player_scored: bool
}

#[derive(Event)]
pub struct BallCollided;

impl Collider {
    pub fn cuboid(x: f32, y: f32) -> Self {
        Self(Vec2{x, y})
    }

    pub fn circle(radius: f32) -> Self {
        let diameter = radius * 2.;
        Self(Vec2::new(diameter, diameter))
    }
}

#[derive(Component, Default, Debug)]
pub struct Velocity(Vec2);

#[derive(Resource, Default)]
pub struct Scoreboard {
    pub player: u32,
    pub computer: u32,
}

#[derive(Component)]
pub struct Score(bool);

fn main() {
    App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Pong!".into(),
            mode: bevy::window::WindowMode::BorderlessFullscreen,
            ..default()
        }),
        ..default()    
    }))
    .insert_resource(ClearColor(Color::BLACK))
    .insert_resource(Scoreboard::default())
    .add_event::<BallDestroyed>()
    .add_event::<BallCollided>()
    .add_systems(Update, close_on_esc)
    .add_systems(Update, reset_on_r)
    .add_systems(Startup, (setup, spawn_ball))
    .add_systems(Update, computer_movement_control.before(velocity_movement))
    .add_systems(Update, player_movement_control.before(velocity_movement))
    .add_systems(Update, ball_collision.before(velocity_movement))
    .add_systems(Update, (velocity_movement, despawn_ball, award_points).chain())
    .add_systems(Update, respawn_ball.after(despawn_ball))
    .add_systems(Update, (update_scores, collision_sounds))
    .run();
}

fn reset_on_r(input: Res<Input<KeyCode>>, mut scoreboard: ResMut<Scoreboard>, mut ball: Query<(&mut Transform, &mut Velocity), (With<Ball>, Without<Player>, Without<Computer>)>, mut computer: Query<&mut Transform, (With<Computer>, Without<Ball>, Without<Player>)>, mut player: Query<&mut Transform, (With<Player>, Without<Ball>, Without<Computer>)>) {
    if input.pressed(KeyCode::R) {
        scoreboard.player = 0;
        scoreboard.computer = 0;
        for (mut ball_transform, mut ball_velocity) in ball.iter_mut() {
            ball_transform.translation = Vec3::ZERO;
            ball_velocity.0 = Vec2::new(coin_flip(), coin_flip()).normalize() * BALL_SPEED;
        }
        for mut computer_transform in computer.iter_mut() {
            computer_transform.translation = Vec3::new(-PLAYER_START_POSITION, 0., 0.);
        }
        for mut player_transform in player.iter_mut() {
            player_transform.translation = Vec3::new(PLAYER_START_POSITION, 0., 0.);
        }
    }
    
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        TransformBundle {
            local: Transform::from_translation(Vec3::new(0., WALL_HEIGHT + 10., 0.)),
            ..default()
        },
        Collider::cuboid(1920., 10.),
    ));

    commands.spawn((
        TransformBundle {
            local: Transform::from_translation(Vec3::new(0., -WALL_HEIGHT - 10., 0.)),
            ..default()
        },
        Collider::cuboid(1920., 10.),
    ));

    // divider line
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(2., 1080.)),
            ..default()
        },
        ..default()
    });

    // paddle one
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/player.png"),
            transform: Transform::from_translation(Vec3::new(PLAYER_START_POSITION, 0., 0.)),
            ..default()
        },
        Player,
        Collider::cuboid(17., 120.),
        Velocity::default(),
    ));

    // paddle two
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/computer.png"),
            transform: Transform::from_translation(Vec3::new(-PLAYER_START_POSITION, 0., 0.)),
            ..default()
        },
        Computer,
        Collider::cuboid(17., 120.),
        Velocity::default(),
    ));

    // score contianer
    let container = commands.spawn(NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceAround,
            width: Val::Percent(100.),
            ..default()
        },
        ..default()
    }).id();

    // player score
    let player_score = commands.spawn((
        Score(true),
        TextBundle::from_section("0", TextStyle {
            font_size: 96.,
            color: Color::WHITE,
            ..default()
        })
    )).id();

    // computer score
    let computer_score = commands.spawn((
        Score(false),
        TextBundle::from_section("0", TextStyle {
            font_size: 96.,
            color: Color::WHITE,
            ..default()
        })
    )).id();

    commands.entity(container).push_children(&[computer_score, player_score]);
}

fn coin_flip() -> f32 {
    let mut random = rand::thread_rng();

    let number = random.gen::<u32>() % 2;
    if number == 0 { -1. } else { 1. }
}

fn player_movement_control(mut query: Query<&mut Velocity, With<Player>>, input: Res<Input<KeyCode>>, time: Res<Time>) {
    let mut direction: Vec2 = Vec2::ZERO;
    if input.pressed(KeyCode::W) {
        direction.y = 1.0;
    } else if input.pressed(KeyCode::S) {
        direction.y = -1.0;
    }

    for mut velocity in query.iter_mut() {
        velocity.0 = direction * PLAYER_SPEED;
    }
}

fn computer_movement_control(mut computer: Query<(&mut Velocity, &Transform), With<Computer>>, ball: Query<&Transform, With<Ball>>) {
    let ball_transform = if let Ok(transform) = ball.get_single() {
        transform
    } else { return; };

    for (mut velocity, computer_transform) in computer.iter_mut() {
        if ball_transform.translation.x > 0. {
            if (50.0..-50.0).contains(&computer_transform.translation.y) {velocity.0.y = 0.;}
            else if 0. > computer_transform.translation.y {velocity.0.y = COMPUTER_SPEED / 2.}
            else if 0. < computer_transform.translation.y {velocity.0.y = -COMPUTER_SPEED / 2.}
            continue;
        }

        let speed = if ball_transform.translation.x > -(1920. * 0.275) {COMPUTER_SPEED * 0.7} else {COMPUTER_SPEED};

        if ball_transform.translation.y < computer_transform.translation.y {
            velocity.0.y = -speed;
        } else if ball_transform.translation.y > computer_transform.translation.y {
            velocity.0.y = speed;
        }
    }
}

fn ball_collision(mut ball: Query<(&mut Velocity, &Collider, &Transform), With<Ball>>, others: Query<(&Collider, &Transform), Without<Ball>>, mut events: EventWriter<BallCollided>) {
    let (mut velocity, ball_collider, ball_transform) = {
        if let Ok(ball) = ball.get_single_mut() { ball } else { return; }
    };

    for (other_collider, other_transform) in others.iter() {
        match if let Some(collision) = collide(ball_transform.translation, ball_collider.0, other_transform.translation, other_collider.0) {
            events.send(BallCollided);
             collision } else { continue; } {
            Collision::Left | Collision::Right => velocity.0.x *= -1.,
            Collision::Top | Collision::Bottom => velocity.0.y *= -1.,
            Collision::Inside => (),
        }

    }
}

fn velocity_movement(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.0.extend(0.) * time.delta_seconds();
    }
}

 fn despawn_ball(mut events: EventWriter<BallDestroyed>, mut commands: Commands, query: Query<(&Transform, Entity), With<Ball>>) {
    for (transform, entity) in query.iter() {
        let mut player_scored = false;
        if transform.translation.x > 1920. / 2. + 10. {
            player_scored = false;
        } else if transform.translation.x < -(1920. / 2. + 10.) {
            player_scored = true;   
        } else {
            continue;
        }
        commands.entity(entity).despawn_recursive();
        events.send(BallDestroyed { player_scored })
    }
 }

 fn award_points(mut events: EventReader<BallDestroyed>, mut scoreboard: ResMut<Scoreboard>, mut commands: Commands, asset_server: Res<AssetServer>) {
    for event in events.read() {
        if event.player_scored {
            scoreboard.player += 1;

            commands.spawn(AudioBundle {
                source: asset_server.load("sounds/score.ogg"),
                settings: PlaybackSettings::DESPAWN,
            }); 
        } else {
            scoreboard.computer += 1;
        }
    }
 }

 fn update_scores(scoreboard: Res<Scoreboard>, mut query: Query<(&mut Text, &Score)>) {
    for (mut text, score) in query.iter_mut() {
        text.sections.clear();
        if score.0 {
            text.sections.push(TextSection::new(scoreboard.player.to_string(), TextStyle {
                font_size: 96.,
                color: Color::WHITE,
                ..default()
            })); 
        } else {
            text.sections.push(TextSection::new(scoreboard.computer.to_string(), TextStyle {
                font_size: 96.,
                color: Color::WHITE,
                ..default()
            }))
        }
    }
 }

 fn spawn_ball(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    let ball_direction = Vec2::new(coin_flip(), coin_flip()).normalize();

    // ball
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/ball.png"),
            ..default()
        },
        Ball,
        Collider::circle(15.),
        Velocity(ball_direction * BALL_SPEED),
    ));
 }

 fn respawn_ball(mut events: EventReader<BallDestroyed>, mut commands: Commands, asset_server: ResMut<AssetServer>) {
    for _ in events.read() {
        let ball_direction = Vec2::new(coin_flip(), coin_flip()).normalize();

        // ball
        commands.spawn((
            SpriteBundle {
                texture: asset_server.load("sprites/ball.png"),
                ..default()
            },
            Ball,
            Collider::circle(15.),
            Velocity(ball_direction * BALL_SPEED),
        ));
    }
 }

 fn collision_sounds(mut events: EventReader<BallCollided>, mut commands: Commands, asset_server: Res<AssetServer>) {
    for _ in events.read() {
        commands.spawn(AudioBundle {
            source: asset_server.load("sounds/bounce.ogg"),
            settings: PlaybackSettings::DESPAWN,
        });
    }
 }