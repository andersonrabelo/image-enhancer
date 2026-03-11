pub mod onnx;
pub mod segmentation;
pub mod models;
pub mod blending;
pub mod upscaler;

#[derive(serde::Serialize)]
pub struct ProcessResult {
    pub success: bool,
    pub original_path: String,
    pub restored_path: Option<String>,
    pub upscaled_path: Option<String>,
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

    // 1. SCUNet (Restoration/Denoising)
    let scunet_session = std::sync::Arc::clone(&state.scunet);
    let restored_image = models::process_scunet(base_image, scunet_session).await?;

    // 2. Salvar Imagem Restaurada (Intermediária)
    let path_obj = std::path::Path::new(&path);
    let base_dir = path_obj.parent().unwrap_or(std::path::Path::new(""));
    let file_stem = path_obj.file_stem().unwrap_or_default().to_string_lossy();
    let file_ext = path_obj.extension().unwrap_or_default().to_string_lossy();

    let mut restored_path = base_dir.to_path_buf();
    restored_path.push(format!("{}_restored.{}", file_stem, file_ext));
    restored_image.save(&restored_path).map_err(|e| format!("Erro ao salvar imagem restaurada: {}", e))?;

    // 3. Real-ESRGAN (Upscaling)
    let realesrgan_session = std::sync::Arc::clone(&state.realesrgan);
    let upscaled_image = upscaler::process_realesrgan(restored_image, realesrgan_session).await?;
    
    // 4. Salvar Imagem com Upscale (Final)
    let mut upscaled_path = base_dir.to_path_buf();
    upscaled_path.push(format!("{}_upscaled.{}", file_stem, file_ext));
    upscaled_image.save(&upscaled_path).map_err(|e| format!("Erro ao salvar imagem upscaled: {}", e))?;
    
    let duration = start_time.elapsed();
    let res_restored = restored_path.to_string_lossy().to_string();
    let res_upscaled = upscaled_path.to_string_lossy().to_string();

    Ok(ProcessResult {
        success: true,
        original_path: path.clone(),
        restored_path: Some(res_restored),
        upscaled_path: Some(res_upscaled),
        message: format!("Pipeline concluída com sucesso (SCUNet + Real-ESRGAN) em {:.2?}.", duration),
    })
}
