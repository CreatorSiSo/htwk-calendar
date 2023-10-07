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

struct Ext {
	id: &'static str,
	groups: Vec<&'static str>,
}

pub async fn subjects(state: State<Arc<RwLock<Cache>>>) -> Result<Json<Vec<Subject>>, String> {
	let faculties = cached_faculties(state).await?.0;
	let mut subjects: Vec<Subject> = faculties
		.into_iter()
		.flat_map(|faculty| faculty.subjects)
		.collect();

	let extensions = parse_ext_groups_file();

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

fn parse_ext_groups_file() -> Vec<Ext> {
	include_str!("../.groups")
		.split("\n\n")
		.map(|subject_data| {
			let Some((id, groups)) = subject_data.split_once("\n") else {
				panic!("{subject_data}");
			};
			Ext {
				id,
				groups: groups.lines().collect(),
			}
		})
		.collect()
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
