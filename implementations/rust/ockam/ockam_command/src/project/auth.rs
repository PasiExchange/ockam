use clap::Args;
use std::path::PathBuf;

use anyhow::anyhow;
use ockam::Context;
use ockam_api::cloud::enroll::auth0::{Auth0Token, Auth0TokenProvider, AuthenticateAuth0Token};
use ockam_api::cloud::CloudRequestWrapper;
use ockam_core::api::{Request, RequestBuilder, Status};
use ockam_multiaddr::MultiAddr;
use tracing::info;

use crate::enroll::{Auth0Provider, Auth0Service, OktaAuth0Data};
use crate::node::util::delete_embedded_node;
use crate::util::api::CloudOpts;
use crate::util::{node_rpc, Rpc};
use crate::{help, CommandGlobalOpts};

/// An authorised enroller can add members to a project.
#[derive(Clone, Debug, Args)]
#[command(hide = help::hide())]
pub struct AuthCommand {
    /// Path to file containing project information
    #[arg(long = "project", id = "project", value_name = "PROJECT_CONFIG")]
    project: PathBuf,

    #[command(flatten)]
    cloud_opts: CloudOpts,
}

impl AuthCommand {
    pub fn run(self, opts: CommandGlobalOpts) {
        node_rpc(run_impl, (opts, self));
    }
}

async fn run_impl(
    ctx: Context,
    (opts, cmd): (CommandGlobalOpts, AuthCommand),
) -> crate::Result<()> {
    let auth0 = Auth0Service::new(Auth0Provider::Okta(okta_data(&cmd.project)?));
    let token = auth0.token().await?;
    let mut rpc = Rpc::embedded(&ctx, &opts).await?;
    rpc.request(req(cmd.cloud_opts.route(), token)).await?;
    let (res, dec) = rpc.check_response()?;
    let res = if res.status() == Some(Status::Ok) {
        info!("Enrolled successfully");
        Ok(())
    } else if res.status() == Some(Status::BadRequest) {
        info!("Already enrolled");
        Ok(())
    } else {
        eprintln!("{}", rpc.parse_err_msg(res, dec));
        Err(anyhow!("Failed to enroll").into())
    };
    delete_embedded_node(&opts.config, rpc.node_name()).await;
    res
}

/// Extract okta data from the project config
fn okta_data(project: &PathBuf) -> crate::Result<OktaAuth0Data> {
    let s = std::fs::read_to_string(project)?;
    Ok(serde_json::from_str(&s)?)
}

fn req(
    cloud_route: MultiAddr,
    token: Auth0Token,
) -> RequestBuilder<CloudRequestWrapper<AuthenticateAuth0Token>> {
    let token = AuthenticateAuth0Token::new(token);
    Request::post("v0/enroll/okta").body(CloudRequestWrapper::new(token, &cloud_route))
}
