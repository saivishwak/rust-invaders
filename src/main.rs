#![allow(unused)] // silence unused warnings while exploring (to comment out)

use bevy::diagnostic::Diagnostics;
use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;
use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, math::Vec3Swizzles};
use components::{
	Enemy, Explosion, ExplosionTimer, ExplosionToSpawn, FromEnemy, FromPlayer, Laser, Movable,
	Player, Score, SpriteSize, Velocity,
};
use enemy::EnemyPlugin;
use player::PlayerPlugin;
use std::collections::HashSet;

use text::TextPlugin;

mod components;
mod enemy;
mod player;
mod text;

// region:    --- Asset Constants

const PLAYER_SPRITE: &str = "player_a_01.png";
const PLAYER_SIZE: (f32, f32) = (144., 75.);
const PLAYER_LASER_SPRITE: &str = "laser_a_01.png";
const PLAYER_LASER_SIZE: (f32, f32) = (9., 54.);

const ENEMY_SPRITE: &str = "enemy_a_01.png";
const ENEMY_SIZE: (f32, f32) = (144., 75.);
const ENEMY_LASER_SPRITE: &str = "laser_b_01.png";
const ENEMY_LASER_SIZE: (f32, f32) = (17., 55.);

const EXPLOSION_SHEET: &str = "explo_a_sheet.png";
const EXPLOSION_LEN: usize = 16;

const SPRITE_SCALE: f32 = 0.5;

const PLAYER_MAX_LIFE: u32 = 3;

// endregion: --- Asset Constants

// region:    --- Game Constants

const TIME_STEP: f32 = 1. / 60.;
const BASE_SPEED: f32 = 500.;

const PLAYER_RESPAWN_DELAY: f64 = 2.;
const ENEMY_MAX: u32 = 10;
const FORMATION_MEMBERS_MAX: u32 = 2;

// endregion: --- Game Constants

// region:    --- Resources
pub struct WinSize {
	pub w: f32,
	pub h: f32,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
	Splash,
	End,
	Game,
}

struct GameTextures {
	player: Handle<Image>,
	player_laser: Handle<Image>,
	enemy: Handle<Image>,
	enemy_laser: Handle<Image>,
	explosion: Handle<TextureAtlas>,
}

struct EnemyCount(u32);
pub struct GameScore(u32);

pub struct PlayerLife(u32);

struct PlayerState {
	on: bool,       // alive
	last_shot: f64, // -1 if not shot
}
impl Default for PlayerState {
	fn default() -> Self {
		Self {
			on: false,
			last_shot: -1.,
		}
	}
}

impl PlayerState {
	pub fn shot(&mut self, time: f64) {
		self.on = false;
		self.last_shot = time;
	}
	pub fn spawned(&mut self) {
		self.on = true;
		self.last_shot = -1.;
	}
}
// endregion: --- Resources

fn main() {
	App::new()
		.add_state(GameState::Game)
		.insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
		.insert_resource(WindowDescriptor {
			title: "Rust Invaders!".to_string(),
			width: 598.0,
			height: 676.0,
			..Default::default()
		})
		.add_plugins(DefaultPlugins)
		.add_plugin(PlayerPlugin)
		.add_plugin(EnemyPlugin)
		.add_plugin(TextPlugin)
		.add_plugin(FrameTimeDiagnosticsPlugin::default())
		.add_startup_system(setup_system)
		.add_system_set(
			SystemSet::on_update(GameState::Game)
				.with_system(movable_system)
				.with_system(player_laser_hit_enemy_system)
				.with_system(enemy_laser_hit_player_system)
				.with_system(explosion_to_spawn_system)
				.with_system(explosion_animation_system),
		)
		.add_system_set(SystemSet::on_exit(GameState::Game).with_system(despawn_all))
		.add_system_set(SystemSet::on_update(GameState::End).with_system(restart_game))
		.run();
}

