use crate::{events::Event, meta::Subject};
use once_cell::sync::OnceCell;
use std::{fmt::Display, time::Instant};

pub use axum::{http::StatusCode, Json};
pub use std::{
	collections::HashMap,
	sync::{Arc, RwLock},
};
pub use tracing::{debug, error, info};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
	pub port: u16,
	pub formatting: Formatting,
	pub site: String,
}

#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Formatting {
	Compact,
	Pretty,
}

pub static CONFIG: OnceCell<Config> = OnceCell::new();

// TODO Automatically switch between summer and winter (ss/ws) semester
pub const URL_FACULTIES: &str =
	"https://stundenplan.htwk-leipzig.de/stundenplan/xml/public/semgrp_ws.xml";
pub const URL_EVENTS: fn(&str) -> String = |group| {
	format!("https://stundenplan.htwk-leipzig.de/ws/Berichte/Text-Listen;Studenten-Sets;name;{group}?template=sws_semgrp&weeks=1-100")
};

#[derive(Debug)]
pub struct Cache {
	pub subjects: Option<(Instant, Vec<Subject>)>,
	pub group_events: HashMap<String, (Instant, Vec<Event>)>,
}

pub type ErrorRes = (StatusCode, String);

pub fn bad_request<M: Into<String>, E: Display>(msg: M, err: E) -> ErrorRes {
	(StatusCode::BAD_REQUEST, format!("{}: {}", msg.into(), err))
}

pub fn server_error<M: Into<String>, E: Display>(msg: M, err: E) -> ErrorRes {
	(
		StatusCode::INTERNAL_SERVER_ERROR,
		format!("{}: {}", msg.into(), err),
	)
}
