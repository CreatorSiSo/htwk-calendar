use std::{
	sync::{Arc, RwLock},
	time::{Duration, Instant},
};

use axum::{extract::State, Json};
use tracing::debug;

use crate::Cache;

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

#[axum::debug_handler]
pub async fn all(State(cache): State<Arc<RwLock<Cache>>>) -> Result<Json<Vec<Faculty>>, String> {
	if let Some((instant, faculties)) = &cache.read().unwrap().faculties {
		if instant.elapsed() < Duration::from_secs(60 * 30 /* 30 minutes */) {
			return Ok(Json(faculties.clone()));
		}
	}

	let faculties = scrape(crate::URL_FACULTIES)
		.await
		.map_err(|err| format!("Unable to scrape faculties.\n\nInternal error: {err}"))?;

	let extensions = vec![
		Faculty {
			id: "FIMN".into(),
			name: "Fakultät Informatik und Medien LFB Informatik".into(),
			subjects: vec![
				Subject {
					id: "INB".into(),
					name: "Informatik (Bachelor of Science)".into(),
					groups: vec!["23INB-1".into(), "23INB-2".into(), "23INB-3".into()],
				},
				Subject {
					id: "MIB".into(),
					name: "Medieninformatik (Bachelor of Science)".into(),
					groups: vec!["23MIB-1".into(), "23MIB-2".into()],
				},
			],
		},
		Faculty {
			id: "FB".into(),
			name: "Fakultät Bauwesen".into(),
			subjects: vec![Subject {
				id: "BIB".into(),
				name: "Bauingenieurwesen (Bachelor of Engineering)".into(),
				groups: vec![
					"23BIB-1a".into(),
					"23BIB-1b".into(),
					"23BIB-2a".into(),
					"23BIB-2b".into(),
					"23BIB-3a".into(),
					"23BIB-3b".into(),
					"23BIB-3a".into(),
					"23BIB-3b".into(),
				],
			}],
		},
		Faculty {
			id: "FWW".into(),
			name: "".into(),
			subjects: vec![
				Subject {
					id: "SMB".into(),
					name: "Wirtschaftsingenieurwesen Maschinenbau (Bachelor of Engineering)".into(),
					groups: vec!["23SMB".into()],
				},
				Subject {
					id: "STB".into(),
					name: "Wirtschaftsingenieurwesen Elektrotechnik (Bachelor of Engineering)"
						.into(),
					groups: vec!["23STB".into()],
				},
			],
		},
		Faculty {
			id: "FME".into(),
			name: "Fakultät Ingenieurwissenschaften LFB Maschinenbau und Energietechnik".into(),
			subjects: vec![Subject {
				id: "MBB".into(),
				name: "Maschinenbau (Bachelor of Engineering)".into(),
				groups: vec!["23MBB-1".into(), "23MBB-2".into()],
			}],
		},
	];

	let faculties: Vec<_> = faculties
		.into_iter()
		.map(|mut faculty| {
			let Some(extension) = extensions
				.iter()
				.find(|extension| &extension.id == &faculty.id)
			else {
				return faculty;
			};

			for subject in &mut faculty.subjects {
				let Some(extension_subject) = extension
					.subjects
					.iter()
					.find(|extension_subject| extension_subject.id == subject.id)
				else {
					continue;
				};

				subject.groups.append(&mut extension_subject.groups.clone());
			}

			faculty
		})
		.collect();

	cache.write().unwrap().faculties = Some((Instant::now(), faculties.clone()));
	Ok(Json(faculties))
}

async fn scrape(url: &str) -> color_eyre::Result<Vec<Faculty>> {
	let faculties_text = reqwest::get(url).await?.text().await?;
	debug!("made HTTP get request");
	let faculties = quick_xml::de::from_str::<Study>(&faculties_text)?.faculties;

	Ok(faculties)
}

// async fn filter_groups(faculties: Vec<Faculty>) -> Vec<Group> {
// 	let groups = tokio_stream::iter(faculties.iter().flat_map(|Faculty { subjects, .. }| {
// 		subjects
// 			.into_iter()
// 			.flat_map(|Subject { groups, .. }| groups.clone())
// 	}))
// 	.filter_map(|group| async move {
// 		let res = reqwest::get(crate::URL_TEMPLATE.replace("{$group$}", &group.id))
// 			.await
// 			.unwrap();

// 		match dbg!(res.status()) {
// 			StatusCode::OK => Some(group),
// 			StatusCode::NOT_FOUND | StatusCode::FORBIDDEN | StatusCode::BAD_REQUEST => None,
// 			_ => todo!(),
// 		}
// 	});

// 	tokio_stream::StreamExt::chunks_timeout(groups, 3, Duration::from_secs(20))
// 		.collect::<Vec<_>>()
// 		.await
// 		.into_iter()
// 		.flatten()
// 		.collect()
// }
