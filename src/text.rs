use super::components::{
	Enemy, Explosion, ExplosionTimer, ExplosionToSpawn, FromEnemy, FromPlayer, Laser, Movable,
	Player, PlayerLifeComp, Score, SpriteSize, Velocity,
};
use crate::WinSize;
use crate::components::RestartText;
use crate::GameScore;
use crate::GameState;
use crate::PlayerLife;
use bevy::prelude::*;

pub struct TextPlugin;

impl Plugin for TextPlugin {
	fn build(&self, app: &mut App) {
		app.add_startup_system_to_stage(StartupStage::PostStartup, text_init_system)
			.add_system(text_update_system)
			.add_system(text_update_life_system)
			.add_system_set(
				SystemSet::on_enter(GameState::End).with_system(text_spawn_restart_game),
			)
			.add_system_set(SystemSet::on_exit(GameState::End).with_system(clear_text));
	}
}

fn text_init_system(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	score: Res<GameScore>,
	player_life: Res<PlayerLife>,
) {
	commands
		.spawn_bundle(
			TextBundle::from_section(
				format!("Score: {}", score.0.to_string()),
				TextStyle {
					font_size: 20.0,
					color: Color::WHITE,
					font: asset_server.load(r"fonts\IBMPlexSans-Regular.ttf"),
				},
			)
			.with_style(Style {
				position_type: PositionType::Absolute,
				position: UiRect {
					top: Val::Px(5.0),
					left: Val::Px(5.0),
					..default()
				},
				..default()
			}),
		)
		.insert(Score);

	commands
		.spawn_bundle(
			TextBundle::from_section(
				format!("Life: {}", player_life.0.to_string()),
				TextStyle {
					font_size: 20.0,
					color: Color::WHITE,
					font: asset_server.load(r"fonts\IBMPlexSans-Regular.ttf"),
				},
			)
			.with_style(Style {
				position_type: PositionType::Absolute,
				position: UiRect {
					top: Val::Px(25.0),
					left: Val::Px(5.0),
					..default()
				},
				..default()
			}),
		)
		.insert(PlayerLifeComp);
}

fn text_update_system(score: Res<GameScore>, mut query: Query<&mut Text, With<Score>>) {
	let mut ini_score = query.single_mut();
	ini_score.sections[0].value = format!("Score: {}", score.0.to_string());
}

fn text_update_life_system(
	player_life: Res<PlayerLife>,
	mut query: Query<&mut Text, With<PlayerLifeComp>>,
) {
	let mut ini_player_life = query.single_mut();
	ini_player_life.sections[0].value = format!("Life: {}", player_life.0.to_string());
}

fn text_spawn_restart_game(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	score: Res<GameScore>,
    win_size: Res<WinSize>,
	mut query: Query<&mut Text, With<Score>>,
) {
	commands
		.spawn_bundle(
			TextBundle::from_section(
				"Game Over! \n Press SpaceBar to Restart",
				TextStyle {
					font_size: 40.0,
					color: Color::WHITE,
					font: asset_server.load(r"fonts\IBMPlexSans-Regular.ttf"),
				},
			)
			.with_text_alignment(TextAlignment::TOP_CENTER)
			.with_style(Style {
				position_type: PositionType::Absolute,
				position: UiRect {
					bottom: Val::Px(win_size.h / 2.),
					right: Val::Px(win_size.w / 2. - 200.0),
					..default()
				},
				..default()
			}),
		)
		.insert(RestartText);
}

fn clear_text(
	mut commands: Commands,
	mut query: Query<(Entity, &mut Text), With<RestartText>>,
) {
	for (t_entity, _) in query.iter() {
		commands.entity(t_entity).despawn();
	}
}
