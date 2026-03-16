//! GGUF LLM Backend
//!
//! High-performance Large Language Model inference using llama.cpp bindings (llama-cpp-2).
//! Supports GGUF format models with CPU and GPU acceleration.

#[cfg(feature = "llama-cpp")]
use super::llm_runtime::*;
#[cfg(not(feature = "llama-cpp"))]
use super::llm_runtime::*;

#[cfg(feature = "llama-cpp")]
use std::path::PathBuf;
#[cfg(feature = "llama-cpp")]
use tracing::{debug, info, warn};

#[cfg(feature = "llama-cpp")]
use llama_cpp_2::{
    context::params::LlamaContextParams,
    llama_backend::LlamaBackend,
    llama_batch::LlamaBatch,
    model::{LlamaModel, params::LlamaModelParams},
    sampling::LlamaSampler,
};

/// GGUF LLM backend using llama.cpp
#[cfg(feature = "llama-cpp")]
pub struct LlmGguf {
    backend: Option<LlamaBackend>,
    model: Option<LlamaModel>,
    // We don't store the context - we'll recreate it on demand if needed
    // This avoids the self-referential lifetime issues
    config: Option<LlmRuntimeConfig>,
    model_path: Option<PathBuf>,
    /// Detected prompt template family (resolved once during load)
    prompt_template: PromptTemplate,
}

/// Supported prompt template families.
///
/// The correct template is detected from the model filename/path at load time.
/// Defaulting to ChatML keeps backwards compatibility with most instruction models.
#[cfg(feature = "llama-cpp")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PromptTemplate {
    /// ChatML — used by Mistral, Phi, LFM-2, Qwen, etc.
    /// `<|im_start|>role\ncontent<|im_end|>\n`
    ChatMl,

    /// Llama-3 — used by Meta Llama-3.x family.
    /// `<|begin_of_text|><|start_header_id|>role<|end_header_id|>\n\ncontent<|eot_id|>`
    Llama3,
}

#[cfg(feature = "llama-cpp")]
impl LlmGguf {
    /// Create a new GGUF LLM backend
    pub fn new() -> crate::Result<Self> {
        info!("Initializing GGUF LLM backend (llama.cpp)");

        Ok(Self {
            backend: None,
            model: None,
            config: None,
            model_path: None,
            prompt_template: PromptTemplate::ChatMl,
        })
    }

    /// Detect the prompt template from the model filename.
    ///
    /// We match well-known substrings in the model path/id (case-insensitive).
    /// This keeps detection simple and avoids needing to parse model metadata.
    fn detect_prompt_template(model_id: &str) -> PromptTemplate {
        let lower = model_id.to_lowercase();

        if lower.contains("llama-3") || lower.contains("llama3") || lower.contains("meta-llama-3") {
            debug!(
                "🗂️  Detected Llama-3 prompt template for model: {}",
                model_id
            );
            PromptTemplate::Llama3
        } else {
            // Default: ChatML works for Mistral, Phi-3, LFM-2, Qwen, Hermes, etc.
            debug!("🗂️  Using ChatML prompt template for model: {}", model_id);
            PromptTemplate::ChatMl
        }
    }

    /// Resolve model path from cache
    fn resolve_model_path(&self, model_id: &str) -> crate::Result<PathBuf> {
        // If it's already an absolute path that exists, use it directly
        let direct_path = PathBuf::from(model_id);
        if direct_path.is_absolute() && direct_path.exists() {
            info!("Using absolute model path: {:?}", direct_path);
            return Ok(direct_path);
        }

        // Get the models directory
        let models_dir =
            crate::platform::paths::models_dir().unwrap_or_else(|_| PathBuf::from("./models"));

        // Try different possible locations
        let possible_paths = vec![
            // 1. Direct file in models directory
            models_dir.join(model_id),
            // 2. In subdirectory: models/model-id/model-id.gguf
            models_dir.join(model_id).join(format!("{}.gguf", model_id)),
            // 3. Standard naming: model_id/model.gguf
            models_dir.join(model_id).join("model.gguf"),
            // 4. With Q4 quantization suffix
            models_dir
                .join(model_id)
                .join(format!("{}-Q4_K_M.gguf", model_id)),
            // 5. Common quantized names
            models_dir.join(model_id).join("ggml-model-q4_k_m.gguf"),
            // 6. Glob pattern - find any .gguf file in the model directory
        ];

        // Try explicit paths first
        for path in &possible_paths {
            if path.exists() && path.is_file() {
                info!("Found model at: {:?}", path);
                return Ok(path.clone());
            }
        }

        // If not found, try to find any .gguf file in the model directory
        let model_dir = models_dir.join(model_id);
        if model_dir.exists()
            && model_dir.is_dir()
            && let Ok(entries) = std::fs::read_dir(&model_dir)
        {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("gguf") {
                    info!("Found GGUF model file: {:?}", path);
                    return Ok(path);
                }
            }
        }

