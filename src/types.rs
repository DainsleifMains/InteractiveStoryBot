// © 2024 ElementalAlchemist and the Dainsleif Mains Development Team
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use serenity::prelude::TypeMapKey;

pub struct StoryText(String);

impl StoryText {
	pub fn new(text: String) -> Self {
		Self(text)
	}

	pub fn get(&self) -> &str {
		self.0.as_str()
	}
}

impl TypeMapKey for StoryText {
	type Value = StoryText;
}
