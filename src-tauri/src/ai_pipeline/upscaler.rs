use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};
use std::sync::{Arc, Mutex};
use ort::session::Session;
use ndarray::Array4;

/// Motor Real-ESRGAN Tiled p/ ONNX Runtime
/// Converte a foto em fatias matemáticas (Tensores 1x3x256x256),
/// roda na Sessão Global de IA, e repassa para re-assemblagem x4.
pub async fn process_realesrgan(
    img: DynamicImage, 
    session_arc: Arc<Mutex<Session>>
) -> Result<DynamicImage, String> {
    println!(">>> Real-ESRGAN ONNX Tiling Iniciado: {}x{}", img.width(), img.height());

    let (width, height) = img.dimensions();
    let tile_size: u32 = 256; 
    let scale: u32 = 4; // realesrgan-x4 = multiplica resolução por 4x

    // Criamos o Canvas Final Gigante (x4)
    let out_width = width * scale;
    let out_height = height * scale;
    let mut final_img = RgbaImage::new(out_width, out_height);
    
    // Converte Input pra RGBA se n for 
    let img_rgba = img.to_rgba8();

    // Loop de Tiling: "Varre" blocos de 256x256 (Poderíamos usar overlap para bordas suaves no futuro)
    for y in (0..height).step_by(tile_size as usize) {
        for x in (0..width).step_by(tile_size as usize) {
            
            // Garantir que a crop window não passe do fim da imagem (Padding se necessário)
            let mut crop_w = tile_size;
            let mut crop_h = tile_size;
            
            if x + crop_w > width { crop_w = width - x; }
            if y + crop_h > height { crop_h = height - y; }
            
            // 1. Recortando e Preenchendo a fatia para forçar ser 256x256 (Exigência do Static Model)
            let mut tile = RgbaImage::new(tile_size, tile_size); // Preto por default
            let crop = img_rgba.view(x, y, crop_w, crop_h);
            image::imageops::replace(&mut tile, &crop.to_image(), 0, 0);

            // 2. Transforma RgbaImage -> Ndarray::Array4 (BCHW - Batch, Channel, Height, Width) RGB puro sem Alpha
            let mut input_tensor = Array4::<f32>::zeros((1, 3, 256, 256));
            
            for (tx, ty, pixel) in tile.enumerate_pixels() {
                // Real-ESRGAN espera valores float entre 0.0 e 1.0 (RGB)
                input_tensor[[0, 0, ty as usize, tx as usize]] = pixel[0] as f32 / 255.0; // R
                input_tensor[[0, 1, ty as usize, tx as usize]] = pixel[1] as f32 / 255.0; // G
                input_tensor[[0, 2, ty as usize, tx as usize]] = pixel[2] as f32 / 255.0; // B
            }

            // 3. Inferência VRAM Pesada no ONNX via `ort` binding
            // Envia o Tensor Input Tensor f32 pro modelo e extrai o Output Tensor (que sairá 1x3x1024x1024)
            let input_tensor_value = ort::value::Tensor::from_array(input_tensor).map_err(|e| format!("Erro convertendo Tensor: {}", e))?;
            
            println!("   Processando Tile X:{} Y:{} na GPU/CPU...", x, y);
            let session = session_arc.lock().map_err(|_| "Falha ao obter Mutex da Session")?;
            let input_name = session.inputs[0].name.as_str();
            let inputs = ort::inputs![input_name => input_tensor_value].map_err(|e| format!("Erro cfg inputs Real-ESRGAN: {}", e))?;
            let outputs = session.run(inputs).map_err(|e| format!("Falha na predição Real-ESRGAN: {}", e))?;
            
            // 4. Extrai a saída da Memória diretamente para um ndarray
            let output_view = outputs[0].try_extract_tensor::<f32>().map_err(|e| format!("Tensor mismatch Real-ESRGAN: {}", e))?;

            // 5. Costurar (Stitching) no Output Image Final Gigante
            // Transformar BCHW Tensor devolta p/ RBA e colar apenas o tamanho Original (crop_w * scale)
            let tile_out_w = crop_w * scale;
            let tile_out_h = crop_h * scale;
            
            let dest_x = x * scale;
            let dest_y = y * scale;
            
            for out_y in 0..tile_out_h {
                for out_x in 0..tile_out_w {
                    let r = (output_view[[0, 0, out_y as usize, out_x as usize]] * 255.0).clamp(0.0, 255.0) as u8;
                    let g = (output_view[[0, 1, out_y as usize, out_x as usize]] * 255.0).clamp(0.0, 255.0) as u8;
                    let b = (output_view[[0, 2, out_y as usize, out_x as usize]] * 255.0).clamp(0.0, 255.0) as u8;
                    
                    // Recupera o Alpha Ratio correspondente da imagem original (já que ONNX só operou RGB)
                    let original_rx = x + (out_x / scale);
                    let original_ry = y + (out_y / scale);
                    let original_alpha = if original_rx < width && original_ry < height {
                       img_rgba.get_pixel(original_rx, original_ry)[3] 
                    } else { 255 };

                    final_img.put_pixel(dest_x + out_x, dest_y + out_y, Rgba([r, g, b, original_alpha]));
                }
            }
        }
    }

    println!("<<< Real-ESRGAN Concluído. Dimensões Finais: {}x{}", out_width, out_height);
    Ok(DynamicImage::ImageRgba8(final_img))
}
