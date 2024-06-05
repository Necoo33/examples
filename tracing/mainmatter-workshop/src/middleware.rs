use std::time::Instant;

use actix_web::HttpMessage as _;
use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    http::header::{HeaderName, HeaderValue},
};
use actix_web_lab::middleware::Next;
use tracing_actix_web::RequestId;

use crate::metric_names::*;

pub(crate) async fn request_telemetry(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> actix_web::Result<ServiceResponse<impl MessageBody>> {
    let now = Instant::now();

    metrics::gauge!(GAUGE_HTTP_CONCURRENT_REQUESTS).increment(1);

    let mut res = next.call(req).await?;

    let req_id = res.request().extensions().get::<RequestId>().copied();

    if let Some(req_id) = req_id {
        res.headers_mut().insert(
            HeaderName::from_static("request-id"),
            // this unwrap never fails, since UUIDs are valid ASCII strings
            HeaderValue::from_str(&req_id.to_string()).unwrap(),
        );
    };

    let diff = now.elapsed();
    metrics::histogram!(HISTOGRAM_HTTP_REQUEST_DURATION).record(diff);

    metrics::gauge!(GAUGE_HTTP_CONCURRENT_REQUESTS).decrement(1);

    Ok(res)
}