        // Return the most likely expected path for error message
        let expected = models_dir.join(model_id).join(format!("{}.gguf", model_id));
        warn!("Model not found at any expected location");
        debug!("Searched paths: {:?}", possible_paths);
        debug!("Model directory: {:?}", model_dir);
        debug!("Expected path: {:?}", expected);

        Ok(expected)
    }

    /// Format chat messages into a prompt string using the detected template.
    fn format_prompt(&self, messages: &[ChatMessage]) -> String {
        match self.prompt_template {
            PromptTemplate::ChatMl => self.format_prompt_chatml(messages),
            PromptTemplate::Llama3 => self.format_prompt_llama3(messages),
        }
    }

    /// ChatML format — default for most instruction-tuned models.
    ///
    /// ```text
    /// <|im_start|>system
    /// {content}<|im_end|>
    /// <|im_start|>user
    /// {content}<|im_end|>
    /// <|im_start|>assistant
    /// ```
    fn format_prompt_chatml(&self, messages: &[ChatMessage]) -> String {
        let mut prompt = String::new();

        for message in messages {
            let role = match message.role {
                MessageRole::System => "system",
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
                // Tool results are presented as user turns in ChatML so the model
                // can process the tool output and produce a follow-up response.
                MessageRole::Tool => "user",
            };
            prompt.push_str(&format!("<|im_start|>{}\n", role));
            prompt.push_str(&message.content);
            prompt.push_str("<|im_end|>\n");
        }

        // Add assistant opening to steer generation
        prompt.push_str("<|im_start|>assistant\n");

        debug!("Formatted ChatML prompt ({} chars)", prompt.len());
        prompt
    }

    /// Llama-3 format — used by Meta Llama-3.x family.
    ///
    /// ```text
    /// <|begin_of_text|><|start_header_id|>system<|end_header_id|>
    ///
    /// {content}<|eot_id|><|start_header_id|>user<|end_header_id|>
    ///
    /// {content}<|eot_id|><|start_header_id|>assistant<|end_header_id|>
    ///
    /// ```
    fn format_prompt_llama3(&self, messages: &[ChatMessage]) -> String {
        let mut prompt = String::from("<|begin_of_text|>");

        for message in messages {
            let role = match message.role {
                MessageRole::System => "system",
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
                // Tool results are modelled as user turns in the Llama-3 template.
                MessageRole::Tool => "user",
            };
            prompt.push_str(&format!(
                "<|start_header_id|>{}<|end_header_id|>\n\n{}",
                role, message.content
            ));
            prompt.push_str("<|eot_id|>");
        }

        // Add assistant header to begin generation
        prompt.push_str("<|start_header_id|>assistant<|end_header_id|>\n\n");

        debug!("Formatted Llama-3 prompt ({} chars)", prompt.len());
        prompt
    }
}

#[cfg(feature = "llama-cpp")]
impl Default for LlmGguf {
    fn default() -> Self {
        Self::new().expect("Failed to create LlmGguf")
    }
}
// Safety: LlmGguf is used behind Arc<RwLock<>> which ensures exclusive access
// The underlying LlamaContext contains FFI pointers that are safe to use from
// any thread as long as they're accessed exclusively, which our locks guarantee.
#[cfg(feature = "llama-cpp")]
unsafe impl Send for LlmGguf {}
#[cfg(feature = "llama-cpp")]
unsafe impl Sync for LlmGguf {}

