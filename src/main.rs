// © 2024 ElementalAlchemist and the Dainsleif Mains Development Team
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use miette::{bail, ensure, IntoDiagnostic};
use tweep::Story;

mod config;

fn main() -> miette::Result<()> {
	let config = config::parse_config("config.kdl")?;

	let story_str = Story::from_string(std::fs::read_to_string(&config.story_file).into_diagnostic()?);
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

	let Some(initial_passage_title) = story.get_start_passage_name() else {
		bail!("No start passage defined");
	};
	println!("Start: {}", initial_passage_title);

	for (passage_title, passage_data) in story.passages.iter() {
		println!("Passage: {}", passage_title);
		println!();
		println!("{}", passage_data.content.content);
		println!();
		println!("Links:");
		let links = passage_data.content.get_links();
		for link_data in links.iter() {
			let mut link = link_data.context.get_contents();
			link = link.strip_prefix("[[").unwrap_or(link);
			link = link.strip_suffix("]]").unwrap_or(link);
			if let Some((name, destination)) = link.split_once("->") {
				println!("Name: {}; Target: {}", name, destination);
			} else {
				println!("Name & Target: {}", link);
			}
		}
		println!();
	}

	Ok(())
}
