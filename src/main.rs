// © 2024 ElementalAlchemist and the Dainsleif Mains Development Team
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use miette::{bail, ensure, IntoDiagnostic};
use serenity::prelude::*;
use tweep::Story;

mod config;
mod database;
mod handler;
mod play_story_command;
mod types;

use database::{DatabaseConnection, connect_db, run_embedded_migrations};
use handler::Handler;
use types::StoryText;

#[tokio::main]
async fn main() -> miette::Result<()> {
	let config = config::parse_config("config.kdl")?;
	let db_connection_pool = connect_db(&config)?;
	run_embedded_migrations(&db_connection_pool)?;

	let story_text = tokio::fs::read_to_string(&config.story_file).await.into_diagnostic()?;
	let story_str = Story::from_string(story_text.clone());
	let (story, warnings) = story_str.take();
	ensure!(
		warnings.is_empty(),
		"There are errors/warnings in the story data.\n{:?}",
		warnings
	);

	let story = match story {
		Ok(story) => story,
		Err(error) => bail!(error),
	};

	if story.get_start_passage_name().is_none() {
		bail!("No start passage defined");
	};

	let story_text = StoryText::new(story_text);

	let intents = GatewayIntents::empty();
	let client_builder = Client::builder(&config.discord_bot_token, intents)
		.event_handler(Handler)
		.type_map_insert::<StoryText>(story_text)
		.type_map_insert::<DatabaseConnection>(db_connection_pool);
	let mut client = client_builder.await.into_diagnostic()?;

	client.start().await.into_diagnostic()?;

	Ok(())
}
