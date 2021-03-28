// Copyright (C) 2020 Mangata team
// Based on Snowfork bridge implementation
use crate::types::AppId;
use sp_std::convert::TryInto;

/// Short identifer for an application.
#[derive(Copy, Clone)]
pub enum App {
	ETH,
	ERC20,
}

#[derive(Copy, Clone)]
struct Entry {
	pub app: App,
	pub id: AppId,
}

// FIXME add app ids as constants in readable format
static APP_REGISTRY: &[Entry] = &[
	Entry {
		app: App::ETH,
		id: [252, 151, 166, 25, 125, 201, 11, 239, 107, 190, 253, 103, 39, 66, 237, 117, 233, 118, 133, 83],
	},
	Entry {
		app: App::ERC20,
		id: [237, 163, 56, 228, 220, 70, 3, 132, 147, 184, 133, 50, 120, 66, 253, 62, 48, 28, 171, 57],
	}
];

/// Looks up an application in the registry identified by `app_id`.
pub fn lookup_app(app_id: AppId) -> Option<App> {
	for entry in APP_REGISTRY.iter() {
		if app_id == entry.id {
			return Some(entry.app)
		}
	}
	None
}
