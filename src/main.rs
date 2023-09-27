#![feature(iterator_try_collect)]

use std::str::FromStr;
use table_extract::{
	scraper::{ElementRef, Html, Selector},
	Table,
};
use time::{Time, Weekday};

mod faculties;
use faculties::{Faculty, Group, Study};

use crate::faculties::Subject;

const URL_FACULTIES: &str =
	"https://stundenplan.htwk-leipzig.de/stundenplan/xml/public/semgrp_ss.xml";
const URL_TEMPLATE: &str = "https://stundenplan.htwk-leipzig.de/ws/Berichte/Text-Listen;Studenten-Sets;name;{$group$}?template=sws_semgrp&weeks=1-65";

#[tokio::main]
async fn main() -> eyre::Result<()> {
	let faculties = get_faculties().await?;
	println!("{faculties:#?}");

	let subject = "23INB-3";
	let url = URL_TEMPLATE.replace("{$group$}", subject);

	let events = scrape_events(&url).await?;
	println!("{events:#?}");

	Ok(())
}

async fn scrape_events(url: &str) -> eyre::Result<Vec<Event>> {
	let html_text = reqwest::get(url).await?.text().await?;
	let html = Html::parse_document(&html_text);

	Ok(html
		.select(&Selector::parse("p > span.labelone").unwrap())
		.into_iter()
		.flat_map(|day| {
			let siblings = day.parent().unwrap().next_siblings();
			let table_element = siblings.skip(1).next().unwrap();
			let table = Table::new(ElementRef::wrap(table_element).unwrap());

			table
				.iter()
				.skip(1)
				.map(|row| {
					let row = row.as_slice();

					Event {
						name: row[3].clone(),
						notes: row[7].clone(),
						week_day: match day.text().collect::<String>().as_str() {
							"Montag" => Weekday::Monday,
							"Dienstag" => Weekday::Tuesday,
							"Mittwoch" => Weekday::Wednesday,
							"Donnerstag" => Weekday::Thursday,
							"Freitag" => Weekday::Friday,
							"Samstag" => Weekday::Saturday,
							"Sonntag" => Weekday::Sunday,
							_ => panic!(),
						},
						weeks: row[0].parse().unwrap(),
						start: parse_time(&row[1]).unwrap(),
						end: parse_time(&row[2]).unwrap(),
					}
				})
				.collect::<Vec<Event>>()
		})
		.collect())
}

async fn get_faculties() -> eyre::Result<Vec<Faculty>> {
	let faculties_text = reqwest::get(URL_FACULTIES).await?.text().await?;
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

#[derive(Debug)]
struct Event {
	name: String,
	notes: String,
	week_day: Weekday,
	weeks: Weeks,
	start: Time,
	end: Time,
}

fn parse_time(string: &str) -> eyre::Result<Time> {
	let (hour, minute) = string.split_once(':').unwrap();
	Ok(Time::from_hms(hour.parse()?, minute.parse()?, 0)?)
}

#[derive(Debug)]
enum Weeks {
	Single(u32),
	Multiple(Vec<u32>),
	Range { start: u32, end: u32 },
}

impl FromStr for Weeks {
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
