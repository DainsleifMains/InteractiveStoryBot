// © 2024 ElementalAlchemist and the Dainsleif Mains Development Team
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::database::DatabaseConnection;
use crate::models::UserProgress;
use crate::schema::user_progress;
use crate::types::StoryText;
use diesel::prelude::*;
use miette::{bail, ensure, IntoDiagnostic};
use serenity::builder::{
	CreateActionRow, CreateButton, CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage,
	EditInteractionResponse,
};
use serenity::client::Context;
use serenity::collector::ComponentInteractionCollector;
use serenity::model::application::{CommandInteraction, CommandType, ComponentInteractionDataKind};
use serenity::model::id::UserId;
use std::time::Duration;
use tweep::{Story, TwinePassage};

const RESPONSE_TIMEOUT_SECONDS: u64 = 600;

pub fn command_definition() -> CreateCommand {
	CreateCommand::new("play_story")
		.kind(CommandType::ChatInput)
		.dm_permission(false)
		.description("Play an interactive story")
}

pub async fn command_execute(ctx: &Context, command: &CommandInteraction) -> miette::Result<()> {
	let data = ctx.data.read().await;
	let story_text = data.get::<StoryText>();
	let Some(story_text) = story_text else {
		bail!("Story text not found");
	};
	let db_connection_pool = data.get::<DatabaseConnection>();
	let Some(db_connection_pool) = db_connection_pool else {
		bail!("Failed to get database connection pool");
	};

	let executing_user_id = command.user.id;
	let db_user_id = executing_user_id.get() as i64;

	let user_progress_data = {
		let mut db_connection = db_connection_pool.get().into_diagnostic()?;

		let user_progress_data: QueryResult<Option<UserProgress>> = user_progress::table
			.find(&db_user_id)
			.first(&mut db_connection)
			.optional();
		match user_progress_data {
			Ok(data) => data,
			Err(error) => bail!("Failed to retrieve user progress from the database: {}", error),
		}
	};

	let (message, message_link_ids) = {
		// Unfortunately, due to how tweep defined the Story type, we cannot pass Story values around or even hold onto
		// them through an `await`.
		let story = Story::from_string(story_text.get().to_string());
		let (story, warnings) = story.take();
		ensure!(warnings.is_empty(), "Story has unresolved warnings: {:?}", warnings);
		let story = story.into_diagnostic()?;

		let current_passage = user_progress_data.and_then(|progress| story.passages.get(&progress.current_passage));

		match current_passage {
			Some(passage) => message_from_passage(executing_user_id, passage),
			None => {
				let Some(initial_passage) = story.get_start_passage_name() else {
					bail!("No start passage defined");
				};
				tutorial_passage(executing_user_id, initial_passage)
			}
		}
	};
	command
		.create_response(&ctx.http, CreateInteractionResponse::Message(message))
		.await
		.into_diagnostic()?;

	if message_link_ids.is_empty() {
		return Ok(());
	}

	let Some(mut interaction) = ComponentInteractionCollector::new(&ctx.shard)
		.custom_ids(message_link_ids)
		.timeout(Duration::from_secs(RESPONSE_TIMEOUT_SECONDS))
		.await
	else {
		let _ = command
			.edit_response(&ctx.http, EditInteractionResponse::new().components(Vec::new()))
			.await
			.into_diagnostic();
		return Ok(());
	};

	let _ = command
		.edit_response(&ctx.http, EditInteractionResponse::new().components(Vec::new()))
		.await
		.into_diagnostic();

	loop {
		match &interaction.data.kind {
			ComponentInteractionDataKind::Button => {
				let Some((_, selected_passage_name)) = interaction.data.custom_id.split_once('|') else {
					unreachable!();
				};
				if let Ok(mut db_connection) = db_connection_pool.get() {
					let passage_progress_state = UserProgress {
						user_id: db_user_id,
						current_passage: selected_passage_name.to_string(),
					};
					diesel::insert_into(user_progress::table)
						.values(passage_progress_state)
						.on_conflict(user_progress::user_id)
						.do_update()
						.set(user_progress::current_passage.eq(selected_passage_name))
						.execute(&mut db_connection)
						.into_diagnostic()?;
				}

				let (message, message_link_ids) = {
					let story = Story::from_string(story_text.get().to_string());
					let (story, _) = story.take();
					let story = story.into_diagnostic()?;

					let current_passage = match story.passages.get(selected_passage_name) {
						Some(passage) => passage,
						None => bail!("Next passage not defined"),
					};

					message_from_passage(executing_user_id, current_passage)
				};

				interaction
					.create_response(&ctx.http, CreateInteractionResponse::Message(message))
					.await
					.into_diagnostic()?;

				if message_link_ids.is_empty() {
					break;
				}

				let new_interaction = ComponentInteractionCollector::new(&ctx.shard)
					.custom_ids(message_link_ids)
					.timeout(Duration::from_secs(RESPONSE_TIMEOUT_SECONDS))
					.await;
				let _ = interaction
					.edit_response(&ctx.http, EditInteractionResponse::new().components(Vec::new()))
					.await
					.into_diagnostic();
				interaction = match new_interaction {
					Some(interaction) => interaction,
					None => break,
				};
			}
			_ => unreachable!(),
		}
	}

	Ok(())
}