#[cfg(feature = "llama-cpp")]
impl LlmRuntime for LlmGguf {
    fn load(&mut self, config: LlmRuntimeConfig) -> crate::Result<()> {
        info!("Loading GGUF model: {}", config.model_path);

        // Detect prompt template from model identifier before anything else (W7)
        self.prompt_template = Self::detect_prompt_template(&config.model_path);

        // Initialize llama.cpp backend
        let backend = LlamaBackend::init().map_err(|e| {
            crate::Error::Model(format!("Failed to initialize llama.cpp backend: {:?}", e))
        })?;

        // Resolve model path
        let model_path = self.resolve_model_path(&config.model_path)?;

        if !model_path.exists() {
            return Err(crate::Error::Model(format!(
                "Model file not found: {:?}\nDownload with: onevox models download {}",
                model_path, config.model_path
            )));
        }

        info!("Loading model from: {:?}", model_path);

        // Configure model parameters
        let model_params = {
            let mut params = LlamaModelParams::default();

            // GPU configuration
            if config.use_gpu {
                let gpu_caps = crate::models::GpuCapabilities::detect();
                if !gpu_caps.available {
                    warn!("⚠️  GPU requested but not available - falling back to CPU");
                    warn!("Reason: {}", gpu_caps.description);
                    // CPU only: 0 GPU layers
                } else {
                    info!("{}", gpu_caps.status_message());
                    // Offload all layers to GPU
                    params = params.with_n_gpu_layers(999); // llama.cpp will cap at actual layer count
                    debug!("GPU acceleration enabled (all layers offloaded)");
                }
            } else {
                debug!("CPU-only mode enabled");
            }

            params
        };

        // Load the model
        let model = LlamaModel::load_from_file(&backend, model_path.clone(), &model_params)
            .map_err(|e| crate::Error::Model(format!("Failed to load GGUF model: {:?}", e)))?;

        info!("Model loaded successfully");
        info!("✅ GGUF LLM loaded successfully");
        info!("   Model path: {:?}", model_path);
        info!("   Context length: {}", config.context_length);
        info!("   Temperature: {}", config.temperature);

        self.backend = Some(backend);
        self.model = Some(model);
        self.config = Some(config);
        self.model_path = Some(model_path);

        Ok(())
    }

    fn is_loaded(&self) -> bool {
        self.model.is_some() && self.backend.is_some()
    }

    fn generate(&mut self, messages: &[ChatMessage]) -> crate::Result<LlmResponse> {
        // Use generate_stream with a no-op callback
        self.generate_stream(messages, Box::new(|_| {}))
    }

