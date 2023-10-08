use crate::prelude::*;
use axum::extract::{Path, Query, State};
use htmlize::unescape;
use std::{
	fmt::Display,
	time::{Duration, Instant},
};
use table_extract::{
	scraper::{ElementRef, Html, Selector},
	Table,
};
use time::{format_description::well_known::Iso8601, Date, PrimitiveDateTime, Time, Weekday};

#[derive(Debug, Clone, serde::Serialize)]
pub struct Event {
	pub title: String,
	pub notes: String,
	pub kind: EventKind,
	pub kind_display: String,
	pub rooms: Vec<String>,
	pub staff: Vec<String>,
	pub color: String,
	#[serde(serialize_with = "serialize_date_time")]
	pub start: PrimitiveDateTime,
	#[serde(serialize_with = "serialize_date_time")]
	pub end: PrimitiveDateTime,
}

#[derive(Debug, serde::Deserialize)]
pub struct TimeRange {
	start: Option<String>,
	end: Option<String>,
}

pub async fn of_group(
	Path(group): Path<String>,
	Query(time_range): Query<TimeRange>,
	State(cache): State<Arc<RwLock<Cache>>>,
) -> Result<Json<Vec<Event>>, ErrorRes> {
	let start = match time_range.start {
		Some(ref start) => Some(
			PrimitiveDateTime::parse(start, &Iso8601::DEFAULT)
				.map_err(|err| bad_request("Invalid format for 'start' query parameter", err))?,
		),
		None => None,
	};
	let end = match time_range.end {
		Some(ref end) => Some(
			PrimitiveDateTime::parse(end, &Iso8601::DEFAULT)
				.map_err(|err| bad_request("Invalid format for 'end' query parameter", err))?,
		),
		None => None,
	};

	let events = 'cached: {
		if let Some((instant, events)) = cache.read().unwrap().group_events.get(&group) {
			if instant.elapsed() < Duration::from_secs(60 * 60 /* 1 hour */) {
				break 'cached events.clone();
			}
		}

		let url = URL_EVENTS(&group);
		let events = events(&url).await.map_err(|err| {
			(
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Unable to scrape timetable for '{group}': {err}"),
			)
		})?;

		cache
			.write()
			.unwrap()
			.group_events
			.insert(group, (Instant::now(), events.clone()));

		events
	};

	let events: Vec<_> = events
		.into_iter()
		.filter(|event| match (start, end) {
			(None, None) => true,
			(None, Some(end)) => event.end <= end,
			(Some(start), None) => event.start >= start,
			(Some(start), Some(end)) => event.end >= start && event.start <= end,
		})
		.collect();

	Ok(Json(events))
}

pub async fn events(url: &str) -> color_eyre::Result<Vec<Event>> {
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
					kind: event.kind,
					kind_display: event.kind.to_string(),
					rooms: event.rooms.clone(),
					staff: event.staff.clone(),
					start: PrimitiveDateTime::new(date, event.start),
					end: PrimitiveDateTime::new(date, event.end),
					color: event.kind.color().into(),
				}
			})
		})
		.collect())
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RawEvent {
	pub title: String,
	pub notes: String,
	pub kind: EventKind,
	pub rooms: Vec<String>,
	pub staff: Vec<String>,
	pub weekday: Weekday,
	pub weeks: Weeks,
	pub start: Time,
	pub end: Time,
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

impl EventKind {
	fn color(self) -> &'static str {
		match self {
			EventKind::Vorlesung => "#275BB6",
			EventKind::VorlesungPflicht => "#275BB6",
			EventKind::VorlesungWahlpflicht => "#275BB6",

			EventKind::Seminar => "#00787D",
			EventKind::SeminarPflicht => "#00787D",
			EventKind::SeminarWahlpflicht => "#00787D",

			EventKind::Praktikum => "#FF8863",
			EventKind::PraktikumPflicht => "#FF8863", // 008800
			EventKind::PraktikumWahlpflicht => "#FF8863",

			EventKind::Zusatzveranstaltung => "#E7004F",
			EventKind::FakultativeVeranstaltung => "#E7004F",
			EventKind::Tutorium => "#E7004F",

			EventKind::Pflicht => "#FD0049",
			EventKind::Wahlpflicht => "#FD0049",

			EventKind::Gebucht => "#575757",
			EventKind::Sperr => "#575757",
			EventKind::Unknown => "#575757",
		}
	}
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

pub async fn raw_events(url: &str) -> color_eyre::Result<Vec<RawEvent>> {
	let html_text = reqwest::get(url).await?.text().await?;
	debug!("made HTTP get request");
	let html = Html::parse_document(&html_text);

	Ok(html
		.select(&Selector::parse("p > span.labelone").unwrap())
		.flat_map(|day| {
			let mut siblings = day.parent().unwrap().next_siblings();
			let table_element = siblings.nth(1).unwrap();
			let table = Table::new(ElementRef::wrap(table_element).unwrap());

			table
				.iter()
				.skip(1)
				.map(|row| -> color_eyre::Result<RawEvent> {
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
						rooms: unescape(&row[6])
							.split_whitespace()
							.map(Into::into)
							.collect(),
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
						staff: unescape(&row[5])
							.trim()
							.split('|')
							.flat_map(|str| {
								if str.is_empty() {
									None
								} else {
									Some(str.into())
								}
							})
							.collect(),
						weeks: row[0].parse()?,
						start: parse_time(&row[1])?,
						end: parse_time(&row[2])?,
					};
					Ok(event)
				})
				.map(|t| t.unwrap())
				.collect::<Vec<RawEvent>>()
		})
		.collect())
}

fn parse_time(string: &str) -> color_eyre::Result<Time> {
	let (hour, minute) = string.split_once(':').unwrap();
	Ok(Time::from_hms(hour.parse()?, minute.parse()?, 0)?)
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum Weeks {
	Single(u32),
	Multiple(Vec<u32>),
	Range { start: u32, end: u32 },
}

impl std::str::FromStr for Weeks {
	type Err = std::num::ParseIntError;

	fn from_str(string: &str) -> Result<Self, Self::Err> {
		if string.contains(',') {
			return Ok(Weeks::Multiple(
				string
					.split(", ")
					.flat_map(|week_str| match week_str.parse::<Weeks>().unwrap() {
						Weeks::Single(week) => vec![week],
						Weeks::Multiple(weeks) => weeks,
						Weeks::Range { start, end } => (start..=end).collect(),
					})
					.collect(),
			));
		}

		if let Some((start, end)) = string.split_once('-') {
			return Ok(Weeks::Range {
				start: start.parse().map_err(|_| end).unwrap(),
				end: end.parse().map_err(|_| end).unwrap(),
			});
		}

		Ok(Weeks::Single(string.parse().unwrap()))
	}
}

use serde::ser::Error as _;
use serde::{Serialize, Serializer};
fn serialize_date_time<S: Serializer>(
	datetime: &PrimitiveDateTime,
	serializer: S,
) -> color_eyre::Result<S::Ok, S::Error> {
	datetime
		.format(&Iso8601::DATE_TIME)
		.map_err(S::Error::custom)?
		.serialize(serializer)
}
