use image::{DynamicImage, GenericImageView};
use std::sync::{Arc, Mutex};
use ort::session::Session;
use ndarray::Array4;

#[derive(Debug)]
pub enum ProcessTarget {
    Face(DynamicImage, DynamicImage), // (Recorte, Máscara)
    Body(DynamicImage),               // Para a imagem base ou grandes blocos
}


/// Processa o Denoising da Imagem Base no SCUNet
pub async fn process_scunet(
    img: DynamicImage, 
    session_arc: Arc<Mutex<Session>>
) -> Result<DynamicImage, String> {
    println!(">>> SCUNet ONNX Tiling Iniciado: {}x{}", img.width(), img.height());
    
    let (width, height) = img.dimensions();
    let tile_size = 512;
    let img_rgb = img.to_rgb8();
    let mut out_img = image::RgbImage::new(width, height);

    for y in (0..height).step_by(tile_size as usize) {
        for x in (0..width).step_by(tile_size as usize) {
            let mut crop_w = tile_size;
            let mut crop_h = tile_size;
            
            if x + crop_w > width { crop_w = width - x; }
            if y + crop_h > height { crop_h = height - y; }

            // 1. Recortar e preparar tile 512x512
            let mut tile = image::RgbImage::new(tile_size, tile_size);
            let crop = img_rgb.view(x, y, crop_w, crop_h);
            image::imageops::replace(&mut tile, &crop.to_image(), 0, 0);

            let mut input_tensor = Array4::<f32>::zeros((1, 3, tile_size as usize, tile_size as usize));
            for (tx, ty, pixel) in tile.enumerate_pixels() {
                input_tensor[[0, 0, ty as usize, tx as usize]] = pixel[0] as f32 / 255.0; 
                input_tensor[[0, 1, ty as usize, tx as usize]] = pixel[1] as f32 / 255.0; 
                input_tensor[[0, 2, ty as usize, tx as usize]] = pixel[2] as f32 / 255.0; 
            }

            // 2. Inferência
            let input_tensor_value = ort::value::Tensor::from_array(input_tensor)
                .map_err(|e| format!("Erro Tensor SCUNet: {}", e))?;
                
            let session = session_arc.lock().map_err(|_| "Falha de Mutex SCU")?;
            let input_name = session.inputs[0].name.as_str();
            let inputs = ort::inputs![input_name => input_tensor_value]
                .map_err(|e| format!("Erro inputs SCU: {}", e))?;
            let outputs = session.run(inputs)
                .map_err(|e| format!("Falha predição SCUNet: {}", e))?;

            let output_view = outputs[0].try_extract_tensor::<f32>()
                .map_err(|e| format!("Mismatch Tensor SCUNet: {}", e))?;

            // 3. Recompor tile no output
            for ty in 0..crop_h {
                for tx in 0..crop_w {
                    let r = (output_view[[0, 0, ty as usize, tx as usize]] * 255.0).clamp(0.0, 255.0) as u8;
                    let g = (output_view[[0, 1, ty as usize, tx as usize]] * 255.0).clamp(0.0, 255.0) as u8;
                    let b = (output_view[[0, 2, ty as usize, tx as usize]] * 255.0).clamp(0.0, 255.0) as u8;
                    out_img.put_pixel(x + tx, y + ty, image::Rgb([r, g, b]));
                }
            }
        }
    }

    println!("<<< SCUNet Concluído");
    Ok(DynamicImage::ImageRgb8(out_img))
}
