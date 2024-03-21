#![feature(let_chains)]

use tonic::{transport::Server, Request, Response, Status};
use envoy_types::pb::envoy::service::auth::v3::{
    authorization_server::{Authorization, AuthorizationServer}, CheckRequest, CheckResponse,
};
use envoy_types::ext_authz::v3::{CheckRequestExt, CheckResponseExt};

#[derive(Default)]
struct MyAuthz;

#[tonic::async_trait]
impl Authorization for MyAuthz {
    async fn check(
        &self,
        request: Request<CheckRequest>,
    ) -> Result<Response<CheckResponse>, Status> {
        let request = request.into_inner();

        let client_headers = request
            .get_client_headers()
            .ok_or(Status::invalid_argument("client headers not populated by envoy"))?;

        let request_status = if let Some(h) = client_headers.get("let-me-in") && 
            h == "pls" {
            Status::ok("request is valid")
        } else  {
            Status::unauthenticated("not authorized")
        };

        Ok(Response::new(CheckResponse::with_status(request_status)))
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let addr = "0.0.0.0:50051".parse()?;
    let server = MyAuthz;

    println!("AuthorizationServer listening on {addr}");

    Server::builder()
        .add_service(AuthorizationServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}
