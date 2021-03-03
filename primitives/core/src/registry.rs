// Copyright (C) 2020 Mangata team
// Based on Snowfork bridge implementation
use crate::types::AppId;

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
		id: [
			0xdd, 0x51, 0x4b, 0xaa, 0x31, 0x7b, 0xf0, 0x95, 0xdd, 0xba, 0x2c, 0x0a, 0x84, 0x77,
			0x65, 0xfe, 0xb3, 0x89, 0xc6, 0xa0,
		],
	},
	Entry {
		app: App::ERC20,
		id: [
			0x00, 0xe3, 0x92, 0xc0, 0x47, 0x43, 0x35, 0x9e, 0x39, 0xf0, 0x0c, 0xd2, 0x68, 0xa5,
			0x39, 0x0d, 0x27, 0xef, 0x6b, 0x44,
		],
	},
];

/// Looks up an application in the registry identified by `app_id`.
pub fn lookup_app(app_id: AppId) -> Option<App> {
	for entry in APP_REGISTRY.iter() {
		if app_id == entry.id {
			return Some(entry.app);
		}
	}
	None
}
