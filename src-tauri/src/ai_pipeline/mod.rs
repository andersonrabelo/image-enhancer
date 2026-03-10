pub mod onnx;
pub mod segmentation;
pub mod models;
pub mod blending;
pub mod upscaler;

#[derive(serde::Serialize)]
pub struct ProcessResult {
    pub success: bool,
    pub original_path: String,
    pub result_path: Option<String>,
    pub message: String,
}

pub async fn process_image(
    path: String,
    state: std::sync::Arc<onnx::CoreModels>
) -> Result<ProcessResult, String> {
    println!("Iniciando processamento da imagem: {}", path);
    let start_time = std::time::Instant::now();
    
    // 0. Carregar a Imagem Base
    let base_image = image::open(&path).map_err(|e| format!("Erro ao abrir imagem: {}", e))?;

    // 1. Segmentation
    let yolov8_session = std::sync::Arc::clone(&state.yolov8_face);
    let coords = segmentation::detect_faces_and_features(&path, yolov8_session).await
        .map_err(|e| format!("Falha na segmentação: {}", e))?;

    let codeformer_session = std::sync::Arc::clone(&state.codeformer);
    let scunet_session = std::sync::Arc::clone(&state.scunet);

    let mut face_jobs = vec![];
    let mut success_crops = 0;

    for (i, bbox) in coords.iter().enumerate() {
        if let Some((cropped, mask)) = segmentation::crop_and_mask(&base_image, bbox) {
            println!("Criado recorte {}, dimensões: {}x{}", i, cropped.width(), cropped.height());
            success_crops += 1;
            
            // 2.A Models (Paralelo: CodeFormer para Rostos)
            if bbox.class == "face" {
                let bbox_clone = bbox.clone();
                let cf_session = std::sync::Arc::clone(&codeformer_session);
                
                let handle = tokio::spawn(async move {
                    let (restored, mask) = models::process_codeformer(cropped, mask, cf_session).await?;
                    // Retornamos os resultados com a bbox pertencente para a Fase 3
                    Ok::<(image::DynamicImage, image::DynamicImage, segmentation::BoundingBox), String>((restored, mask, bbox_clone))
                });
                face_jobs.push(handle);
            }
        }
    }
    
    // 2.B Models (Paralelo: SCUNet para Imagem Base)
    let body_job = tokio::spawn(async move {
         models::process_scunet(base_image, scunet_session).await
    });

    println!("Aguardando Sidecars completarem processamento paralelo...");
    
    // 3. Recomposição (The Great Stitching)
    // O Body agora é a nova imagem de fundo melhorada pelo SCUNet.
    let mut final_image = body_job.await.map_err(|e| format!("Tokio join error: {}", e))??;

    let mut processed_faces = vec![];
    for job in face_jobs {
        let (restored, mask, bbox) = job.await.map_err(|e| format!("Tokio join error: {}", e))??;
        
        // 3.A Blending de Face sob Body
        final_image = blending::stitch_face_seamless(final_image, &restored, &mask, &bbox)?;
        processed_faces.push(restored);
    }
    
    // Extraímos o ponteiro Arc da sessão global Real-ESRGAN
    let realesrgan_session = std::sync::Arc::clone(&state.realesrgan);

    // 4. Upscaler (Real-ESRGAN Global via ONNX)
    let upscaled_image = upscaler::process_realesrgan(final_image, realesrgan_session).await?;
    
    // 5. Output Writer (Salvar no disco)
    let path_obj = std::path::Path::new(&path);
    let base_dir = path_obj.parent().unwrap_or(std::path::Path::new(""));
    let mut out_path = base_dir.to_path_buf();
    
    let file_stem = path_obj.file_stem().unwrap_or_default().to_string_lossy();
    let file_ext = path_obj.extension().unwrap_or_default().to_string_lossy();
    out_path.push(format!("{}_enhanced.{}", file_stem, file_ext));

    upscaled_image.save(&out_path).map_err(|e| format!("Erro ao salvar resultado: {}", e))?;
    
    let duration = start_time.elapsed();
    let final_res_path = out_path.to_string_lossy().to_string();

    Ok(ProcessResult {
        success: true,
        original_path: path.clone(),
        result_path: Some(final_res_path),
        message: format!("Processados {} rostos com Blending Seamless sobre 1 corpo base com sucesso em {:.2?}.", processed_faces.len(), duration),
    })
}
