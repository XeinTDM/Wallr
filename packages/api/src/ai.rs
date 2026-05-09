#![cfg(feature = "server")]
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::clip;
use hf_hub::api::sync::Api;
use image::DynamicImage;
use std::sync::OnceLock;
use tokenizers::Tokenizer;

pub static TAGGER: OnceLock<LocalTagger> = OnceLock::new();

pub fn init_tagger() {
    let standard_tags = crate::tags::CATEGORIES
        .iter()
        .map(|(id, _)| id.to_string())
        .collect();

    std::thread::spawn(|| match LocalTagger::new(standard_tags) {
        Ok(tagger) => {
            TAGGER.set(tagger).ok();
        }
        Err(e) => {
            eprintln!("⚠️ Failed to initialize AI Tagger: {}", e);
        }
    });
}

pub struct LocalTagger {
    model: clip::ClipModel,
    device: Device,
    text_embeddings: Tensor,
    tags: Vec<String>,
    mean: Tensor,
    std: Tensor,
}

impl LocalTagger {
    pub fn new(tags: Vec<String>) -> anyhow::Result<Self> {
        let device = Device::new_cuda(0)
            .or_else(|_| Device::new_metal(0))
            .unwrap_or(Device::Cpu);

        let api = Api::new().map_err(|e| anyhow::anyhow!("Failed to create HF API: {}", e))?;
        let repo = api.repo(hf_hub::Repo::with_revision(
            "openai/clip-vit-base-patch32".to_string(),
            hf_hub::RepoType::Model,
            "refs/pr/15".to_string(),
        ));

        let model_file = repo
            .get("model.safetensors")
            .map_err(|e| anyhow::anyhow!("Failed to download weights: {}", e))?;
        let tokenizer_file = repo
            .get("tokenizer.json")
            .map_err(|e| anyhow::anyhow!("Failed to download tokenizer: {}", e))?;

        let config = clip::ClipConfig::vit_base_patch32();

        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[model_file], DType::F32, &device)
                ?
        };
        let model = clip::ClipModel::new(vb, &config)?;

        let tokenizer = Tokenizer::from_file(tokenizer_file).map_err(|e| anyhow::anyhow!("{}", e))?;

        let tag_prompts: Vec<String> = tags.iter().map(|t| format!("a photo of {}", t)).collect();
        let tokens = tokenizer
            .encode_batch(tag_prompts, true)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let token_ids: Vec<Vec<u32>> = tokens.iter().map(|t| t.get_ids().to_vec()).collect();

        let max_len = token_ids.iter().map(|v| v.len()).max().unwrap_or(0);
        let mut padded_tokens = Vec::new();
        for mut id_vec in token_ids {
            id_vec.resize(max_len, 0);
            padded_tokens.push(id_vec);
        }

        let text_tensor = Tensor::new(padded_tokens, &device)?;
        let text_features = model
            .get_text_features(&text_tensor)
            ?;

        let text_norms = text_features
            .sqr()
            ?
            .sum_keepdim(1)
            ?
            .sqrt()
            ?;

        let normalized_text_features = text_features
            .broadcast_div(&text_norms)
            ?;

        let mean = Tensor::new(&[0.48145466f32, 0.4578275, 0.40821073], &device)?
            .reshape((1, 3, 1, 1))?;
        let std = Tensor::new(&[0.26862954f32, 0.26130258, 0.27577711], &device)?
            .reshape((1, 3, 1, 1))?;

        Ok(Self {
            model,
            device,
            text_embeddings: normalized_text_features,
            tags,
            mean,
            std,
        })
    }

    pub fn tag_image(&self, img: &DynamicImage) -> anyhow::Result<Vec<String>> {
        let resized = img.resize_exact(224, 224, image::imageops::FilterType::Triangle);
        let pixels = resized.to_rgb8().into_raw();

        let tensor = Tensor::from_vec(pixels, (224, 224, 3), &self.device)
            ?
            .permute((2, 0, 1))
            ? // HWC to CHW
            .to_dtype(DType::F32)
            ?
            .affine(1.0 / 255.0, 0.0)
            ?
            .unsqueeze(0)
            ?;

        let tensor = tensor
            .broadcast_sub(&self.mean)
            ?
            .broadcast_div(&self.std)
            ?;

        let image_features = self
            .model
            .get_image_features(&tensor)
            ?;

        let img_norm = image_features
            .sqr()
            ?
            .sum_keepdim(1)
            ?
            .sqrt()
            ?;
        let normalized_image_features = image_features
            .broadcast_div(&img_norm)
            ?;

        let similarities = normalized_image_features
            .matmul(&self.text_embeddings.t()?)
            ?;

        let similarities_vec: Vec<f32> = similarities
            .squeeze(0)
            ?
            .to_vec1()
            ?;

        let threshold = 0.24;
        let mut matched_tags = Vec::new();

        for (i, &score) in similarities_vec.iter().enumerate() {
            if score > threshold && i < self.tags.len() {
                matched_tags.push(self.tags[i].clone());
            }
        }

        if matched_tags.is_empty() {
            matched_tags.push("misc".to_string());
        }

        Ok(matched_tags)
    }
}
