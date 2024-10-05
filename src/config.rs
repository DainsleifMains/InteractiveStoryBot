// © 2024 ElementalAlchemist and the Dainsleif Mains Development Team
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use knuffel::Decode;
use miette::{IntoDiagnostic, Result};

pub fn parse_config(config_path: &str) -> Result<ConfigDocument> {
	let config_file_contents = std::fs::read_to_string(config_path).into_diagnostic()?;
	let config = knuffel::parse(config_path, &config_file_contents)?;
	Ok(config)
}

#[derive(Debug, Decode)]
pub struct ConfigDocument {
	#[knuffel(child, unwrap(argument))]
	pub story_file: String,
}
