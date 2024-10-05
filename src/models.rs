// © 2024 ElementalAlchemist and the Dainsleif Mains Development Team
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::schema::user_progress;
use diesel::{Insertable, Queryable};

/// An instance of user progress through the story
#[derive(Clone, Debug, Insertable, Queryable)]
#[diesel(table_name = user_progress)]
pub struct UserProgress {
	/// User ID from Discord (do a simple convert to unsigned)
	pub user_id: i64,
	/// Name of the last passage the user read/revealed
	pub current_passage: String,
}
