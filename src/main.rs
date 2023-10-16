mod events;
mod meta;
mod middleware;
mod prelude;
use prelude::*;

use axum::{http::HeaderValue, middleware::from_fn, Router};
use std::{
	net::SocketAddr,
	sync::{Arc, RwLock},
};
use tower_http::{compression::CompressionLayer, cors::CorsLayer, services, trace::TraceLayer};
use tracing::Level;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

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

	let groups_cache = Arc::new(RwLock::new(HashMap::new()));

	let routes = {
		let api_routes = Router::new()
			.route("/subjects", axum::routing::get(meta::subjects_all))
			.with_state(meta::spawn_subjects_scraper().await?)
			.route("/events/:group", axum::routing::get(events::of_group))
			.with_state(groups_cache);

		let frontend_routes = Router::new()
			.nest_service("/", services::ServeDir::new("frontend/dist"))
			.layer(from_fn(middleware::cache_control));

		Router::new()
			.nest("/", frontend_routes)
			.nest("/api", api_routes)
			.layer(
				CorsLayer::new().allow_origin([
					config.site.parse::<HeaderValue>().unwrap(),
					"https://htwk-calendar-16672e5a5a3b.herokuapp.com"
						.parse::<HeaderValue>()
						.unwrap(),
				]),
			)
			.layer(from_fn(middleware::redirect))
			.layer(CompressionLayer::new())
			.layer(TraceLayer::new_for_http())
	};

	let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
	info!("Listening on {}", addr);
	axum::Server::bind(&addr)
		.serve(routes.into_make_service())
		.await?;

	Ok(())
}
