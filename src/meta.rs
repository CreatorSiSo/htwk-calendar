use crate::Cache;
use axum::{extract::State, Json};
use std::{
	sync::{Arc, RwLock},
	time::{Duration, Instant},
};
use tracing::debug;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Study {
	#[serde(default, rename(deserialize = "$value"))]
	pub faculties: Vec<Faculty>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Faculty {
	#[serde(rename(deserialize = "@id"))]
	pub id: String,
	#[serde(rename(deserialize = "@name"))]
	pub name: String,
	#[serde(default, rename(deserialize = "$value"))]
	pub subjects: Vec<Subject>,
}

impl PartialEq for Faculty {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Subject {
	#[serde(rename(deserialize = "@id"))]
	pub id: String,
	#[serde(rename(deserialize = "@name"))]
	pub name: String,
	#[serde(default, rename(deserialize = "$value"))]
	pub groups: Vec<Group>,
}

impl PartialEq for Subject {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Group {
	#[serde(rename(deserialize = "@id"))]
	pub id: String,
}

impl From<&'static str> for Group {
	fn from(value: &'static str) -> Self {
		Self { id: value.into() }
	}
}

pub async fn subjects(state: State<Arc<RwLock<Cache>>>) -> Result<Json<Vec<Subject>>, String> {
	let faculties = cached_faculties(state).await?.0;
	let mut subjects: Vec<Subject> = faculties
		.into_iter()
		.flat_map(|faculty| faculty.subjects)
		.collect();

	struct Ext {
		id: &'static str,
		groups: &'static [&'static str],
	}
	let extensions = [
		Ext {
			id: "INB",
			groups: &["23INB-1", "23INB-2", "23INB-3"],
		},
		Ext {
			id: "MIB",
			groups: &["23MIB-1", "23MIB-2"],
		},
		Ext {
			id: "BIB",
			groups: &[
				"23BIB-1a", "23BIB-1b", "23BIB-2a", "23BIB-2b", "23BIB-3a", "23BIB-3b", "23BIB-4a",
				"23BIB-4b",
			],
		},
		Ext {
			id: "SMB",
			groups: &["23SMB"],
		},
		Ext {
			id: "STB",
			groups: &["23STB"],
		},
		Ext {
			id: "MBB",
			groups: &["23MBB-1", "23MBB-2"],
		},
		Ext {
			id: "SBB",
			groups: &["23SBB-1", "23SBB-2"],
		},
		Ext {
			id: "IMB",
			groups: &["23IMB"],
		},
		Ext {
			id: "ARB",
			groups: &["23ARB-1", "23ARB-2", "23ARB-3", "23ARB-4"],
		},
		Ext {
			id: "BWB",
			groups: &["23BWB-1", "23BWB-2"],
		},
		Ext {
			id: "SGB",
			groups: &["23SGB"],
		},
		Ext {
			id: "BIK",
			groups: &["23BIKa", "23BIKb"],
		},
	];

	for subject in &mut subjects {
		let Some(ext) = extensions.iter().find(|ext| ext.id == subject.id) else {
			continue;
		};

		subject
			.groups
			.extend(ext.groups.iter().map(|group_id| (*group_id).into()));
	}

	Ok(Json(subjects))
}

pub async fn cached_faculties(
	State(cache): State<Arc<RwLock<Cache>>>,
) -> Result<Json<Vec<Faculty>>, String> {
	if let Some((instant, faculties)) = &cache.read().unwrap().faculties {
		if instant.elapsed() < Duration::from_secs(60 * 30 /* 30 minutes */) {
			return Ok(Json(faculties.clone()));
		}
	}

	let faculties = scrape_faculties(crate::URL_FACULTIES)
		.await
		.map_err(|err| format!("Unable to scrape faculties.\n\nInternal error: {err}"))?;

	cache.write().unwrap().faculties = Some((Instant::now(), faculties.clone()));
	Ok(Json(faculties))
}

async fn scrape_faculties(url: &str) -> color_eyre::Result<Vec<Faculty>> {
	let faculties_text = reqwest::get(url).await?.text().await?;
	debug!("made HTTP get request");
	let faculties = quick_xml::de::from_str::<Study>(&faculties_text)?.faculties;

	Ok(faculties)
}
