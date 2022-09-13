use crate::{
    constants::HISTORY_RECEIPTS_COUNT,
    keys::{Keys, KEYS},
    AppConfig, SharedState, SharedTranscript,
};
use axum::{
    response::{IntoResponse, Response},
    Extension, Json,
};
use axum_extra::response::ErasedJson;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use small_powers_of_tau::sdk::TranscriptJSON;

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct StatusResponse {
    lobby_size:        usize,
    num_contributions: usize,
}

impl IntoResponse for StatusResponse {
    fn into_response(self) -> Response {
        let status = StatusCode::OK;
        (status, Json(self)).into_response()
    }
}

pub async fn status(Extension(store): Extension<SharedState>) -> StatusResponse {
    let app_state = store.read().await;

    let lobby_size = app_state.lobby.len();
    let num_contributions = app_state.num_contributions;

    StatusResponse {
        lobby_size,
        num_contributions,
    }
}

pub struct CurrentStateResponse {
    state: TranscriptJSON,
}

impl IntoResponse for CurrentStateResponse {
    fn into_response(self) -> Response {
        // We use ErasedJson for the case that one wants to view the
        // transcript in the browser and it needs to be prettified
        (StatusCode::OK, ErasedJson::pretty(self.state)).into_response()
    }
}


pub(crate) async fn current_state(
    Extension(config): Extension<AppConfig>,
) -> CurrentStateResponse {
    // let app_state = transcript.read().await;
    let transcript_json = todo!(); //TranscriptJSON::from(&*app_state);
    CurrentStateResponse {
        state: transcript_json,
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtInfoResponse {
    alg:         &'static str,
    rsa_pem_key: String,
}

impl IntoResponse for JwtInfoResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, ErasedJson::pretty(self)).into_response()
    }
}

// Returns the relevant JWT information
#[allow(clippy::unused_async)] // Required for axum function signature
pub async fn jwt_info() -> JwtInfoResponse {
    let rsa_public_key_pem_as_string = KEYS.get().unwrap().decode_key_to_string();

    JwtInfoResponse {
        alg:         Keys::alg_str(),
        rsa_pem_key: rsa_public_key_pem_as_string,
    }
}
