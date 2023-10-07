#![feature(iterator_try_collect)]

use axum::{
	extract::{Path, Query, State},
	http::{HeaderValue, Request, StatusCode},
	middleware::{self, Next},
	response::{Redirect, Response},
	Json, Router,
};
use faculties::{Faculty, Subject};
use once_cell::sync::OnceCell;
use std::{
	collections::HashMap,
	net::SocketAddr,
	sync::{Arc, RwLock},
	time::{Duration, Instant},
};
use time::{format_description::well_known::Iso8601, PrimitiveDateTime};
use tower_http::trace::TraceLayer;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, services};
use tracing::{debug, info, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

mod faculties;
mod scrape;
use scrape::Event;

#[derive(Debug, Clone, serde::Deserialize)]
struct Config {
	port: u16,
	formatting: Formatting,
	site: String,
}

static CONFIG: OnceCell<Config> = OnceCell::new();

#[derive(Debug, Clone, Copy, serde::Deserialize)]
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
	CONFIG.set(config.clone()).unwrap();

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
		.route("/subjects", axum::routing::get(subjects))
		.route("/events/:group", axum::routing::get(events_of_group))
		.with_state(shared_cache);

	let routes = Router::new()
		.nest_service("/", services::ServeDir::new("frontend/dist"))
		.nest("/api", api_routes)
		.layer(
			CorsLayer::new().allow_origin([
				config.site.parse::<HeaderValue>().unwrap(),
				"https://htwk-calendar-16672e5a5a3b.herokuapp.com"
					.parse::<HeaderValue>()
					.unwrap(),
			]),
		)
		.layer(middleware::from_fn(redirect))
		.layer(CompressionLayer::new())
		.layer(TraceLayer::new_for_http());

	let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
	info!("Listening on {}", addr);
	axum::Server::bind(&addr)
		.serve(routes.into_make_service())
		.await?;

	Ok(())
}

/// Redirects all requests to `CONFIG.host`
async fn redirect<B>(request: Request<B>, next: Next<B>) -> Result<Response, Redirect> {
	if let Some(req_host) = request.headers().get("host") {
		let expected_site = CONFIG.get().unwrap().site.clone();
		let req_host = req_host.to_str().unwrap();

		if !expected_site.contains(req_host) {
			let redirect_url = expected_site + &request.uri().to_string();
			info!("Redirecting from host {} to {}", req_host, redirect_url);
			return Err(Redirect::permanent(&redirect_url));
		}
	}

	Ok(next.run(request).await)
}

#[axum::debug_handler]
async fn subjects(state: State<Arc<RwLock<Cache>>>) -> Result<Json<Vec<Subject>>, String> {
	let faculties = faculties::all(state).await?.0;
	Ok(Json(
		faculties
			.into_iter()
			.flat_map(|faculty| faculty.subjects)
			.collect(),
	))
}

#[derive(Debug, serde::Deserialize)]
struct TimeRange {
	start: Option<String>,
	end: Option<String>,
}

#[axum::debug_handler]
async fn events_of_group(
	Path(group): Path<String>,
	Query(time_range): Query<TimeRange>,
	State(cache): State<Arc<RwLock<Cache>>>,
) -> Result<Json<Vec<Event>>, (StatusCode, String)> {
	let start = match time_range.start {
		Some(ref start) => Some(PrimitiveDateTime::parse(start, &Iso8601::DEFAULT).map_err(
			|err| {
				(
					StatusCode::BAD_REQUEST,
					format!("Invalid format for 'start' query parameter: {err}"),
				)
			},
		)?),
		None => None,
	};
	let end = match time_range.end {
		Some(ref end) => Some(
			PrimitiveDateTime::parse(end, &Iso8601::DEFAULT).map_err(|err| {
				(
					StatusCode::BAD_REQUEST,
					format!("Invalid format for 'end' query parameter: {err}"),
				)
			})?,
		),
		None => None,
	};

	let events = 'inner: {
		if let Some((instant, events)) = cache.read().unwrap().group_events.get(&group) {
			if instant.elapsed() < Duration::from_secs(60 * 5 /* 5 minutes */) {
				break 'inner events.clone();
			}
		}

		let url = URL_TEMPLATE.replace("{$group$}", &group);
		let events = scrape::events(&url).await.map_err(|err| {
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

// TODO add caching or get rid of RawEvent
// #[axum::debug_handler]
// async fn raw_events_of_group(Path(group): Path<String>) -> Result<Json<Vec<Event>>, String> {
// 	let url = URL_TEMPLATE.replace("{$group$}", &group);
// 	let raw_events = scrape::events(&url).await.map_err(|err| {
// 		format!("Unable to scrape timetable for {group}.\n\nInternal error: {err}")
// 	})?;

// 	Ok(Json(raw_events))
// }
