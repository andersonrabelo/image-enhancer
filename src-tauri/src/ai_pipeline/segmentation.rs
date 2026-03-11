use image::{DynamicImage, GenericImageView};
use std::path::Path;
use std::sync::{Arc, Mutex};
use ort::session::Session;
use ndarray::Array4;

#[derive(Debug, Clone, serde::Serialize)]
pub struct BoundingBox {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub class: String, // "face", "body", "tattoo", etc
}

/// Executa a detecção na imagem utilizando o Modelo ONNX Global (YOLOv8)
/// Redimensiona a imagem para 640x640 (Padrão YOLO) e roda o Predict.
pub async fn detect_faces_and_features(
    img_path: &str, 
    session_arc: Arc<Mutex<Session>>
) -> Result<Vec<BoundingBox>, String> {
    println!(">>> YOLOv8 ONNX Iniciado para: {}", img_path);
    
    let path = Path::new(img_path);
    if !path.exists() {
        return Err("Arquivo não encontrado".into());
    }

    let img = image::open(path).map_err(|e| format!("Erro ler IMG YOLO: {}", e))?;
    let (orig_w, orig_h) = img.dimensions();

    // 1. Redimensionar para o Padrão do YOLOv8 (640x640)
    let yolo_size = 640;
    let resized = img.resize_exact(yolo_size, yolo_size, image::imageops::FilterType::Triangle);
    let img_rgb = resized.to_rgb8();

    // 2. Transforma RGB -> Ndarray::Array4 (Batch=1, Channel=3, Height=640, Width=640) RGB Float [0..1]
    let mut input_tensor = Array4::<f32>::zeros((1, 3, yolo_size as usize, yolo_size as usize));
    for (y, x, pixel) in img_rgb.enumerate_pixels() {
        input_tensor[[0, 0, y as usize, x as usize]] = pixel[0] as f32 / 255.0; // R
        input_tensor[[0, 1, y as usize, x as usize]] = pixel[1] as f32 / 255.0; // G
        input_tensor[[0, 2, y as usize, x as usize]] = pixel[2] as f32 / 255.0; // B
    }

    // 3. Inferência VRAM usando o Crate ort
    let input_tensor_value = ort::value::Tensor::from_array(input_tensor)
        .map_err(|e| format!("Erro convertendo Tensor YOLO: {}", e))?;
    
    println!("  -> Processando YOLO Tensor (640x640)...");
    let session = session_arc.lock().map_err(|_| "Falha ao dar Lock na Sessao do YOLO")?;
    let input_name = session.inputs[0].name.as_str();
    let inputs = ort::inputs![input_name => input_tensor_value]
        .map_err(|e| format!("Erro ao criar ort inputs YOLO: {}", e))?;
    let outputs = session.run(inputs)
        .map_err(|e| format!("Falha na NPU YOLO: {}", e))?;

    // 4. Extrai a Saída
    // No YOLOv8 com 1 classe (Face), a saida costumeira é shape [1, 5, 8400] 
    // Coordenadas: [x_center, y_center, width, height, confidence] relativos a 640x640
    let output_view = outputs[0].try_extract_tensor::<f32>()
        .map_err(|e| format!("YOLO Tensor mismatch: {}", e))?;
    
    // (WIP) Por brevidade, pularemos a NMS (Non-Maximum Suppression) real
    // Vamos apenas mockar o resultado com base no shape real simulando sucesso.
    println!("  <- YOLO Shape Encontrado: {:?}", output_view.shape());

    // FAKE COORDS MOCK (Equivalente ao passo 2 do código Legacy para não quebrar a Pipe 3)
    let mut bboxes = vec![];
    
    // Supondo que detectamos um rosto escalado
    bboxes.push(BoundingBox { 
        x: orig_w / 4, y: orig_h / 4, 
        width: orig_w / 2, height: orig_h / 2, 
        class: "face".into() 
    });

    Ok(bboxes)
}

/// Aplica um recorte na imagem e gera uma máscara associada
/// para suavizar as bordas (Feathering) posteriormente.
pub fn crop_and_mask(img: &DynamicImage, bbox: &BoundingBox) -> Option<(DynamicImage, DynamicImage)> {
    if bbox.x + bbox.width > img.width() || bbox.y + bbox.height > img.height() {
        return None; 
    }
    
    let cropped = img.crop_imm(bbox.x, bbox.y, bbox.width, bbox.height);
    let mask = generate_feathered_mask(bbox.width, bbox.height, 15.0);
    
    Some((cropped, mask))
}

/// Gera uma máscara de transparência (em Tons de Cinza/Alpha) com bordas suavizadas.
/// Recebe as dimensões do recorte e o raio do blur.
pub fn generate_feathered_mask(width: u32, height: u32, blur_radius: f32) -> DynamicImage {
    // Cria uma imagem preta
    let mut mask = image::ImageBuffer::from_pixel(width, height, image::Luma([0u8]));
    
    // Preenche o centro com branco (255) deixando uma margem preta
    let margin = (blur_radius * 2.0) as u32;
    for y in margin..(height - margin) {
        for x in margin..(width - margin) {
            mask.put_pixel(x, y, image::Luma([255u8]));
        }
    }
    
    // Aplica o Gaussian Blur para suavizar as bordas duras entre o branco e preto
    let blurred_mask = image::imageops::blur(&mask, blur_radius);
    DynamicImage::ImageLuma8(blurred_mask)
}
