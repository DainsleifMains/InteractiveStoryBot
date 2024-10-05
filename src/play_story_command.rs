// © 2024 ElementalAlchemist and the Dainsleif Mains Development Team
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::types::StoryText;
use miette::{bail, ensure, IntoDiagnostic};
use serenity::builder::{CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::client::Context;
use serenity::model::application::{CommandInteraction, CommandType};
use tweep::Story;

pub fn command_definition() -> CreateCommand {
	CreateCommand::new("play_story")
		.kind(CommandType::ChatInput)
		.dm_permission(false)
		.description("Play an interactive story")
}

pub async fn command_execute(ctx: &Context, command: &CommandInteraction) -> miette::Result<()> {
	let message = {
		let data = ctx.data.read().await;
		let story_text = data.get::<StoryText>();
		let Some(story_text) = story_text else {
			bail!("Story text not found");
		};

		// Unfortunately, due to how tweep defined the Story type, we cannot pass Story values around or even hold onto
		// them through an `await`.
		let story = Story::from_string(story_text.get().to_string());
		ensure!(
			!story.has_warnings(),
			"Story has unresolved warnings: {:?}",
			story.get_warnings()
		);
		let (story, _) = story.take();
		let story = story.into_diagnostic()?;

		let Some(initial_passage_title) = story.get_start_passage_name() else {
			bail!("No initial passage defined");
		};
		CreateInteractionResponseMessage::new()
			.content(format!(
				"You're about to start a story with a first passage called {}. (Your user id: {})",
				initial_passage_title,
				command.user.id.get()
			))
			.ephemeral(true)
	};
	command
		.create_response(&ctx.http, CreateInteractionResponse::Message(message))
		.await
		.into_diagnostic()?;

	Ok(())
}
