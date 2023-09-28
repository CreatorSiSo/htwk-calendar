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
const URL_TEMPLATE: &str = "https://stundenplan.htwk-leipzig.de/ws/Berichte/Text-Listen;Studenten-Sets;name;{$group$}?template=sws_semgrp&weeks=1-65";

#[tokio::main]
async fn main() -> eyre::Result<()> {
	color_eyre::install()?;

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

	let faculties = scrape::faculties(URL_FACULTIES).await?;
	println!("{faculties:#?}");

	let routes = Router::new()
		.route("/events/:group", axum::routing::get(events_of_group))
		.layer(TraceLayer::new_for_http());

	let addr = SocketAddr::from(([0, 0, 0, 0], 5000));
	tracing::info!("Listening on {}", addr);
	axum::Server::bind(&addr)
		.serve(routes.into_make_service())
		.await?;

	Ok(())
}

#[axum::debug_handler]
async fn events_of_group(Path(group): Path<String>) -> Result<Json<Vec<Event>>, String> {
	let url = URL_TEMPLATE.replace("{$group$}", &group);
	let events = scrape::events(&url).await.map_err(|err| {
		format!("Unable to scrape timetable for {group}.\n\nInternal error: {err}")
	})?;

	tracing::debug!("{events:#?}");
	Ok(Json(events))
}
