use crate::domain::*;
use crate::pipeline::SharablePipeline;
use actix_web::{web, HttpResponse};
use anyhow::Context;
use chrono::Utc;
use std::convert::{TryFrom, TryInto};

impl TryFrom<String> for SessionDescription {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let sdp = SessionDescription::parse(value)?;
        Ok(sdp)
    }
}

#[allow(clippy::async_yields_async)]
#[tracing::instrument(
    name = "Receive an offer from a client",
    skip(form, app_state, pipeline_state)
)]
pub async fn subscribe(
    form: String,
    app_state: web::Data<SharableAppState>,
    pipeline_state: web::Data<SharablePipeline>,
) -> Result<HttpResponse, SubscribeError> {
    tracing::info!("Received SDP at time: {:?}", Utc::now());

    if form.is_empty() {
        pipeline_state.add_client().unwrap();
        
        let result = app_state
            .wait_on_whip_offer()
            .await
            .context("Failed to receive a whip offer")?;

        return Ok(HttpResponse::Created()
            .append_header((
                "Location",
                format!("http://0.0.0.0:8000/channel/{}", result.resource_id),
            ))
            .content_type("application/sdp")
            .body(result.sdp.as_ref().to_string()));
    }

    return Ok(HttpResponse::BadRequest().body("Client initialization not supported"));
}

#[allow(clippy::async_yields_async)]
#[tracing::instrument(name = "Receive an answer from a client", skip(form, app_state))]
pub async fn patch(
    form: String,
    path: web::Path<String>,
    app_state: web::Data<SharableAppState>,
) -> Result<HttpResponse, SubscribeError> {
    let sdp: SessionDescription = form.try_into().map_err(SubscribeError::ValidationError)?;
    let id = path.into_inner();

    app_state
        .save_whep_offer(sdp, Some(id))
        .await
        .context("Failed to save whep offer")?;

    return Ok(HttpResponse::NoContent().into());
}