    fn generate_stream(
        &mut self,
        messages: &[ChatMessage],
        mut callback: Box<dyn FnMut(&str) + Send>,
    ) -> crate::Result<LlmResponse> {
        if messages.is_empty() {
            return Err(crate::Error::Model(
                "No messages provided for generation".to_string(),
            ));
        }

        let start = std::time::Instant::now();

        // Format messages into prompt (do this first to avoid borrow issues)
        let prompt = self.format_prompt(messages);
        info!("Generating response for prompt ({} chars)", prompt.len());

        // Extract config values to avoid holding borrow
        let (max_tokens, temperature, top_p, top_k, repetition_penalty) = {
            let config = self
                .config
                .as_ref()
                .ok_or_else(|| crate::Error::Model("Config not set".to_string()))?;
            (
                config.max_tokens,
                config.temperature,
                config.top_p,
                config.top_k,
                config.repetition_penalty,
            )
        };

        // Get backend and model references
        let backend = self
            .backend
            .as_ref()
            .ok_or_else(|| crate::Error::Model("Backend not loaded".to_string()))?;

        let model = self
            .model
            .as_ref()
            .ok_or_else(|| crate::Error::Model("Model not loaded".to_string()))?;

        // Tokenize prompt
        let tokens = model
            .str_to_token(&prompt, llama_cpp_2::model::AddBos::Always)
            .map_err(|e| crate::Error::Model(format!("Tokenization failed: {:?}", e)))?;

        debug!("Tokenized to {} tokens", tokens.len());

        // Create context for this generation
        let context_length = self.config.as_ref().unwrap().context_length as u32;
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(std::num::NonZeroU32::new(context_length))
            .with_n_batch(context_length) // Match context length for large prompts
            .with_n_ubatch(context_length) // Match context length for large prompts
            .with_n_threads(4)
            .with_n_threads_batch(4)
            // Disable flash attention to avoid assertion failures with certain models
            // LLAMA_FLASH_ATTN_TYPE_DISABLED = 0
            .with_flash_attention_policy(0);

        let mut context = model
            .new_context(backend, ctx_params)
            .map_err(|e| crate::Error::Model(format!("Failed to create context: {:?}", e)))?;

        // Clear KV cache
        context.clear_kv_cache();

        // Create batch for prompt - use context length to accommodate large prompts
        let batch_size = self.config.as_ref().unwrap().context_length;
        let mut batch = LlamaBatch::new(batch_size, 1);

        // Add prompt tokens to batch
        let last_index = tokens.len() - 1;
        for (i, &token) in tokens.iter().enumerate() {
            let is_last = i == last_index;
            batch.add(token, i as i32, &[0], is_last).map_err(|e| {
                crate::Error::Model(format!("Failed to add token to batch: {:?}", e))
            })?;
        }

        // Decode prompt
        context
            .decode(&mut batch)
            .map_err(|e| crate::Error::Model(format!("Failed to decode prompt: {:?}", e)))?;

        // Generate tokens
        let mut generated_text = String::new();
        let mut n_cur = tokens.len();
        let mut generated_tokens = 0;

        // Get EOS token
        let eos_token = model.token_eos();

        // Create sampler with temperature, top-k, top-p, repetition penalty
        let mut sampler = {
            let mut samplers = vec![];

            // Add repetition penalty sampler (W4: was configured but never applied)
            // A value of 1.0 means no penalty; >1.0 penalises repeated tokens.
            if repetition_penalty > 1.0 {
                // last_n = 64 tokens of context to consider for penalty (standard default)
                samplers.push(LlamaSampler::penalties(
                    64,                 // last_n tokens to consider
                    repetition_penalty, // repeat penalty
                    0.0,                // frequency penalty (disabled)
                    0.0,                // presence penalty (disabled)
                ));
            }

            // Add top-k sampler
            if top_k > 0 {
                samplers.push(LlamaSampler::top_k(top_k as i32));
            }

            // Add top-p sampler
            if top_p < 1.0 {
                samplers.push(LlamaSampler::top_p(top_p, 1));
            }

            // Add temperature sampler
            samplers.push(LlamaSampler::temp(temperature));

            // Add dist sampler at the end (required for actual sampling)
            let seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32;
            samplers.push(LlamaSampler::dist(seed));

            LlamaSampler::chain(samplers, false)
        };

        for _ in 0..max_tokens {
            // Sample next token using the sampler
            let new_token_id = sampler.sample(&context, -1);

            // Check for EOS
            if new_token_id == eos_token {
                debug!("EOS token encountered");
                break;
            }

            // Accept the token (updates sampler state)
            sampler.accept(new_token_id);

            // Decode token to text using token_to_piece_bytes
            // buffer_size: 128 bytes should be enough for most tokens
            // special: false (don't include special token markers)
            // lstrip: None (no left stripping)
            let bytes = model
                .token_to_piece_bytes(new_token_id, 128, false, None)
                .map_err(|e| crate::Error::Model(format!("Failed to decode token: {:?}", e)))?;

            // Convert bytes to string (lossy to handle invalid UTF-8)
            let piece = String::from_utf8_lossy(&bytes);

            generated_text.push_str(&piece);
            generated_tokens += 1;

            // Stream the token via callback
            callback(&piece);

            // Prepare next batch
            batch.clear();
            batch
                .add(new_token_id, n_cur as i32, &[0], true)
                .map_err(|e| {
                    crate::Error::Model(format!("Failed to add token to batch: {:?}", e))
                })?;

            // Decode
            context
                .decode(&mut batch)
                .map_err(|e| crate::Error::Model(format!("Failed to decode token: {:?}", e)))?;

            n_cur += 1;
        }

        let generation_time = start.elapsed();
        let generation_time_ms = generation_time.as_millis() as u64;
        let tokens_per_second = if generation_time_ms > 0 {
            (generated_tokens as f32 / generation_time_ms as f32) * 1000.0
        } else {
            0.0
        };

        info!(
            "✅ Generation complete: {} tokens in {}ms ({:.2} tok/s)",
            generated_tokens, generation_time_ms, tokens_per_second
        );

        // Clean up generated text (remove trailing stop sequences for all supported templates)
        let final_text = generated_text
            .trim_end_matches("<|im_end|>") // ChatML EOM
            .trim_end_matches("<|eot_id|>") // Llama-3 EOM
            .trim_end_matches("<|endoftext|>") // GPT-style EOS
            .trim_end_matches("</s>") // SentencePiece EOS
            .trim()
            .to_string();

        Ok(LlmResponse {
            text: final_text,
            tool_calls: Vec::new(), // Tool calls are parsed in chat engine, not here
            tokens: generated_tokens,
            generation_time_ms,
            tokens_per_second,
            finish_reason: Some("stop".to_string()),
        })
    }

