-- © 2024 ElementalAlchemist and the Dainsleif Mains Development Team
--
-- This Source Code Form is subject to the terms of the Mozilla Public
-- License, v. 2.0. If a copy of the MPL was not distributed with this
-- file, You can obtain one at https://mozilla.org/MPL/2.0/.

CREATE TABLE user_progress (
	user_id BIGINT PRIMARY KEY,
	current_passage TEXT NOT NULL
);