fn format_passage(passage: &TwinePassage) -> String {
	let mut passage_text = passage.content.content.clone();

	for link_data in passage.content.get_links().iter() {
		let link_text_data = link_data.context.get_contents();
		if let Some((link_name, _)) = link_text_data.split_once("->") {
			let Some(passage_position) = passage_text.find(link_text_data) else {
				continue;
			};
			let new_link_name = format!("{}]]", link_name);
			passage_text.replace_range(
				passage_position..(passage_position + link_text_data.len()),
				&new_link_name,
			);
		}
	}

	passage_text = passage_text.replace("''", "**"); // bold
	passage_text = passage_text.replace("//", "*"); // italics

	// The default strikethrough is the same as Discord's.

	passage_text
}

fn get_passage_links(passage: &TwinePassage) -> Vec<(String, String)> {
	let links = passage.content.get_links();
	links
		.iter()
		.map(|link_data| {
			let mut link = link_data.context.get_contents();
			link = link.strip_prefix("[[").unwrap_or(link);
			link = link.strip_suffix("]]").unwrap_or(link);

			match link.split_once("->") {
				Some((name, target)) => (name.to_string(), target.to_string()),
				None => (link.to_string(), link.to_string()),
			}
		})
		.collect()
}

fn message_from_passage(user_id: UserId, passage: &TwinePassage) -> (CreateInteractionResponseMessage, Vec<String>) {
	let passage_text = format_passage(passage);
	let link_data = get_passage_links(passage);

	if link_data.is_empty() {
		(
			CreateInteractionResponseMessage::new()
				.content(passage_text)
				.ephemeral(true),
			Vec::new(),
		)
	} else {
		let mut buttons: Vec<CreateButton> = Vec::new();
		let mut custom_ids: Vec<String> = Vec::new();
		for (link_index, (link_name, link_target)) in link_data.into_iter().enumerate() {
			let button_custom_id = format!("{}-{}|{}", user_id.get(), link_index, link_target);
			buttons.push(CreateButton::new(&button_custom_id).label(link_name));
			custom_ids.push(button_custom_id);
		}
		let button_row = CreateActionRow::Buttons(buttons);

		(
			CreateInteractionResponseMessage::new()
				.content(passage_text)
				.ephemeral(true)
				.components(vec![button_row]),
			custom_ids,
		)
	}
}

fn tutorial_passage(user_id: UserId, opening_passage_title: &str) -> (CreateInteractionResponseMessage, Vec<String>) {
	let start_button_id = format!("{}-0|{}", user_id.get(), opening_passage_title);
	let buttons = vec![CreateButton::new(&start_button_id).label("Click here to start")];
	let button_row = CreateActionRow::Buttons(buttons);
	let message_content = "# Tutorial\n\nWhen playing through this story, you will see text in double-brackets. For example, it'll look like this: [[Click here to start]].\nIf you see that, a button with the same text at the bottom of the post will take you to that page.\n\nTry it now!";
	(
		CreateInteractionResponseMessage::new()
			.content(message_content)
			.components(vec![button_row]),
		vec![start_button_id],
	)
}
