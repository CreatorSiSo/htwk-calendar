use crate::prelude::*;
use axum::extract::State;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

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

pub async fn subjects(
	State(cache): State<Arc<RwLock<Cache>>>,
) -> Result<Json<Vec<Subject>>, ErrorRes> {
	if let Some((instant, ref subjects)) = cache.read().unwrap().subjects {
		if instant.elapsed() < Duration::from_secs(60 * 60 * 5 /* 5 hours */) {
			return Ok(Json(subjects.clone()));
		}
	}

	let faculties = scrape_faculties(crate::URL_FACULTIES)
		.await
		.map_err(|report| server_error("Unable to scrape faculties", report))?;

	let mut subjects: Vec<Subject> = faculties
		.into_iter()
		.flat_map(|faculty| faculty.subjects)
		.collect();

	let extensions = parse_ext_groups_file();
	for subject in &mut subjects {
		let Some(ext_groups) = extensions.get(subject.id.as_str()) else {
			continue;
		};

		subject
			.groups
			.extend(ext_groups.iter().map(|group| (*group).into()));
	}

	cache.write().unwrap().subjects = Some((Instant::now(), subjects.clone()));
	Ok(Json(subjects))
}

fn parse_ext_groups_file() -> HashMap<&'static str, Vec<&'static str>> {
	let mut subject_map = HashMap::new();

	for subject_data in include_str!("../.groups").split("\n\n") {
		let Some((id, groups)) = subject_data.split_once('\n') else {
			panic!("{subject_data}");
		};
		subject_map.insert(id, groups.lines().collect::<Vec<&str>>());
	}

	subject_map
}

async fn scrape_faculties(url: &str) -> color_eyre::Result<Vec<Faculty>> {
	let faculties_text = reqwest::get(url).await?.text().await?;
	debug!("made HTTP get request");
	let faculties = quick_xml::de::from_str::<Study>(&faculties_text)?.faculties;

	Ok(faculties)
}