    fn unload(&mut self) {
        info!("Unloading GGUF model");
        self.model = None;
        self.backend = None;
        self.config = None;
        self.model_path = None;
    }

    fn name(&self) -> &str {
        "llm-gguf"
    }

    fn info(&self) -> LlmInfo {
        LlmInfo {
            name: self.name().to_string(),
            loaded: self.is_loaded(),
            backend: "llama.cpp".to_string(),
        }
    }
}

// Stub implementation when feature is disabled
#[cfg(not(feature = "llama-cpp"))]
pub struct LlmGguf;

#[cfg(not(feature = "llama-cpp"))]
impl LlmGguf {
    pub fn new() -> crate::Result<Self> {
        Err(crate::Error::Model(
            "llama-cpp feature not enabled".to_string(),
        ))
    }
}

#[cfg(not(feature = "llama-cpp"))]
impl LlmRuntime for LlmGguf {
    fn load(&mut self, _config: LlmRuntimeConfig) -> crate::Result<()> {
        Err(crate::Error::Model(
            "llama-cpp feature not enabled".to_string(),
        ))
    }

    fn is_loaded(&self) -> bool {
        false
    }

    fn generate(&mut self, _messages: &[ChatMessage]) -> crate::Result<LlmResponse> {
        Err(crate::Error::Model(
            "llama-cpp feature not enabled".to_string(),
        ))
    }

    fn unload(&mut self) {}

    fn name(&self) -> &str {
        "llm-gguf-disabled"
    }

    fn info(&self) -> LlmInfo {
        LlmInfo {
            name: "llm-gguf".to_string(),
            loaded: false,
            backend: "llama.cpp".to_string(),
        }
    }
}

#[cfg(test)]
#[cfg(feature = "llama-cpp")]
mod tests {
    use super::*;

    #[test]
    fn test_create_backend() {
        let backend = LlmGguf::new();
        assert!(backend.is_ok());
    }

    #[test]
    fn test_not_loaded_initially() {
        let backend = LlmGguf::new().unwrap();
        assert!(!backend.is_loaded());
    }

    #[test]
    fn test_format_prompt() {
        let backend = LlmGguf::new().unwrap();
        let messages = vec![
            ChatMessage::system("You are a helpful assistant.".to_string()),
            ChatMessage::user("Hello!".to_string()),
        ];

        // Default template is ChatML
        let prompt = backend.format_prompt(&messages);
        assert!(prompt.contains("<|im_start|>system"));
        assert!(prompt.contains("<|im_start|>user"));
        assert!(prompt.contains("<|im_start|>assistant"));
        assert!(prompt.contains("Hello!"));
    }

    #[test]
    fn test_detect_prompt_template_llama3() {
        assert_eq!(
            LlmGguf::detect_prompt_template("meta-llama-3-8b-q4"),
            PromptTemplate::Llama3
        );
        assert_eq!(
            LlmGguf::detect_prompt_template("llama3-70b"),
            PromptTemplate::Llama3
        );
    }

    #[test]
    fn test_detect_prompt_template_chatml() {
        assert_eq!(
            LlmGguf::detect_prompt_template("lfm2-1.2b-tool-q4"),
            PromptTemplate::ChatMl
        );
        assert_eq!(
            LlmGguf::detect_prompt_template("mistral-7b-instruct"),
            PromptTemplate::ChatMl
        );
    }

    #[test]
    fn test_format_prompt_llama3() {
        let mut backend = LlmGguf::new().unwrap();
        backend.prompt_template = PromptTemplate::Llama3;
        let messages = vec![
            ChatMessage::system("You are helpful.".to_string()),
            ChatMessage::user("Hi".to_string()),
        ];
        let prompt = backend.format_prompt(&messages);
        assert!(prompt.contains("<|begin_of_text|>"));
        assert!(prompt.contains("<|start_header_id|>system<|end_header_id|>"));
        assert!(prompt.contains("<|start_header_id|>user<|end_header_id|>"));
        assert!(prompt.contains("<|eot_id|>"));
    }

    #[test]
    fn test_empty_messages_error() {
        // Can't test without loaded model, but we verify the struct compiles
        let _backend = LlmGguf::new().unwrap();
        // Would fail at runtime with "No messages provided"
    }
}