fn setup_system(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut texture_atlases: ResMut<Assets<TextureAtlas>>,
	mut windows: ResMut<Windows>,
) {
	// camera
	commands.spawn_bundle(Camera2dBundle::default());

	// capture window size
	let window = windows.get_primary_mut().unwrap();
	let (win_w, win_h) = (window.width(), window.height());

	// position window (for tutorial)
	// window.set_position(IVec2::new(2780, 4900));

	// add WinSize resource
	let win_size = WinSize { w: win_w, h: win_h };
	commands.insert_resource(win_size);

	// create explosion texture atlas
	let texture_handle = asset_server.load(EXPLOSION_SHEET);
	let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64., 64.), 4, 4);
	let explosion = texture_atlases.add(texture_atlas);

	// add GameTextures resource
	let game_textures = GameTextures {
		player: asset_server.load(PLAYER_SPRITE),
		player_laser: asset_server.load(PLAYER_LASER_SPRITE),
		enemy: asset_server.load(ENEMY_SPRITE),
		enemy_laser: asset_server.load(ENEMY_LASER_SPRITE),
		explosion,
	};
	commands.insert_resource(game_textures);
	commands.insert_resource(EnemyCount(0));
	commands.insert_resource(GameScore(0));
	commands.insert_resource(PlayerLife(PLAYER_MAX_LIFE));
}

fn despawn_all(
	mut commands: Commands,
	kb: Res<Input<KeyCode>>,
	mut state: ResMut<State<GameState>>,
	enemy_query: Query<Entity, With<Enemy>>,
	player_query: Query<Entity, With<Player>>,
	laser_query: Query<Entity, With<Laser>>,
	explosion_query: Query<Entity, With<Explosion>>,
) {
	for (enemy_en) in enemy_query.iter() {
		commands.entity(enemy_en).despawn();
	}
	for (player_en) in player_query.iter() {
		commands.entity(player_en).despawn();
	}
	for (laser_en) in laser_query.iter() {
		commands.entity(laser_en).despawn();
	}
	for (explosion_en) in explosion_query.iter() {
		commands.entity(explosion_en).despawn();
	}
}

fn restart_game(
	mut commands: Commands,
	kb: Res<Input<KeyCode>>,
	mut state: ResMut<State<GameState>>,
	mut score: ResMut<GameScore>,
	mut player_life: ResMut<PlayerLife>,
	mut enemy_count: ResMut<EnemyCount>,
) {
	if kb.just_pressed(KeyCode::Space) {
		println!("Restart game!");
		score.0 = 0;
		enemy_count.0 = 0;
		player_life.0 = PLAYER_MAX_LIFE;
		state.set(GameState::Game).expect("Failed to restart state to Game");
	}
}

fn movable_system(
	mut commands: Commands,
	win_size: Res<WinSize>,
	mut query: Query<(Entity, &Velocity, &mut Transform, &Movable)>,
) {
	for (entity, velocity, mut transform, movable) in query.iter_mut() {
		let translation = &mut transform.translation;
		translation.x += velocity.x * TIME_STEP * BASE_SPEED;
		translation.y += velocity.y * TIME_STEP * BASE_SPEED;

		if movable.auto_despawn {
			// despawn when out of screen
			const MARGIN: f32 = 200.;
			if translation.y > win_size.h / 2. + MARGIN
				|| translation.y < -win_size.h / 2. - MARGIN
				|| translation.x > win_size.w / 2. + MARGIN
				|| translation.x < -win_size.w / 2. - MARGIN
			{
				commands.entity(entity).despawn();
			}
		}
	}
}

