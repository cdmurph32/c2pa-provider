//! c2pa-provider capability provider
//!
//!
use std::io::Write;
//use tracing::info;
use wasmbus_rpc::provider::prelude::*;
use wasmcloud_interface_c2pa::{C2Pa, C2PaReceiver, CalculateInput};

use c2pa::{
    assertions::{c2pa_action, Action, Actions},
    create_signer, Ingredient, Manifest, Signer, SigningAlg,
};

const SIGN_CERT: &[u8] = include_bytes!("../etc/es256_certs.pem");
const SIGN_KEY: &[u8] = include_bytes!("../etc/es256_private.key");

pub static GENERATOR: &str = concat!("CAI_Everywhere", "/", env!("CARGO_PKG_VERSION"));

// main (via provider_main) initializes the threaded tokio executor,
// listens to lattice rpcs, handles actor links,
// and returns only when it receives a shutdown message
//
fn main() -> Result<(), Box<dyn std::error::Error>> {
    provider_main(C2PaProvider::default(), Some("C2PaProvider".to_string()))?;

    eprintln!("c2pa-provider provider exiting");
    Ok(())
}

/// c2pa-provider capability provider implementation
#[derive(Default, Clone, Provider)]
#[services(C2Pa)]
struct C2PaProvider {}

/// use default implementations of provider message handlers
impl ProviderDispatch for C2PaProvider {}
impl ProviderHandler for C2PaProvider {}

/// Embed a transcode claim.
#[async_trait]
impl C2Pa for C2PaProvider {
    /// accepts a number and calculates its factorial
    async fn embed_transcode_claim(
        &self,
        _ctx: &Context,
        req: &CalculateInput,
    ) -> RpcResult<Vec<u8>> {
        eprintln!("HERE");
        let transcode_action = Action::new(c2pa_action::TRANSCODED)
            .set_parameter("origin", &req.origin_url)
            .map_err(|err| sign_error(err.to_string()))?;
        let actions = Actions::new().add_action(transcode_action);
        let signer = get_signer()?;
        let mut manifest = Manifest::new(GENERATOR);
        let mut parent = get_parent_ingredient(&req.origin, &req.origin_mime_type)?;
        parent.set_title(&req.origin_filename);
        manifest
            .set_parent(parent)
            .map_err(|err| sign_error(err.to_string()))?;
        manifest
            .set_title(&req.origin_filename)
            .set_thumbnail(&req.thumb_mime_type, req.thumb.clone())
            .add_assertion(&actions)
            .map_err(|err| sign_error(err.to_string()))?;
        manifest
            .embed_from_memory(&req.output_mime_type, &req.render, &*signer)
            .map_err(|err| sign_error(err.to_string()))
    }
}

fn sign_error<S: Into<String>>(reason: S) -> RpcError {
    RpcError::HostError(reason.into())
}

fn get_signer() -> Result<Box<dyn Signer>, RpcError> {
    let tsa_url = String::from("http://timestamp.digicert.com");
    create_signer::from_keys(SIGN_CERT, SIGN_KEY, SigningAlg::Es256, Some(tsa_url))
        .map_err(|err| sign_error(err.to_string()))
}

// TODO: Get rid of this once we have Ingredient::from_bytes()
fn get_parent_ingredient(origin: &[u8], mime_type: &str) -> Result<Ingredient, RpcError> {
    let suffix = if mime_type.eq("image/png") {
        ".png"
    } else {
        ".jpg"
    };
    let mut temp = tempfile::Builder::new()
        .suffix(suffix)
        .tempfile()
        .map_err(|err| sign_error(err.to_string()))?;
    let to_write: &mut [u8] = &mut origin.to_owned();
    temp.write_all(to_write)
        .map_err(|err| sign_error(err.to_string()))?;
    Ingredient::from_file(temp.path()).map_err(|err| sign_error(err.to_string()))
}
