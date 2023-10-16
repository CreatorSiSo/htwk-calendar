use crate::prelude::{self, *};
use axum::extract::State;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

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

pub async fn subjects_all(
	State((cache, sender)): State<(
		Arc<RwLock<Option<(Instant, Vec<Subject>)>>>,
		mpsc::Sender<()>,
	)>,
) -> Result<Json<Vec<Subject>>, ErrorRes> {
	sender.send(()).await.unwrap();

	if let Some((_, ref subjects)) = *cache.read().unwrap() {
		Ok(Json(subjects.clone()))
	} else {
		Err(server_error("Server not finished scraping subjects", ""))
	}
}

// fn parse_ext_groups_file() -> HashMap<&'static str, Vec<&'static str>> {
// 	let mut subject_map = HashMap::new();

// 	for subject_data in include_str!("../.groups").split("\n\n") {
// 		let Some((id, groups)) = subject_data.split_once('\n') else {
// 			panic!("{subject_data}");
// 		};
// 		subject_map.insert(id, groups.lines().collect::<Vec<&str>>());
// 	}

// 	subject_map
// }

pub async fn scrape_faculties(url: &str) -> color_eyre::Result<Vec<Faculty>> {
	let faculties_text = reqwest::get(url).await?.text().await?;
	debug!("made HTTP get request");
	let faculties = quick_xml::de::from_str::<Study>(&faculties_text)?.faculties;

	Ok(faculties)
}

type SubjectsCache = Arc<RwLock<Option<(Instant, Vec<Subject>)>>>;
pub async fn spawn_subjects_scraper() -> color_eyre::Result<(SubjectsCache, mpsc::Sender<()>)> {
	let (sender, mut receiver) = mpsc::channel::<()>(1);
	sender.send(()).await?;
	let cache = Arc::new(RwLock::new(None::<(Instant, Vec<Subject>)>));
	let result = (Arc::clone(&cache), sender);

	tokio::spawn(async move {
		loop {
			// Waiting for requests
			let Some(_) = receiver.recv().await else {
				panic!("Channel was closed")
			};

			if let Some((instant, _)) = *cache.read().unwrap() {
				if instant.elapsed() < Duration::from_secs(1 /* 5 minutes */) {
					// Cache is still valid start waiting for requests again
					continue;
				}
			}

			let faculties = match scrape_faculties(prelude::URL_FACULTIES).await {
				Ok(faculties) => faculties,
				Err(err) => {
					error!("{err}");
					continue;
				}
			};

			let subjects = faculties
				.into_iter()
				.flat_map(|faculty| faculty.subjects)
				.collect();

			*cache.write().unwrap() = Some((Instant::now(), subjects));

			info!("updated subjects cache");
		}
	});

	Ok(result)
}
