use axum::{extract::Query, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use openrank_common::{
    merkle::{fixed::DenseMerkleTree, hash_leaf, Hash},
    parse_score_entries_from_file, JobResult,
};
use serde::{Deserialize, Serialize};
use sha3::Keccak256;
use std::{fs::File, net::SocketAddr, path::Path};
use tracing::{error, info};

/// Query parameters for the /score-proof endpoint
#[derive(Debug, Deserialize)]
pub struct ScoreProofQuery {
    /// The compute ID (hex-encoded hash of the meta job results)
    pub compute_id: String,
    /// The user ID to get the score proof for
    pub user_id: String,
}

/// Response structure containing the score inclusion proof
#[derive(Debug, Serialize)]
pub struct ScoreProofResponse {
    /// The compute ID
    pub compute_id: String,
    /// The user ID
    pub user_id: String,
    /// The user's score value
    pub score: f32,
    /// The index of the score in the scores tree
    pub score_index: usize,
    /// Merkle path for the score in the scores tree (leaf to root)
    pub scores_tree_path: Vec<Hash>,
    /// The scores tree root (commitment)
    pub scores_tree_root: Hash,
    /// The index of this job's commitment in the meta tree
    pub meta_index: usize,
    /// Merkle path for the commitment in the meta tree (leaf to root)
    pub meta_tree_path: Vec<Hash>,
    /// The meta tree root (final commitment)
    pub meta_tree_root: Hash,
}

/// Error response structure
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Server error type
#[derive(Debug)]
pub enum ServerError {
    NotFound(String),
    InternalError(String),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            ServerError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ServerError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        (status, Json(ErrorResponse { error: message })).into_response()
    }
}

/// Handler for the /score-proof endpoint
async fn score_proof_handler(
    Query(params): Query<ScoreProofQuery>,
) -> Result<Json<ScoreProofResponse>, ServerError> {
    info!(
        "Received score-proof request for compute_id: {}, user_id: {}",
        params.compute_id, params.user_id
    );

    // Load job results from local file system
    let meta_path = format!("./meta/{}", params.compute_id);
    let meta_file = File::open(&meta_path).map_err(|e| {
        error!("Failed to open meta file {}: {}", meta_path, e);
        ServerError::NotFound(format!("Compute ID not found: {}", params.compute_id))
    })?;

    let job_results: Vec<JobResult> = serde_json::from_reader(meta_file).map_err(|e| {
        error!("Failed to parse meta file: {}", e);
        ServerError::InternalError(format!("Failed to parse job results: {}", e))
    })?;

    if job_results.is_empty() {
        return Err(ServerError::NotFound("No job results found".to_string()));
    }

    // Find which job contains the user and build the trees
    let mut found_job_index: Option<usize> = None;
    let mut found_score_index: Option<usize> = None;
    let mut found_score_value: Option<f32> = None;
    let mut scores_tree: Option<DenseMerkleTree<Keccak256>> = None;

    for (job_idx, job_result) in job_results.iter().enumerate() {
        let scores_path = format!("./scores/{}.csv", job_result.scores_id);

        if !Path::new(&scores_path).exists() {
            continue;
        }

        let scores_file = File::open(&scores_path).map_err(|e| {
            error!("Failed to open scores file {}: {}", scores_path, e);
            ServerError::InternalError(format!("Failed to open scores file: {}", e))
        })?;

        let score_entries = parse_score_entries_from_file(scores_file).map_err(|e| {
            error!("Failed to parse scores file: {}", e);
            ServerError::InternalError(format!("Failed to parse scores: {}", e))
        })?;

        // Check if user exists in this job's scores
        for (score_idx, entry) in score_entries.iter().enumerate() {
            if entry.id() == &params.user_id {
                found_job_index = Some(job_idx);
                found_score_index = Some(score_idx);
                found_score_value = Some(*entry.value());

                // Build the scores merkle tree
                let score_hashes: Vec<Hash> = score_entries
                    .iter()
                    .map(|e| hash_leaf::<Keccak256>(e.value().to_be_bytes().to_vec()))
                    .collect();

                scores_tree = Some(DenseMerkleTree::<Keccak256>::new(score_hashes).map_err(
                    |e| {
                        error!("Failed to build scores tree: {}", e);
                        ServerError::InternalError(format!("Failed to build scores tree: {}", e))
                    },
                )?);

                break;
            }
        }

        if found_job_index.is_some() {
            break;
        }
    }

    let job_index = found_job_index.ok_or_else(|| {
        ServerError::NotFound(format!("User {} not found in any job", params.user_id))
    })?;
    let score_index = found_score_index.unwrap();
    let score_value = found_score_value.unwrap();
    let scores_tree = scores_tree.unwrap();

    // Generate scores tree path
    let scores_tree_path = scores_tree.generate_path(score_index).map_err(|e| {
        error!("Failed to generate scores tree path: {}", e);
        ServerError::InternalError(format!("Failed to generate scores tree path: {}", e))
    })?;

    let scores_tree_root = scores_tree.root().map_err(|e| {
        error!("Failed to get scores tree root: {}", e);
        ServerError::InternalError(format!("Failed to get scores tree root: {}", e))
    })?;

    // Build the meta tree from all job commitments
    let commitment_hashes: Vec<Hash> = job_results
        .iter()
        .map(|jr| {
            let commitment_bytes = alloy::hex::decode(&jr.commitment).unwrap_or_default();
            Hash::from_slice(&commitment_bytes)
        })
        .collect();

    let meta_tree = DenseMerkleTree::<Keccak256>::new(commitment_hashes).map_err(|e| {
        error!("Failed to build meta tree: {}", e);
        ServerError::InternalError(format!("Failed to build meta tree: {}", e))
    })?;

    // Generate meta tree path
    let meta_tree_path = meta_tree.generate_path(job_index).map_err(|e| {
        error!("Failed to generate meta tree path: {}", e);
        ServerError::InternalError(format!("Failed to generate meta tree path: {}", e))
    })?;

    let meta_tree_root = meta_tree.root().map_err(|e| {
        error!("Failed to get meta tree root: {}", e);
        ServerError::InternalError(format!("Failed to get meta tree root: {}", e))
    })?;

    let response = ScoreProofResponse {
        compute_id: params.compute_id,
        user_id: params.user_id,
        score: score_value,
        score_index,
        scores_tree_path,
        scores_tree_root,
        meta_index: job_index,
        meta_tree_path,
        meta_tree_root,
    };

    info!("Successfully generated score proof");
    Ok(Json(response))
}

/// Health check endpoint
async fn health_handler() -> &'static str {
    "OK"
}

/// Create the router with all endpoints
pub fn create_router() -> Router {
    Router::new()
        .route("/score-proof", get(score_proof_handler))
        .route("/health", get(health_handler))
}

/// Run the server on the specified address
pub async fn run_server(addr: SocketAddr) -> Result<(), std::io::Error> {
    let app = create_router();

    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
