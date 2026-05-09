# AI Pipeline: Local Zero-Shot Tagging

To achieve an optimal tagging system without relying on paid APIs or Python, we use **CLIP (Contrastive Language-Image Pretraining)** running natively in Rust.

## How it Works
CLIP understands both images and text. Instead of training a model on "summer" or "female", we give the model an image and a list of hundreds of predefined tags. The model outputs a probability score for each tag.

### 1. Model Selection
We use a quantized version of CLIP to ensure it runs lightning-fast on the CPU (or local GPU) without eating up system RAM.
*   **Model:** `openai/clip-vit-base-patch32` (or a lighter MobileCLIP).
*   **Format:** Safetensors / GGUF (loaded via `candle`).

### 2. Implementation Framework
We utilize the Hugging Face `candle` machine learning framework (`candle-core`, `candle-nn`, `candle-transformers`) to run these operations natively in Rust. This enables zero-copy tensor operations and avoids the need for a separate Python backend.

## 3. Pipeline Flow

1.  Pre-processing: The uploaded image is resized to 224x224 and normalized (RGB
    values scaled between 0 and 1).
2.  Feature Extraction (Image): The image is passed through the Vision
    Transformer (ViT) to get an "Image Embedding" (a vector of 512 numbers).
3.  Feature Extraction (Text): Your predefined categories (e.g., ["human",
    "female", "summer", "dark theme", "anime", "nature"]) are tokenized and
    passed through the Text Transformer to get "Text Embeddings". (Note: You can
    pre-compute and cache the text embeddings at server startup for optimal
    performance!)
4.  Cosine Similarity: A fast dot-product operation compares the image embedding
    against all text embeddings.
5.  Thresholding: Any tag with a similarity score > 0.25 (tweak based on
    testing) is written to the database.

## 4. Architecture & Data Structures

The system is centered around a tagging service that initializes once at server startup.

*   **Initialization**: 
    1. Loads the CLIP model weights (e.g., `.safetensors` format) into memory (or GPU VRAM).
    2. Tokenizes and pre-computes the text embeddings for all configured database tags. This ensures we don't waste cycles re-calculating text embeddings on every upload.
*   **Inference Phase**:
    1. **Decode & Resize**: Incoming image byte buffers are decoded and resized to the required dimensions (e.g., 224x224).
    2. **Image Embedding**: Passed through the model to generate the image tensor.
    3. **Similarity Calculation**: A cosine similarity matrix operation is performed between the single image embedding and all pre-computed text embeddings.
    4. **Filtering**: Tags that score above a predefined confidence threshold (e.g., 0.85) are collected and attached to the wallpaper record in the database.
