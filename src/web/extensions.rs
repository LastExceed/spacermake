use http::StatusCode;
use http::header::*;
use warp::reply::*;
use warp::*;

#[extend::ext]
pub impl<T: Reply> T {
	fn with_header(self, name: &str, value: &str) -> WithHeader<T> {
		with_header(self, name, value)
	}

	fn with_status(self, status: StatusCode) -> WithStatus<T> {
		with_status(self, status)
	}

	fn with_set_cookie(self, name: &str, value: &str) -> WithHeader<T> {
		self
		.with_header("Set-Cookie", &format!("{name}={value}"))
	}

	fn with_redirect(self, location: &str) -> WithStatus<WithHeader<T>> {
		self
		.with_header("Location", location)
		.with_status(StatusCode::SEE_OTHER)
	}

	fn with_auth(self) -> WithHeader<WithStatus<T>> {
		self
		.with_status(StatusCode::UNAUTHORIZED)
		.with_header(WWW_AUTHENTICATE.as_str(), "Basic")
	}
}