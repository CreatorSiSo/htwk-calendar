use std::fmt::Display;

use crate::faculties::*;
use color_eyre::eyre;
use htmlize::unescape;
use table_extract::scraper::{ElementRef, Html, Selector};
use table_extract::Table;
use time::format_description::well_known::Iso8601;
use time::{Date, PrimitiveDateTime, Time, Weekday};

#[derive(Debug, serde::Serialize)]
pub struct Event {
	pub title: String,
	pub notes: String,
	pub start: String,
	pub end: String,
	pub kind: EventKind,
	pub kind_display: String,
	pub color: String,
}

pub async fn events(url: &str) -> eyre::Result<Vec<Event>> {
	let raw_events = raw_events(url).await?;

	Ok(raw_events
		.into_iter()
		.flat_map(|event| {
			match event.weeks {
				Weeks::Single(week) => vec![week],
				Weeks::Multiple(weeks) => weeks,
				Weeks::Range { start, end } => (start..=end).collect(),
			}
			.into_iter()
			.map(move |week| {
				let year_offset = week / 52;
				let week = match week % 52 {
					0 => 52,
					week => week,
				};

				let date = Date::from_iso_week_date(
					(2023 + year_offset) as i32,
					week as u8,
					event.weekday,
				)
				.unwrap();

				Event {
					title: event.title.clone(),
					notes: event.notes.clone(),
					start: PrimitiveDateTime::new(date, event.start)
						.format(&Iso8601::DATE_TIME)
						.unwrap(),
					end: PrimitiveDateTime::new(date, event.end)
						.format(&Iso8601::DATE_TIME)
						.unwrap(),
					kind: event.kind,
					kind_display: event.kind.to_string(),
					color: match event.kind {
						EventKind::Vorlesung => "blue",
						EventKind::VorlesungPflicht => "blue",
						EventKind::VorlesungWahlpflicht => "blue",

						EventKind::Seminar => "green",
						EventKind::SeminarPflicht => "green",
						EventKind::SeminarWahlpflicht => "green",

						EventKind::Praktikum => "orange",
						EventKind::PraktikumPflicht => "orange",
						EventKind::PraktikumWahlpflicht => "orange",

						EventKind::Zusatzveranstaltung => "red",
						EventKind::FakultativeVeranstaltung => "red",
						EventKind::Tutorium => "red",
						EventKind::Pflicht => "red",
						EventKind::Wahlpflicht => "red",

						EventKind::Gebucht => "gray",
						EventKind::Sperr => "gray",
						EventKind::Unknown => "gray",
					}
					.into(),
				}
			})
		})
		.collect::<Vec<_>>())
}

#[derive(Debug, serde::Serialize)]
pub struct RawEvent {
	pub title: String,
	pub notes: String,
	pub weekday: Weekday,
	pub weeks: Weeks,
	pub start: Time,
	pub end: Time,
	pub kind: EventKind,
}

#[derive(Debug, Clone, Copy, serde_repr::Serialize_repr)]
#[repr(u8)]
pub enum EventKind {
	Vorlesung,
	VorlesungPflicht,
	VorlesungWahlpflicht,

	Seminar,
	SeminarPflicht,
	SeminarWahlpflicht,

	Praktikum,
	PraktikumPflicht,
	PraktikumWahlpflicht,

	Zusatzveranstaltung,
	FakultativeVeranstaltung,
	Tutorium,
	Pflicht,
	Wahlpflicht,

	Gebucht,
	Sperr,
	Unknown,
}

impl Display for EventKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let string = match self {
			EventKind::Vorlesung => "Vorlesung",
			EventKind::VorlesungPflicht => "Vorlesung (Pflicht)",
			EventKind::VorlesungWahlpflicht => "Vorlesung (Wahlpflicht)",
			EventKind::Seminar => "Seminar",
			EventKind::SeminarPflicht => "Seminar (Pflicht)",
			EventKind::SeminarWahlpflicht => "Seminar (Wahlpflicht)",
			EventKind::Praktikum => "Praktikum",
			EventKind::PraktikumPflicht => "Praktikum (Pflicht)",
			EventKind::PraktikumWahlpflicht => "Praktikum (Wahlpflicht)",
			EventKind::Zusatzveranstaltung => "Zusatzveranstaltung",
			EventKind::FakultativeVeranstaltung => "fakultative Veranstaltung",
			EventKind::Tutorium => "Tutorium",
			EventKind::Pflicht => "Pflicht",
			EventKind::Wahlpflicht => "Wahlpflicht",
			EventKind::Gebucht => "Gebucht",
			EventKind::Sperr => "Sperr",
			EventKind::Unknown => "Unbekannt",
		};
		f.write_str(string)
	}
}

