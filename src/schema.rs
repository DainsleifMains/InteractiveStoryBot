// @generated automatically by Diesel CLI.

diesel::table! {
	user_progress (user_id) {
		user_id -> Int8,
		current_passage -> Text,
	}
}