fn player_laser_hit_enemy_system(
	mut commands: Commands,
	mut enemy_count: ResMut<EnemyCount>,
	mut score: ResMut<GameScore>,
	laser_query: Query<(Entity, &Transform, &SpriteSize), (With<Laser>, With<FromPlayer>)>,
	enemy_query: Query<(Entity, &Transform, &SpriteSize), With<Enemy>>,
) {
	let mut despawned_entities: HashSet<Entity> = HashSet::new();

	// iterate through the lasers
	for (laser_entity, laser_tf, laser_size) in laser_query.iter() {
		if despawned_entities.contains(&laser_entity) {
			continue;
		}

		let laser_scale = Vec2::from(laser_tf.scale.xy());

		// iterate through the enemies
		for (enemy_entity, enemy_tf, enemy_size) in enemy_query.iter() {
			if despawned_entities.contains(&enemy_entity)
				|| despawned_entities.contains(&laser_entity)
			{
				continue;
			}

			let enemy_scale = Vec2::from(enemy_tf.scale.xy());

			// determine if collision
			let collision = collide(
				laser_tf.translation,
				laser_size.0 * laser_scale,
				enemy_tf.translation,
				enemy_size.0 * enemy_scale,
			);

			// perform collision
			if let Some(_) = collision {
				// remove the enemy
				commands.entity(enemy_entity).despawn();
				despawned_entities.insert(enemy_entity);
				if enemy_count.0 > 0 {
					enemy_count.0 -= 1;
				}
				score.0 += 1;

				// remove the laser
				commands.entity(laser_entity).despawn();
				despawned_entities.insert(laser_entity);

				// spawn the explosionToSpawn
				commands.spawn().insert(ExplosionToSpawn(enemy_tf.translation.clone()));
			}
		}
	}
}

fn enemy_laser_hit_player_system(
	mut commands: Commands,
	mut player_state: ResMut<PlayerState>,
	mut player_life: ResMut<PlayerLife>,
	mut state: ResMut<State<GameState>>,
	time: Res<Time>,
	laser_query: Query<(Entity, &Transform, &SpriteSize), (With<Laser>, With<FromEnemy>)>,
	player_query: Query<(Entity, &Transform, &SpriteSize), With<Player>>,
) {
	if let Ok((player_entity, player_tf, player_size)) = player_query.get_single() {
		let player_scale = Vec2::from(player_tf.scale.xy());

		for (laser_entity, laser_tf, laser_size) in laser_query.iter() {
			let laser_scale = Vec2::from(laser_tf.scale.xy());

			// determine if collision
			let collision = collide(
				laser_tf.translation,
				laser_size.0 * laser_scale,
				player_tf.translation,
				player_size.0 * player_scale,
			);

			// perform the collision
			if let Some(_) = collision {
				// remove the player
				commands.entity(player_entity).despawn();
				player_state.shot(time.seconds_since_startup());
				println!("{}", player_life.0);
				if player_life.0 > 0 {
					player_life.0 -= 1;
				}

				if player_life.0 <= 0 {
					//End Game
					println!("Setting state to END");
					state.set(GameState::End).expect("Failed to change to end state");
				}

				// remove the laser
				commands.entity(laser_entity).despawn();

				// spawn the explosionToSpawn
				commands.spawn().insert(ExplosionToSpawn(player_tf.translation.clone()));

				break;
			}
		}
	}
}

fn explosion_to_spawn_system(
	mut commands: Commands,
	game_textures: Res<GameTextures>,
	query: Query<(Entity, &ExplosionToSpawn)>,
) {
	for (explosion_spawn_entity, explosion_to_spawn) in query.iter() {
		// spawn the explosion sprite
		commands
			.spawn_bundle(SpriteSheetBundle {
				texture_atlas: game_textures.explosion.clone(),
				transform: Transform {
					translation: explosion_to_spawn.0,
					..Default::default()
				},
				..Default::default()
			})
			.insert(Explosion)
			.insert(ExplosionTimer::default());

		// despawn the explosionToSpawn
		commands.entity(explosion_spawn_entity).despawn();
	}
}

fn explosion_animation_system(
	mut commands: Commands,
	time: Res<Time>,
	mut query: Query<(Entity, &mut ExplosionTimer, &mut TextureAtlasSprite), With<Explosion>>,
) {
	for (entity, mut timer, mut sprite) in query.iter_mut() {
		timer.0.tick(time.delta());
		if timer.0.finished() {
			sprite.index += 1; // move to next sprite cell
			if sprite.index >= EXPLOSION_LEN {
				commands.entity(entity).despawn()
			}
		}
	}
}