pub async fn raw_events(url: &str) -> eyre::Result<Vec<RawEvent>> {
	let html_text = reqwest::get(url).await?.text().await?;
	let html = Html::parse_document(&html_text);

	Ok(html
		.select(&Selector::parse("p > span.labelone").unwrap())
		.into_iter()
		.map(|day| {
			let siblings = day.parent().unwrap().next_siblings();
			let table_element = siblings.skip(1).next().unwrap();
			let table = Table::new(ElementRef::wrap(table_element).unwrap());

			println!("{:?}", table.iter().nth(1).unwrap().as_slice());

			table
				.iter()
				.skip(1)
				.map(|row| -> eyre::Result<RawEvent> {
					let row = row.as_slice();

					let event = RawEvent {
						title: unescape(&row[3]).trim().into(),
						notes: unescape(&row[7]).trim().into(),
						kind: match row[4].to_lowercase().as_str() {
							"v" => EventKind::Vorlesung,
							"vp" => EventKind::VorlesungPflicht,
							"vw" => EventKind::VorlesungWahlpflicht,

							"s" => EventKind::Seminar,
							"sp" => EventKind::SeminarPflicht,
							"sw" => EventKind::SeminarWahlpflicht,

							"p" => EventKind::Praktikum,
							"pp" => EventKind::PraktikumPflicht,
							"pw" => EventKind::PraktikumWahlpflicht,

							"zv" => EventKind::Zusatzveranstaltung,
							"fak" => EventKind::FakultativeVeranstaltung,
							"tut" => EventKind::Tutorium,
							"pf" => EventKind::Pflicht,
							"wpf" => EventKind::Wahlpflicht,

							"gebucht" => EventKind::Gebucht,
							"sperr" => EventKind::Sperr,

							_ => EventKind::Unknown,
						},
						weekday: match day.text().collect::<String>().as_str() {
							"Montag" => Weekday::Monday,
							"Dienstag" => Weekday::Tuesday,
							"Mittwoch" => Weekday::Wednesday,
							"Donnerstag" => Weekday::Thursday,
							"Freitag" => Weekday::Friday,
							"Samstag" => Weekday::Saturday,
							"Sonntag" => Weekday::Sunday,
							_ => panic!(),
						},
						weeks: row[0].parse()?,
						start: parse_time(&row[1])?,
						end: parse_time(&row[2])?,
					};
					Ok(event)
				})
				.try_collect::<Vec<RawEvent>>()
		})
		.map(|result| result.unwrap())
		.flatten()
		.collect())
}

fn parse_time(string: &str) -> eyre::Result<Time> {
	let (hour, minute) = string.split_once(':').unwrap();
	Ok(Time::from_hms(hour.parse()?, minute.parse()?, 0)?)
}

#[derive(Debug, serde::Serialize)]
pub enum Weeks {
	Single(u32),
	Multiple(Vec<u32>),
	Range { start: u32, end: u32 },
}

impl std::str::FromStr for Weeks {
	type Err = std::num::ParseIntError;

	fn from_str(string: &str) -> Result<Self, Self::Err> {
		if let Some((start, end)) = string.split_once("-") {
			return Ok(Weeks::Range {
				start: start.parse()?,
				end: end.parse()?,
			});
		}

		if !string.contains(',') {
			return Ok(Weeks::Single(string.parse()?));
		}

		Ok(Weeks::Multiple(
			string
				.split(", ")
				.map(|week_str| week_str.parse())
				.try_collect()?,
		))
	}
}

pub async fn faculties(url: &str) -> eyre::Result<Vec<Faculty>> {
	let faculties_text = reqwest::get(url).await?.text().await?;
	let faculties = quick_xml::de::from_str::<Study>(&faculties_text)?.faculties;

	let extensions = vec![Faculty {
		id: "FIMN".into(),
		name: "Fakultät Informatik und Medien LFB Informatik".into(),
		subjects: vec![
			Subject {
				id: "INB".into(),
				name: "Informatik (Bachelor of Science)".into(),
				groups: vec![
					Group {
						id: "23INB-1".into(),
					},
					Group {
						id: "23INB-2".into(),
					},
					Group {
						id: "23INB-3".into(),
					},
				],
			},
			Subject {
				id: "MIB".into(),
				name: "Medieninformatik (Bachelor of Science)".into(),
				groups: vec![
					Group {
						id: "23MIB-1".into(),
					},
					Group {
						id: "23MIB-2".into(),
					},
				],
			},
		],
	}];

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
					.find(|extension_subject| *extension_subject == subject)
				else {
					continue;
				};

				subject.groups.append(&mut extension_subject.groups.clone());
			}

			faculty
		})
		.collect();

	Ok(faculties)
}
