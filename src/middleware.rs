use crate::CONFIG;
use axum::{
	http::Request,
	middleware::Next,
	response::{Redirect, Response},
};
use tracing::info;

/// Sets the `Cache-Control` for static assets
pub async fn cache_control<B>(request: Request<B>, next: Next<B>) -> Response {
	let mut response = next.run(request).await;
	response
		.headers_mut()
		.insert("Cache-Control", "no-cache".parse().unwrap());
	response
}

/// Redirects all requests to `CONFIG.host`
pub async fn redirect<B>(request: Request<B>, next: Next<B>) -> Result<Response, Redirect> {
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
