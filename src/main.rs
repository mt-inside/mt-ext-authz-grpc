#![feature(let_chains)]

use envoy_types::{
    ext_authz::v3::{CheckRequestExt, CheckResponseExt},
    pb::envoy::service::auth::v3::{
        authorization_server::{Authorization, AuthorizationServer},
        CheckRequest, CheckResponse,
    },
};
use tonic::{transport::Server, Request, Response, Status};
use tracing::*;
use tracing_subscriber::{filter, prelude::*};

pub static PKG: &str = env!("CARGO_PKG_NAME");
pub static NAME: &str = env!("CARGO_BIN_NAME"); // clap only has a macro for crate name
pub static VERSION: &str = clap::crate_version!();

#[derive(Default)]
struct MyAuthz;

#[tonic::async_trait]
impl Authorization for MyAuthz {
    async fn check(&self, request: Request<CheckRequest>) -> Result<Response<CheckResponse>, Status> {
        let request = request.into_inner();

        println!("{:#?}", request);

        let client_headers = request.get_client_headers().ok_or(Status::invalid_argument("client headers not populated by envoy"))?;

        let authz_decision = if let Some(h) = client_headers.get("x-let-me-in")
            && h == "pls"
        {
            Status::ok("Welcome")
        } else {
            Status::unauthenticated("Not authorized")
        };

        info!(?authz_decision, "Decided");

        Ok(Response::new(CheckResponse::with_status(authz_decision)))
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::registry()
        .with(
            filter::Targets::new()
                .with_default(Level::INFO)
                .with_target(PKG, Level::INFO) // the library package
                .with_target(NAME, Level::INFO), // this binary package
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    let addr = "0.0.0.0:50051".parse()?;
    let server = MyAuthz;

    info!(%addr, "AuthorizationServer listening");

    Server::builder().add_service(AuthorizationServer::new(server)).serve(addr).await?;

    Ok(())
}
