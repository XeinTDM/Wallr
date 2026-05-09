# Image Processing & On-Demand Delivery

To keep storage costs low and delivery speeds "optimal", the system operates on a single-source-of-truth model. 

## 1. Upload & Normalization (Ingest)
When a user uploads an image, the backend immediately performs the following pipeline in memory:
1. Decode image (discard malformed headers/metadata).
2. Send a low-res clone to the **AI Pipeline** and **Color Extractor**.
3. Encode the master image directly to **AVIF** using `ravif`.
4. Save to disk/S3.

### Encoding to AVIF (Pure Rust)
We use the `ravif` crate for pure Rust, in-memory AVIF encoding. This avoids external C-dependencies and allows us to fine-tune quality, alpha quality, and compression speed directly within the upload pipeline.

## 2. Fast Color Extraction

AI is too heavy for color extraction. Instead, we use mathematical clustering.

1.  Use `fast_image_resize` to scale the image down to 64x64 pixels.
2.  Group the remaining 4,096 pixels using K-Means clustering.
3.  Extract the top 3 dominant RGB values.
4.  Map those RGB values to standard CSS color palettes (e.g., #FF0000 -> Red
    Theme) using Euclidean distance in the CIELAB color space.

3. On-Demand Conversion (Egress)

If a user clicks "Download as JPEG", the AVIF master is fetched, decoded, and
converted.

  - Routing: GET /wallpaper/12345/download?format=jpeg&width=1920
  - Because Rust is so fast, generating a 1920x1080 JPEG from a decoded AVIF
    buffer takes ~20-40ms.
  - Once generated, cache it in memory (moka crate) or write it to a
    reverse-proxy cache directory (NGINX/Varnish) so subsequent requests for
    that exact resolution/format take 0ms.
