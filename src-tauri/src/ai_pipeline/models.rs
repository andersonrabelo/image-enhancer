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
    println!(">>> SCUNet ONNX Iniciado: {}x{}", img.width(), img.height());
    
    let (orig_w, orig_h) = img.dimensions();
    let net_size = 512;
    let resized = img.resize_exact(net_size, net_size, image::imageops::FilterType::Triangle);
    let img_rgb = resized.to_rgb8();

    let mut input_tensor = Array4::<f32>::zeros((1, 3, net_size as usize, net_size as usize));
    for (x, y, pixel) in img_rgb.enumerate_pixels() {
        input_tensor[[0, 0, y as usize, x as usize]] = pixel[0] as f32 / 255.0; 
        input_tensor[[0, 1, y as usize, x as usize]] = pixel[1] as f32 / 255.0; 
        input_tensor[[0, 2, y as usize, x as usize]] = pixel[2] as f32 / 255.0; 
    }

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

    let mut out_img = image::RgbImage::new(net_size, net_size);
    for out_y in 0..net_size {
        for out_x in 0..net_size {
            let r = (output_view[[0, 0, out_y as usize, out_x as usize]] * 255.0).clamp(0.0, 255.0) as u8;
            let g = (output_view[[0, 1, out_y as usize, out_x as usize]] * 255.0).clamp(0.0, 255.0) as u8;
            let b = (output_view[[0, 2, out_y as usize, out_x as usize]] * 255.0).clamp(0.0, 255.0) as u8;
            out_img.put_pixel(out_x, out_y, image::Rgb([r, g, b]));
        }
    }

    let restored = DynamicImage::ImageRgb8(out_img).resize_exact(orig_w, orig_h, image::imageops::FilterType::Lanczos3);
    println!("<<< SCUNet Concluído");
    
    Ok(restored)
}
