use image::{DynamicImage, GenericImage, GenericImageView, Rgba};

use crate::ai_pipeline::segmentation::BoundingBox;

/// Reposiciona e mescla um rosto/recorte processado de volta na imagem base.
/// Utiliza a "feathered mask" (máscara de transparência suave) criada no recorte inicial
/// para realizar o Alpha Blending (mistura) suave nas bordas.
pub fn stitch_face_seamless(
    mut base_img: DynamicImage, 
    processed_crop: &DynamicImage, 
    feathered_mask: &DynamicImage, 
    bbox: &BoundingBox
) -> Result<DynamicImage, String> {
    
    // Verificação de segurança das dimensões
    if processed_crop.width() != bbox.width || processed_crop.height() != bbox.height {
        return Err("O recorte processado possui dimensões diferentes da BoundingBox original.".into());
    }

    if feathered_mask.width() != bbox.width || feathered_mask.height() != bbox.height {
        return Err("A máscara de mesclagem é incompatível com o tamanho do recorte.".into());
    }

    let mask_luma = feathered_mask.to_luma8();
    let crop_rgba = processed_crop.to_rgba8();

    // Manipula os pixels da imagem base mutavelmente
    for y in 0..bbox.height {
        for x in 0..bbox.width {
            let base_x = bbox.x + x;
            let base_y = bbox.y + y;
            
            // Impede ultrapassar os limites acidentalmente
            if base_x >= base_img.width() || base_y >= base_img.height() {
                continue;
            }

            // A máscara nos diz o "peso" do pixel novo (0 a 255)
            // 255 (Branco) = 100% pixel novo (Centro do Rosto)
            // 0 (Preto) = 100% pixel original (Bordas fora do Rosto)
            let alpha_pixel = mask_luma.get_pixel(x, y)[0] as f32 / 255.0;

            let new_pixel = crop_rgba.get_pixel(x, y);
            let current_pixel = base_img.get_pixel(base_x, base_y);

            // Fórmula padrão de Alpha Blending:
            // Output = (New * alpha) + (Old * (1 - alpha))
            let r = (new_pixel[0] as f32 * alpha_pixel + current_pixel[0] as f32 * (1.0 - alpha_pixel)) as u8;
            let g = (new_pixel[1] as f32 * alpha_pixel + current_pixel[1] as f32 * (1.0 - alpha_pixel)) as u8;
            let b = (new_pixel[2] as f32 * alpha_pixel + current_pixel[2] as f32 * (1.0 - alpha_pixel)) as u8;
            let a = current_pixel[3]; // Preserva a transparência original do fundo

            let blended = Rgba([r, g, b, a]);
            base_img.put_pixel(base_x, base_y, blended);
        }
    }

    Ok(base_img)
}
