#![feature(iterator_try_collect)]

use std::str::FromStr;
use table_extract::{
	scraper::{ElementRef, Html, Selector},
	Table,
};
use time::{Time, Weekday};

const URL: &str = "https://stundenplan.htwk-leipzig.de/ws/Berichte/Text-Listen;Studenten-Sets;name;23INB-3?template=sws_semgrp&weeks=1-65";

#[tokio::main]
async fn main() -> eyre::Result<()> {
	let res = reqwest::get(URL).await?;
	let text = res.text().await?;
	let html = Html::parse_document(&text);

	for day in html.select(&Selector::parse("p > span.labelone").unwrap()) {
		let siblings = day.parent().unwrap().next_siblings();
		let table_element = siblings.skip(1).next().unwrap();
		let table = Table::new(ElementRef::wrap(table_element).unwrap());

		for row in table.iter().skip(1) {
			let row = row.as_slice();

			let event = Event {
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
				weeks: row[0].parse()?,
				start: parse_time(&row[1])?,
				end: parse_time(&row[2])?,
			};

			println!("{event:?}")
		}
	}

	Ok(())
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
