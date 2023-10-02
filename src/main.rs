#![feature(iterator_try_collect, iterator_try_reduce)]

use axum::{
	extract::{Path, State},
	Json, Router,
};
use faculties::Faculty;
use std::{
	collections::HashMap,
	net::SocketAddr,
	sync::{Arc, RwLock},
	time::{Duration, Instant},
};
use tower_http::services;
use tower_http::trace::TraceLayer;
use tracing::{debug, info, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

mod faculties;
mod scrape;
use scrape::Event;

#[derive(Debug, serde::Deserialize)]
struct Config {
	port: u16,
	formatting: Formatting,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum Formatting {
	Compact,
	Pretty,
}

const URL_FACULTIES: &str =
	"https://stundenplan.htwk-leipzig.de/stundenplan/xml/public/semgrp_ss.xml";
const URL_TEMPLATE: &str = "https://stundenplan.htwk-leipzig.de/ws/Berichte/Text-Listen;Studenten-Sets;name;{$group$}?template=sws_semgrp&weeks=1-100";

#[derive(Debug)]
pub struct Cache {
	pub faculties: Option<(Instant, Vec<Faculty>)>,
	pub group_events: HashMap<String, (Instant, Vec<Event>)>,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
	color_eyre::install()?;

	match dotenvy::dotenv() {
		Ok(path) => debug!("Loaded env variables from: {path:?}"),
		Err(err) => debug!("When searching for .env file: {err}"),
	}
	let config: Config = envy::from_env()?;

	{
		let filter = tracing_subscriber::filter::Targets::new()
			.with_targets([
				("htwk_calendar", Level::DEBUG),
				("tower_http::trace::make_span", Level::DEBUG),
				("tower_http::trace::on_response", Level::TRACE),
				("tower_http::trace::on_request", Level::TRACE),
			])
			.with_default(Level::INFO);

		let registry = tracing_subscriber::registry().with(filter);
		match config.formatting {
			Formatting::Compact => registry
				.with(fmt::layer().with_file(false).without_time())
				.init(),
			Formatting::Pretty => registry.with(fmt::layer().pretty().with_file(false)).init(),
		}
	}

	let shared_cache = Arc::new(RwLock::new(Cache {
		faculties: None,
		group_events: HashMap::new(),
	}));

	let api_routes = Router::new()
		.route("/faculties", axum::routing::get(faculties::all))
		.route("/events/:group", axum::routing::get(events_of_group))
		// .route(
		// 	"/raw_events/:group",
		// 	axum::routing::get(raw_events_of_group),
		// )
		.with_state(shared_cache);

	let routes = Router::new()
		.nest_service("/", services::ServeDir::new("frontend/dist"))
		.nest("/api", api_routes)
		.layer(TraceLayer::new_for_http());

	let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
	info!("Listening on {}", addr);
	axum::Server::bind(&addr)
		.serve(routes.into_make_service())
		.await?;

	Ok(())
}

#[axum::debug_handler]
// TODO iso date range params
async fn events_of_group(
	Path(group): Path<String>,
	State(cache): State<Arc<RwLock<Cache>>>,
) -> Result<Json<Vec<Event>>, String> {
	if let Some((instant, events)) = cache.read().unwrap().group_events.get(&group) {
		if instant.elapsed() < Duration::from_secs(60 * 5 /* 5 minutes */) {
			return Ok(Json(events.clone()));
		}
	}

	let url = URL_TEMPLATE.replace("{$group$}", &group);
	let events = scrape::events(&url).await.map_err(|err| {
		format!("Unable to scrape timetable for {group}.\n\nInternal error: {err}")
	})?;

	cache
		.write()
		.unwrap()
		.group_events
		.insert(group, (Instant::now(), events.clone()));
	Ok(Json(events))
}

// TODO add caching or get rid of RawEvent
// #[axum::debug_handler]
// async fn raw_events_of_group(Path(group): Path<String>) -> Result<Json<Vec<Event>>, String> {
// 	let url = URL_TEMPLATE.replace("{$group$}", &group);
// 	let raw_events = scrape::events(&url).await.map_err(|err| {
// 		format!("Unable to scrape timetable for {group}.\n\nInternal error: {err}")
// 	})?;

// 	Ok(Json(raw_events))
// }
