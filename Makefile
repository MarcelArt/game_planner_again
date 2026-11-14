dummy:
	@SEED_DIR=/tmp/gp/game_planner cargo test seed_sample_game -- --ignored
	@XDG_DATA_HOME=/tmp/gp cargo run
