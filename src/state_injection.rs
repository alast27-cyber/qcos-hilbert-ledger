use axum::{extract::Json, http::StatusCode, response::IntoResponse};

async fn handle_state_injection(
    Json(payload): Json<StateInjectionPayload>,
) -> impl IntoResponse {
    // Log or process the deserialized values
    println!(
        "[INJECTION] Client: {}, Mode: {}, Amplitude: {} + {}i",
        payload.client_id, payload.mode_index, payload.amplitude_re, payload.amplitude_im
    );

    // Forward to Hilbert Ledger core state machine...
    
    StatusCode::OK
}