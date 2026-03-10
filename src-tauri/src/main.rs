use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::Json,
    routing::post,
    Router,
};
use serde::Serialize;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use uuid::Uuid;
use std::io::Write;

pub mod ai_pipeline;

#[tokio::main]
async fn main() {
    println!("Iniciando servidor backend (Image Enhancer)...");

    let cwd = std::env::current_dir().unwrap();
    
    // Resolve caminho dos modelos ONNX
    let models_dir = if cwd.join("src-tauri/models").exists() {
        cwd.join("src-tauri/models")
    } else {
        cwd.join("models")
    };
    println!("Procurando Modelos ONNX em: {:?}", models_dir);
    
    let core_models = ai_pipeline::onnx::init_models(models_dir)
        .expect("Falha ao inicializar sessões do ONNX e Models");
    
    let shared_state = Arc::new(core_models);

    // Resolve caminho do Frontend (Pasta Dist)
    let dist_dir = if cwd.join("dist").exists() {
        cwd.join("dist")
    } else {
        cwd.join("../dist")
    };
    println!("Servindo Frontend da pasta: {:?}", dist_dir);
    if !dist_dir.join("index.html").exists() {
        println!("⚠️ AVISO: 'index.html' não encontrado em {:?}. A interface pode não carregar.", dist_dir);
    }
    
    let serve_dir = ServeDir::new(dist_dir.clone())
        .fallback(ServeFile::new(dist_dir.join("index.html")));
    // Create uploads folder
    let _ = std::fs::create_dir_all("uploads");

    let app = Router::new()
        .route("/api/process", post(upload_and_process_image))
        // Rota para servir as imagens processadas (se quisermos visualizá-las)
        .nest_service("/uploads", ServeDir::new("uploads"))
        .fallback_service(serve_dir)
        .layer(CorsLayer::permissive())
        .with_state(shared_state);

    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Servidor rodando em: http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}

#[derive(Serialize)]
pub struct ApiProcessResult {
    pub success: bool,
    pub original_path: String,
    pub result_path: Option<String>,
    pub message: String,
}

async fn upload_and_process_image(
    State(state): State<Arc<ai_pipeline::onnx::CoreModels>>,
    mut multipart: Multipart,
) -> Result<Json<ApiProcessResult>, (StatusCode, String)> {
    let mut uploaded_file = None;
    let mut ext = "jpg".to_string();

    while let Some(field) = multipart.next_field().await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))? {
        let file_name = field.file_name().unwrap_or("image.jpg").to_string();
        if let Some(e) = file_name.split('.').last() {
            ext = e.to_string();
        }

        let data = field.bytes().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        
        let id = Uuid::new_v4();
        let file_path = format!("uploads/{}.{}", id, ext);
        
        // Save the file
        let mut f = std::fs::File::create(&file_path).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        f.write_all(&data).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        
        uploaded_file = Some(file_path);
        break; // Only accept one file
    }

    let file_path = uploaded_file.ok_or((StatusCode::BAD_REQUEST, "No file provided".to_string()))?;
    
    // Agora chama a pipeline. Como mudamos `process_image` para retornar a struct embutida e não usar mais tauri:
    match ai_pipeline::process_image(file_path.clone(), state).await {
        Ok(res) => {
            // we should convert the struct, or since ProcessResult is similar we can just map it here.
            Ok(Json(ApiProcessResult {
                success: res.success,
                original_path: res.original_path,
                // Make the result path accessible via URL
                result_path: res.result_path.map(|p| p.replace("\\", "/")), 
                message: res.message,
            }))
        },
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}
