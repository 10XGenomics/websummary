use actix_web::{body::BoxBody, http::StatusCode, HttpResponse, HttpResponseBuilder, Responder};
use serde::Serialize;

use crate::{HtmlTemplate, SinglePageHtml};

impl<P: HtmlTemplate + Serialize> Responder for SinglePageHtml<P> {
    type Body = BoxBody;

    fn respond_to(self, _: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        let mut buffer = Vec::new();
        match self.generate_html(&mut buffer) {
            Ok(_) => {
                tracing::info!("Serving {}", bytesize::ByteSize(buffer.len() as u64));
                HttpResponseBuilder::new(StatusCode::OK)
                    .content_type("text/html; charset=utf-8")
                    .body(buffer)
            }
            Err(err) => {
                tracing::error!("Failed to generate html due to {:?}", err);
                HttpResponse::from_error(actix_web::error::ErrorInternalServerError(
                    err.to_string(),
                ))
            }
        }
    }
}
