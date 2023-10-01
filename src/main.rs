#![feature(iterator_try_collect, iterator_try_reduce)]

use axum::{extract::Path, Json, Router};
use color_eyre::eyre;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod faculties;
mod scrape;
use scrape::Event;

const URL_FACULTIES: &str =
	"https://stundenplan.htwk-leipzig.de/stundenplan/xml/public/semgrp_ss.xml";
const URL_TEMPLATE: &str = "https://stundenplan.htwk-leipzig.de/ws/Berichte/Text-Listen;Studenten-Sets;name;{$group$}?template=sws_semgrp&weeks=1-100";

#[tokio::main]
async fn main() -> eyre::Result<()> {
	color_eyre::install()?;

	// TODO cache events
	// let events_cache: HashMap<String, Vec<Event>> = HashMap::new();

	{
		let tracing_layer = tracing_subscriber::fmt::layer();

		let filter = tracing_subscriber::filter::Targets::new()
			.with_target("tower_http::trace::make_span", Level::DEBUG)
			.with_target("tower_http::trace::on_response", Level::TRACE)
			.with_target("tower_http::trace::on_request", Level::TRACE)
			.with_default(Level::INFO);

		tracing_subscriber::registry()
			.with(tracing_layer)
			.with(filter)
			.init();
	}

	let routes = Router::new()
		.route("/faculties", axum::routing::get(faculties::all))
		.route("/events/:group", axum::routing::get(events_of_group))
		.route(
			"/raw_events/:group",
			axum::routing::get(raw_events_of_group),
		)
		.layer(TraceLayer::new_for_http());

	let addr = SocketAddr::from(([0, 0, 0, 0], 5000));
	tracing::info!("Listening on {}", addr);
	axum::Server::bind(&addr)
		.serve(routes.into_make_service())
		.await?;

	Ok(())
}

#[axum::debug_handler]
// TODO iso date range params
async fn events_of_group(Path(group): Path<String>) -> Result<Json<Vec<Event>>, String> {
	let url = URL_TEMPLATE.replace("{$group$}", &group);
	let events = scrape::events(&url).await.map_err(|err| {
		format!("Unable to scrape timetable for {group}.\n\nInternal error: {err}")
	})?;

	Ok(Json(events))
}

#[axum::debug_handler]
async fn raw_events_of_group(Path(group): Path<String>) -> Result<Json<Vec<Event>>, String> {
	let url = URL_TEMPLATE.replace("{$group$}", &group);
	let raw_events = scrape::events(&url).await.map_err(|err| {
		format!("Unable to scrape timetable for {group}.\n\nInternal error: {err}")
	})?;

	Ok(Json(raw_events))
}