use fast_image_resize as fr;
use image::DynamicImage;
use ravif::Encoder;

/// Convert a DynamicImage to AVIF format for master storage.
pub fn convert_to_avif(img: &DynamicImage) -> anyhow::Result<Vec<u8>> {
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    let pixels: &[rgb::RGBA8] = unsafe {
        std::slice::from_raw_parts(
            rgba.as_raw().as_ptr() as *const rgb::RGBA8,
            width as usize * height as usize,
        )
    };
    let img_ref = ravif::Img::new(pixels, width as usize, height as usize);

    let encoder = Encoder::new().with_quality(85.0).with_speed(6);

    let res = encoder
        .encode_rgba(img_ref)
        .map_err(|e| anyhow::anyhow!("AVIF encoding failed: {}", e))?;

    Ok(res.avif_file)
}

/// Extract dominant colors from an image using K-Means clustering.
pub fn extract_dominant_colors(img: &DynamicImage) -> Vec<String> {
    let src_image = fr::images::Image::from_vec_u8(
        img.width(),
        img.height(),
        img.to_rgba8().into_raw(),
        fr::PixelType::U8x4,
    )
    .unwrap();

    let dst_width = 16;
    let dst_height = 16;
    let mut dst_image = fr::images::Image::new(dst_width, dst_height, fr::PixelType::U8x4);
    let mut resizer = fr::Resizer::new();
    resizer
        .resize(
            &src_image,
            &mut dst_image,
            &fr::ResizeOptions::new()
                .resize_alg(fr::ResizeAlg::Convolution(fr::FilterType::Lanczos3)),
        )
        .unwrap();

    let pixels = dst_image.buffer();

    const K: usize = 5;
    let mut centroids: Vec<[f32; 3]> = Vec::new();

    for i in (0..pixels.len()).step_by(4) {
        let p = [pixels[i] as f32, pixels[i + 1] as f32, pixels[i + 2] as f32];
        if centroids.len() < K && !centroids.contains(&p) {
            centroids.push(p);
        }
        if centroids.len() >= K {
            break;
        }
    }

    let mut assignments = vec![0; pixels.len() / 4];

    for _ in 0..10 {
        let mut new_centroids = [[0.0; 3]; K];
        let mut counts = [0; K];

        for (idx, i) in (0..pixels.len()).step_by(4).enumerate() {
            let p = [pixels[i] as f32, pixels[i + 1] as f32, pixels[i + 2] as f32];
            let mut best_dist = f32::MAX;
            let mut best_k = 0;

            for (k, c) in centroids.iter().enumerate() {
                let dist = (p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2) + (p[2] - c[2]).powi(2);
                if dist < best_dist {
                    best_dist = dist;
                    best_k = k;
                }
            }
            assignments[idx] = best_k;
            new_centroids[best_k][0] += p[0];
            new_centroids[best_k][1] += p[1];
            new_centroids[best_k][2] += p[2];
            counts[best_k] += 1;
        }

        let mut changed = false;
        for k in 0..K {
            if counts[k] > 0 {
                let nc = [
                    new_centroids[k][0] / counts[k] as f32,
                    new_centroids[k][1] / counts[k] as f32,
                    new_centroids[k][2] / counts[k] as f32,
                ];
                if nc != centroids[k] {
                    centroids[k] = nc;
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }

    let mut cluster_counts = vec![(0usize, 0usize); K];
    for (k, count) in cluster_counts.iter_mut().enumerate() {
        count.0 = k;
    }
    for &a in &assignments {
        cluster_counts[a].1 += 1;
    }
    cluster_counts.sort_by_key(|b| std::cmp::Reverse(b.1));

    cluster_counts
        .into_iter()
        .take(3)
        .filter(|(_, count)| *count > 0)
        .map(|(k, _)| {
            let c = centroids[k];
            format!("#{:02X}{:02X}{:02X}", c[0] as u8, c[1] as u8, c[2] as u8)
        })
        .collect()
}

/// Generate a thumbnail for the UI.
pub fn generate_thumbnail(img: &DynamicImage, size: u32) -> Vec<u8> {
    let thumb = img.thumbnail(size, size);
    let mut buffer = std::io::Cursor::new(Vec::new());
    thumb
        .write_to(&mut buffer, image::ImageFormat::Jpeg)
        .unwrap();
    buffer.into_inner()
}
