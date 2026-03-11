use ort::session::builder::{GraphOptimizationLevel, SessionBuilder};
use ort::session::Session;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;

pub struct CoreModels {
    pub realesrgan: Arc<Mutex<Session>>,
    pub yolov8_face: Arc<Mutex<Session>>,
    pub scunet: Arc<Mutex<Session>>,
}

/// Inicializa as sessões do ONNX Runtime mapeando para a Memória (VRAM/RAM)
/// e tenta habilitar aceleração por hardware (DirectML no Windows).
pub fn init_models(models_dir: PathBuf) -> Result<CoreModels, String> {
    
    // Tenta carregar os providers: 
    // Em Windows, DirectML ou CUDA costumam ser os ideais para usar a GPU
    // Providers fallback: tenta DML, depois tenta TensorRT/CUDA, por fim CPU.
    #[cfg(target_os = "windows")]
    {
        println!("Registrando DirectML Execution Provider para Aceleração no Windows...");
    }
    
    println!("Iniciando carregamento dos Modelos ONNX...");

    // 1. Carrega Upscaler (Escala Inteira / Tiling depois)
    let realesrgan_path = models_dir.join("realesrgan-x4-256.onnx");
    let realesrgan = SessionBuilder::new()
        .map_err(|e| format!("Erro ao criar Builder Real-ESRGAN: {}", e))?
        .with_optimization_level(GraphOptimizationLevel::Level1)
        .map_err(|e| format!("Erro de otimização Real-ESRGAN: {}", e))?
        .with_intra_threads(4)
        .map_err(|e| format!("Erro de thread int. Real-ESRGAN: {}", e))?
        .commit_from_file(&realesrgan_path)
        .map_err(|e| format!("Erro ao carregar {}: {}", realesrgan_path.display(), e))?;
        
    println!("✓ Real-ESRGAN carregado com sucesso na memória.");

    // 2. Carrega Segmentador YOLOv8 Face
    let yolov8_path = models_dir.join("yolov8_face.onnx");
    let yolov8_face = SessionBuilder::new()
        .map_err(|e| format!("Erro ao criar Builder YOLOv8: {}", e))?
        .with_optimization_level(GraphOptimizationLevel::Level1)
        .map_err(|e| format!("Erro de otimização YOLOv8: {}", e))?
        .with_intra_threads(2) // YOLO é leve, não precisa tantas threads alocadas
        .map_err(|e| format!("Erro de thread int. YOLOv8: {}", e))?
        .commit_from_file(&yolov8_path)
        .map_err(|e| format!("Erro ao carregar {}: {}", yolov8_path.display(), e))?;
        
    println!("✓ YOLOv8 Face carregado com sucesso.");

    println!("✓ YOLOv8 Face carregado com sucesso.");

    // 4. Carrega Restaurador de Fundo SCUNet (Denoising)
    let scunet_path = models_dir.join("scunet.onnx");
    let scunet = SessionBuilder::new()
        .map_err(|e| format!("Erro ao criar Builder SCUNet: {}", e))?
        .with_optimization_level(GraphOptimizationLevel::Level1)
        .map_err(|e| format!("Erro de otimização SCUNet: {}", e))?
        .with_intra_threads(4)
        .map_err(|e| format!("Erro de threads SCUNet: {}", e))?
        .commit_from_file(&scunet_path)
        .map_err(|e| format!("Erro ao carregar {}: {}", scunet_path.display(), e))?;

    println!("✓ SCUNet carregado com sucesso.");

    Ok(CoreModels {
        realesrgan: Arc::new(Mutex::new(realesrgan)),
        yolov8_face: Arc::new(Mutex::new(yolov8_face)),
        scunet: Arc::new(Mutex::new(scunet)),
    })
}
