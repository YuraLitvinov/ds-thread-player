//-------------------------------------------------------------------------------
// Copyright 2025
//  Yurii Litvinov <litvinov.yura@gmail.com>
//
//    This program is free software: you can redistribute it and/or modify
//    it under the terms of the GNU General Public License as published by
//    the Free Software Foundation, either version 3 of the License, or
//    (at your option) any later version.
//
//    This program is distributed in the hope that it will be useful,
//    but WITHOUT ANY WARRANTY; without even the implied warranty of
//    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
//    General Public License <gnu.org/licenses/gpl.html> for more details.
//-------------------------------------------------------------------------------
#[cfg(test)]
mod tests;
mod db;
mod discord;
mod ds_functions;
mod error;
mod playback;
mod yaml;
use dotenv;
use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::runtime::Tokio;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessor;
use tracing_subscriber::filter::{LevelFilter, Targets, filter_fn};
use tracing_subscriber::{
    Layer, fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt
};
use crate::discord::read_msg;
use crate::discord::{CtxMsg, start_client};
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::{
    Mutex,
    mpsc::{Receiver, Sender, channel},
};
use tracing::{Level, event};



pub static BROKER: Lazy<(Sender<Arc<CtxMsg>>, Mutex<Receiver<Arc<CtxMsg>>>)> = Lazy::new(|| {
    let (send, receive) = channel::<std::sync::Arc<CtxMsg>>(1024);
    (send, Mutex::new(receive))
});

#[tokio::main]
async fn main() {

    if let Err(e) = dotenv::dotenv() {
        event!(Level::WARN, "{e} .env probably doesn't exist. Trying to read from variables.")
    }

    setup_tracing();

    let client = crate::db::init_db().await;
    if let Err(ref e) = client {
        event!(Level::WARN, "{e}")
    }

    let client = Arc::new(client.ok());
    tokio::spawn(async move {
        loop {
            if let Err(e) = read_msg(&client).await {
                event!(Level::ERROR, "{e}")
            }
        }
    });
    
    start_client().await;
}

fn setup_tracing() -> SdkTracerProvider {
    // Initialize OTLP exporter using gRPC (Tonic)
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()
        .expect("Failed to create OTLP exporter");

    let resource = Resource::builder()
        .with_service_name("ds-thread-player")
        .build();

    // Since BatchSpanProcessor and BatchSpanProcessorAsyncRuntime are not compatible with each other
    // we just create TracerProvider with different span processors
    let tracing_provider = SdkTracerProvider::builder()
        //.with_span_processor(BatchSpanProcessor::builder(exporter).build())
        .with_span_processor(BatchSpanProcessor::builder(exporter, Tokio).build())
        .with_resource(resource)
        .build();

    let targets_with_level =
        |targets: &[&'static str], level: LevelFilter| -> Vec<(&str, LevelFilter)> {
            // let default_log_targets: Vec<(String, LevelFilter)> =
            targets.iter().map(|t| ((*t), level)).collect()
        };

    tracing_subscriber::registry()
        // Telemetry filtering
        .with(
            tracing_opentelemetry::OpenTelemetryLayer::new(
                tracing_provider.tracer("ds-thread-player"),
            )
            .with_level(true)
            .with_filter(filter_fn(|_| true)),
        )
        // Logs filtering
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_level(true)
                .with_span_events(FmtSpan::CLOSE)
                .with_filter(match std::env::var("RUST_LOG") {
                    Ok(val) => match val.parse::<Targets>() {
                        // env var parse OK
                        Ok(log_targets_from_env) => log_targets_from_env,
                        Err(err) => {
                            eprintln!("Failed to parse RUST_LOG: {err:?}");
                            Targets::default().with_default(LevelFilter::DEBUG)
                        }
                    },
                    // No var set: use default log level INFO
                    _ => Targets::default()
                        .with_targets(targets_with_level(
                            // disable following targets:
                            &[
                                    "tower_sessions", 
                                    "tower_sessions_core",
                                    "tower_http", 
                                    "serenity::gateway::shard", 
                                    "serenity::gateway::bridge::shard_runner",
                                 ],
                            LevelFilter::OFF,
                        ))
                        .with_default(LevelFilter::INFO),
                }),
        )
        .init();

    tracing_provider
}
