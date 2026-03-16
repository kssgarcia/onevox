//! Chat Engine - STT→LLM→TTS Pipeline
//!
//! Orchestrates voice-based conversational AI by chaining:
//! 1. Speech-to-Text (Whisper) - transcribe user speech
//! 2. Large Language Model (GGUF/ONNX) - generate response
//! 3. Text-to-Speech (Kokoro) - synthesize audio response
//! 4. Audio Playback - play response back to user

use crate::Config;
use crate::audio::{AudioPlayer, ContinuousAudioPlayer};
use crate::models::{
    ModelRuntime, Transcription,
    llm_runtime::{ChatMessage, LlmResponse, LlmRuntime, LlmRuntimeConfig},
    tts_runtime::{TtsRuntime, TtsRuntimeConfig, TtsSynthesis},
};
use crate::tools::{
    MemoryTool, ObsidianTool, ToolRegistry, format_tool_definitions, parse_tool_calls,
};

#[cfg(target_os = "macos")]
use crate::tools::AppLauncherTool;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::sentence_splitter::SentenceSplitter;

/// Maximum conversation history length (in messages)
const MAX_HISTORY_LENGTH: usize = 20;

/// Chat engine state
pub struct ChatEngine {
    /// Configuration
    config: Arc<RwLock<Config>>,

    /// STT model (shared with transcription mode)
    stt_model: Arc<RwLock<Box<dyn ModelRuntime>>>,

    /// LLM runtime
    llm_runtime: Arc<RwLock<Box<dyn LlmRuntime>>>,

    /// TTS runtime
    tts_runtime: Arc<RwLock<Box<dyn TtsRuntime>>>,

    /// Audio player
    audio_player: Arc<RwLock<AudioPlayer>>,

    /// Conversation history
    history: Arc<RwLock<Vec<ChatMessage>>>,

    /// System prompt
    system_prompt: Arc<RwLock<String>>,

    /// Tool registry for LLM tool calling
    tool_registry: Arc<RwLock<ToolRegistry>>,
}

/// Chat response with complete metrics
#[derive(Debug, Clone)]
pub struct ChatResponse {
    /// User's transcribed speech
    pub user_text: String,

    /// Assistant's text response
    pub assistant_text: String,

    /// STT metrics
    pub stt_duration_ms: u64,

    /// LLM metrics
    pub llm_response: LlmResponse,

    /// TTS metrics
    pub tts_synthesis: TtsSynthesis,

    /// Audio playback duration (ms)
    pub playback_duration_ms: u64,

    /// Total pipeline duration (ms)
    pub total_duration_ms: u64,
}

impl ChatEngine {
    /// Create a new chat engine
    pub fn new(
        config: Arc<RwLock<Config>>,
        stt_model: Arc<RwLock<Box<dyn ModelRuntime>>>,
        llm_runtime: Arc<RwLock<Box<dyn LlmRuntime>>>,
        tts_runtime: Arc<RwLock<Box<dyn TtsRuntime>>>,
    ) -> crate::Result<Self> {
        info!("🤖 Initializing Chat Engine");

        let audio_player = AudioPlayer::new()
            .map_err(|e| crate::Error::Audio(format!("Failed to create audio player: {}", e)))?;

        // Initialize tool registry
        let tool_registry = Self::init_tool_registry(&config)?;

        // W2: Read system_prompt from config instead of hardcoding it.
        // Fall back to a sensible default only when the configured value is empty.
        let system_prompt = {
            let cfg = config.try_read().map_err(|_| {
                crate::Error::Other("Failed to read config for system prompt".to_string())
            })?;
            let prompt = cfg.chat.llm.system_prompt.trim().to_string();
            if prompt.is_empty() {
                "You are a helpful voice assistant. Be conversational, friendly, and concise. \
                 Only use tools when the user explicitly asks you to perform an action. \
                 For questions about your capabilities or general conversation, just respond naturally.".to_string()
            } else {
                prompt
            }
        };

        Ok(Self {
            config,
            stt_model,
            llm_runtime,
            tts_runtime,
            audio_player: Arc::new(RwLock::new(audio_player)),
            history: Arc::new(RwLock::new(Vec::new())),
            system_prompt: Arc::new(RwLock::new(system_prompt)),
            tool_registry: Arc::new(RwLock::new(tool_registry)),
        })
    }

    /// Initialize tool registry with available tools
    fn init_tool_registry(config: &Arc<RwLock<Config>>) -> crate::Result<ToolRegistry> {
        // Since we're in a sync context but need to read config, we'll use try_read
        let cfg = config
            .try_read()
            .map_err(|_| crate::Error::Other("Failed to read config".to_string()))?;

        let mut registry = ToolRegistry::new();

        if cfg.tools.enabled {
            info!("🔧 Initializing tools...");

            // Register Obsidian tool
            if let Some(vault_path) = &cfg.tools.obsidian_vault_path {
                match ObsidianTool::new(std::path::PathBuf::from(vault_path)) {
                    Ok(obsidian) => {
                        registry.register(Arc::new(obsidian));
                        info!("  ✓ Obsidian tool registered (vault: {})", vault_path);
                    }
                    Err(e) => {
                        warn!("  ✗ Failed to initialize Obsidian tool: {}", e);
                    }
                }
            }

            // Register Memory tool
            if let Some(storage_path) = &cfg.tools.memory_storage_path {
                match MemoryTool::new(std::path::PathBuf::from(storage_path)) {
                    Ok(memory) => {
                        registry.register(Arc::new(memory));
                        info!("  ✓ Memory tool registered (storage: {})", storage_path);
                    }
                    Err(e) => {
                        warn!("  ✗ Failed to initialize Memory tool: {}", e);
                    }
                }
            }

            // Register App Launcher tool (macOS only for now)
            #[cfg(target_os = "macos")]
            {
                let app_launcher = AppLauncherTool::new();
                registry.register(Arc::new(app_launcher));
                info!("  ✓ App Launcher tool registered");
            }

            info!("✅ {} tools registered", registry.tool_names().len());
        } else {
            info!("🔧 Tools disabled in configuration");
        }

        Ok(registry)
    }

    /// Process audio through the full STT→LLM→TTS pipeline
    pub async fn process_audio(&self, audio_data: Vec<f32>) -> crate::Result<ChatResponse> {
        let pipeline_start = Instant::now();

        info!("🎙️  Starting chat pipeline ({} samples)", audio_data.len());

        // Step 1: Speech-to-Text
        info!("📝 Step 1/4: Transcribing speech...");
        let stt_start = Instant::now();
        let transcription = self.transcribe_audio(audio_data).await?;
        let stt_duration_ms = stt_start.elapsed().as_millis() as u64;

        if transcription.text.trim().is_empty() {
            warn!("⚠️  Empty transcription - skipping LLM/TTS");
            return Err(crate::Error::Other("Empty transcription".to_string()));
        }

        info!(
            "✅ Transcribed: \"{}\" ({}ms)",
            transcription.text, stt_duration_ms
        );

        // Step 2: Generate LLM response (includes agentic tool-call loop)
        info!("🧠 Step 2/4: Generating response...");
        let llm_start = Instant::now();
        debug!("About to call generate_response");
        let llm_response = self.generate_response(&transcription.text).await?;
        debug!("generate_response returned");
        let llm_duration_ms = llm_start.elapsed().as_millis() as u64;
        debug!("Calculated LLM duration");

        info!(
            "✅ Generated: \"{}\" ({} tokens, {:.2} tok/s)",
            llm_response.text.chars().take(50).collect::<String>(),
            llm_response.tokens,
            llm_response.tokens_per_second
        );
        debug!("Finished logging LLM response");

        // generate_response now handles the full agentic loop internally, so the
        // final text ready for TTS is already in llm_response.text.
        let final_response_text = llm_response.text.clone();

        // Step 3: Synthesize speech
        info!("🔊 Step 3/4: Synthesizing speech...");
        debug!("About to acquire TTS write lock");
        let tts_start = Instant::now();
        let tts_synthesis = self.synthesize_speech(&final_response_text).await?;
        debug!("TTS synthesis complete, releasing lock");
        let tts_duration_ms = tts_start.elapsed().as_millis() as u64;

        info!(
            "✅ Synthesized: {:.2}s audio (RTF: {:.3}, {}ms)",
            tts_synthesis.duration_secs(),
            tts_synthesis.rtf,
            tts_duration_ms
        );

        // Step 4: Play audio
        info!("🔈 Step 4/4: Playing audio...");
        let playback_start = Instant::now();
        self.play_audio(&tts_synthesis).await?;
        let playback_duration_ms = playback_start.elapsed().as_millis() as u64;

        let total_duration_ms = pipeline_start.elapsed().as_millis() as u64;

        info!(
            "✅ Chat pipeline complete: {}ms total (STT: {}ms, LLM: {}ms, TTS: {}ms, Play: {}ms)",
            total_duration_ms,
            stt_duration_ms,
            llm_duration_ms,
            tts_duration_ms,
            playback_duration_ms
        );

        Ok(ChatResponse {
            user_text: transcription.text,
            assistant_text: final_response_text.clone(),
            stt_duration_ms,
            llm_response,
            tts_synthesis,
            playback_duration_ms,
            total_duration_ms,
        })
    }

    /// Process audio through STREAMING STT→LLM→TTS pipeline
    ///
    /// This method enables concurrent processing:
    /// - LLM generates tokens
    /// - As sentences complete, they're sent to TTS
    /// - As TTS produces audio, it's immediately played
    ///
    /// This significantly reduces perceived latency for long responses.
    pub async fn process_audio_streaming(
        &self,
        audio_data: Vec<f32>,
    ) -> crate::Result<ChatResponse> {
        let pipeline_start = Instant::now();

        info!(
            "🎙️  Starting STREAMING chat pipeline ({} samples)",
            audio_data.len()
        );

        // Step 1: Speech-to-Text (same as before)
        info!("📝 Step 1/4: Transcribing speech...");
        let stt_start = Instant::now();
        let transcription = self.transcribe_audio(audio_data).await?;
        let stt_duration_ms = stt_start.elapsed().as_millis() as u64;

        if transcription.text.trim().is_empty() {
            warn!("⚠️  Empty transcription - skipping LLM/TTS");
            return Err(crate::Error::Other("Empty transcription".to_string()));
        }

        info!(
            "✅ Transcribed: \"{}\" ({}ms)",
            transcription.text, stt_duration_ms
        );

        // Step 2: Setup concurrent streaming pipeline
        info!("🧠 Step 2/4: Starting streaming generation...");
        let llm_start = Instant::now();

        // Create sentence splitter for detecting complete sentences
        let splitter = Arc::new(tokio::sync::Mutex::new(SentenceSplitter::new()));

        // Create continuous audio player and channel for samples
        let continuous_player = ContinuousAudioPlayer::new()?;
        let (audio_tx, audio_rx) = tokio::sync::mpsc::channel::<Vec<f32>>(16);

        // Spawn continuous audio playback task (use spawn_blocking since cpal::Stream is !Send)
        let playback_task = tokio::task::spawn_blocking({
            move || {
                let playback_start = Instant::now();
                // Reuse the current Tokio runtime handle — do NOT create a new Runtime here.
                let rt = tokio::runtime::Handle::current();
                rt.block_on(async move {
                    if let Err(e) = continuous_player.play_stream(audio_rx, 24000).await {
                        warn!("Playback stream error: {}", e);
                    }
                });
                playback_start.elapsed().as_millis() as u64
            }
        });

        // Create TTS channel for sentence processing
        let (sentence_tx, mut sentence_rx) = tokio::sync::mpsc::channel::<String>(16);

        // Spawn TTS processing task
        let tts_runtime = Arc::clone(&self.tts_runtime);
        let audio_sender = audio_tx.clone();
        let tts_task = tokio::spawn(async move {
            let mut tts_count = 0;
            let mut total_tts_ms = 0u64;

            while let Some(sentence) = sentence_rx.recv().await {
                tts_count += 1;
                debug!(
                    "🔊 TTS processing sentence {}: \"{}\"",
                    tts_count,
                    sentence.chars().take(50).collect::<String>()
                );

                let tts_start = Instant::now();

                // Synthesize the sentence
                let mut tts = tts_runtime.write().await;
                match tts.synthesize(&sentence) {
                    Ok(synthesis) => {
                        let tts_duration = tts_start.elapsed().as_millis() as u64;
                        total_tts_ms += tts_duration;

                        info!(
                            "✅ TTS sentence {} complete: {:.2}s audio in {}ms (RTF: {:.3})",
                            tts_count,
                            synthesis.duration_secs(),
                            tts_duration,
                            synthesis.rtf
                        );

                        // Send samples directly to continuous player
                        if let Err(e) = audio_sender.send(synthesis.samples).await {
                            warn!("Failed to send audio samples: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        warn!("TTS synthesis failed for sentence {}: {}", tts_count, e);
                    }
                }
                drop(tts);
            }

            debug!(
                "🔊 TTS task complete: {} sentences, {}ms total",
                tts_count, total_tts_ms
            );
            (tts_count, total_tts_ms)
        });

        // W5 FIX: Use a std::sync::mpsc channel for the synchronous LLM callback instead of
        // spawning a new OS thread + full Tokio Runtime per token (~20-50/sec).
        // The channel is a FIFO queue; the async side drains it in a dedicated tokio task.
        let (token_tx, token_rx) = std::sync::mpsc::channel::<String>();

        // Spawn an async task that owns the sentence splitter and sentence channel.
        // It reads tokens from the std mpsc channel (non-blocking poll) and feeds
        // them to the SentenceSplitter, forwarding complete sentences to TTS.
        let splitter_clone = Arc::clone(&splitter);
        let sentence_tx_for_task = sentence_tx.clone();
        let full_response_text = Arc::new(tokio::sync::Mutex::new(String::new()));
        let full_response_for_task = Arc::clone(&full_response_text);

        let token_drain_task = tokio::spawn(async move {
            // We drive this task by yielding in a tight loop so we can process
            // tokens promptly without burning a dedicated OS thread.
            loop {
                // Drain all currently available tokens in one burst.
                let mut got_any = false;
                loop {
                    match token_rx.try_recv() {
                        Ok(token) => {
                            got_any = true;

                            // Accumulate full response text
                            {
                                let mut text = full_response_for_task.lock().await;
                                text.push_str(&token);
                            }

                            // Feed to sentence splitter
                            let sentences = {
                                let mut splitter = splitter_clone.lock().await;
                                splitter.add_chunk(&token)
                            };

                            // Forward complete sentences to TTS
                            for sentence in sentences {
                                if let Err(e) = sentence_tx_for_task.send(sentence).await {
                                    debug!("Failed to send sentence to TTS: {}", e);
                                }
                            }
                        }
                        Err(std::sync::mpsc::TryRecvError::Empty) => break,
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                            // Sender dropped — LLM generation is done.
                            return;
                        }
                    }
                }

                if !got_any {
                    // Nothing to do this iteration; yield so other tasks can run.
                    tokio::task::yield_now().await;
                }
            }
        });

        // Build message history for streaming LLM call
        let streaming_messages: Vec<ChatMessage> = {
            let mut msgs = Vec::new();

            let base_system_prompt = self.system_prompt.read().await;
            let mut system_prompt = base_system_prompt.clone();
            drop(base_system_prompt);

            let tool_registry = self.tool_registry.read().await;
            let definitions = tool_registry.get_definitions();
            if !definitions.is_empty() {
                system_prompt.push_str("\n\n");
                system_prompt.push_str(&format_tool_definitions(&definitions));
            }
            drop(tool_registry);

            msgs.push(ChatMessage::system(system_prompt));

            let history = self.history.read().await;
            msgs.extend(history.iter().cloned());
            drop(history);

            msgs.push(ChatMessage::user(transcription.text.clone()));
            msgs
        };

        // Generate with streaming callback.
        // The callback simply sends each token over the std mpsc channel — no
        // thread spawn, no runtime creation, just a cheap non-blocking send.
        let llm_response = {
            let mut llm = self.llm_runtime.write().await;

            llm.generate_stream(
                &streaming_messages,
                Box::new(move |token: &str| {
                    // This closure is called synchronously inside generate_stream.
                    // Sending over std::sync::mpsc is non-blocking when the channel has
                    // capacity, so it's safe to call here without spawning anything.
                    let _ = token_tx.send(token.to_string());
                }),
            )?
        };

        let llm_duration_ms = llm_start.elapsed().as_millis() as u64;

        // Wait for the token drain task to finish (it exits when token_tx is dropped,
        // which happens when generate_stream returns above — the capture closes it).
        let _ = token_drain_task.await;

        // Extract final response text accumulated by the drain task
        let streaming_text = {
            let text = full_response_text.lock().await;
            text.clone()
        };

        // Use the streaming-accumulated text as the authoritative response.
        // Fall back to llm_response.text if the accumulator is somehow empty.
        let raw_text = if streaming_text.is_empty() {
            llm_response.text.clone()
        } else {
            streaming_text
        };

        info!(
            "✅ LLM generation complete: \"{}\" ({} tokens, {:.2} tok/s, {}ms)",
            raw_text.chars().take(50).collect::<String>(),
            llm_response.tokens,
            llm_response.tokens_per_second,
            llm_duration_ms
        );

        // W1 FIX: After streaming is done, run the complete text through the tool-call
        // parser and execute any tool calls found. This mirrors the agentic loop in
        // `generate_response` but is applied to the streaming path.
        //
        // We also implement the feedback loop (W8) here: if tools were called we
        // append a Tool message and call generate() again for a natural follow-up.
        let final_response_text = self
            .run_tool_calls_and_get_final_text(raw_text.clone(), &streaming_messages)
            .await?;

        // Flush any remaining text from sentence splitter
        {
            let mut splitter = splitter.lock().await;
            if let Some(remaining) = splitter.flush() {
                debug!(
                    "🔊 Flushing remaining text to TTS: \"{}\"",
                    remaining.chars().take(50).collect::<String>()
                );
                // Only send the plain TTS portion (non-tool text from the first pass)
                let _ = sentence_tx.send(remaining).await;
            }
        }

        // Close the sentence channel (no more sentences coming)
        drop(sentence_tx);

        // Wait for TTS processing to complete
        let (tts_sentence_count, total_tts_ms) = tts_task
            .await
            .map_err(|e| crate::Error::Other(format!("TTS task failed: {}", e)))?;

        info!(
            "✅ All TTS processing complete: {} sentences in {}ms",
            tts_sentence_count, total_tts_ms
        );

        // Close audio channel to signal playback task to finish after draining buffer
        drop(audio_tx);

        // Wait for all audio playback to complete
        info!("🔈 Step 4/4: Waiting for playback to finish...");
        let playback_duration_ms = playback_task
            .await
            .map_err(|e| crate::Error::Other(format!("Playback task failed: {}", e)))?;

        // Update history with user turn and the final assistant response
        self.update_history(transcription.text.clone(), final_response_text.clone())
            .await;

        let total_duration_ms = pipeline_start.elapsed().as_millis() as u64;

        info!(
            "✅ STREAMING pipeline complete: {}ms total (STT: {}ms, LLM: {}ms, TTS: {}ms, Playback: {}ms)",
            total_duration_ms, stt_duration_ms, llm_duration_ms, total_tts_ms, playback_duration_ms
        );

        // Create synthetic TtsSynthesis for response (aggregate metrics)
        let tts_synthesis = TtsSynthesis::new(vec![], 24000).with_synthesis_time(total_tts_ms);

        Ok(ChatResponse {
            user_text: transcription.text,
            assistant_text: final_response_text,
            stt_duration_ms,
            llm_response,
            tts_synthesis,
            playback_duration_ms,
            total_duration_ms,
        })
    }

    /// Execute any tool calls found in `raw_text` and return the text that should
    /// be spoken and stored in history.
    ///
    /// If no tool calls are found, returns `raw_text` unchanged.
    /// If tool calls are found, executes them and performs up to
    /// `MAX_AGENTIC_ITERATIONS` additional LLM generate() passes to produce a
    /// natural follow-up response (W8 / W1 fix for streaming path).
    async fn run_tool_calls_and_get_final_text(
        &self,
        raw_text: String,
        base_messages: &[ChatMessage],
    ) -> crate::Result<String> {
        /// Hard cap matching the non-streaming agentic loop.
        const MAX_AGENTIC_ITERATIONS: usize = 5;

        let tool_calls = parse_tool_calls(&raw_text);

        if tool_calls.is_empty() {
            // Nothing to do — return the LLM text as-is.
            return Ok(raw_text);
        }

        info!(
            "🔧 [streaming] Detected {} tool call(s), executing agentic loop...",
            tool_calls.len()
        );

        // Build a mutable copy of the message list that we can extend with
        // Tool result messages for subsequent generate() passes.
        let mut messages: Vec<ChatMessage> = base_messages.to_vec();

        // Append the assistant turn that contained the tool call JSON.
        messages.push(ChatMessage::assistant(raw_text.clone()));

        let mut last_text = raw_text;

        for iteration in 0..MAX_AGENTIC_ITERATIONS {
            let calls = parse_tool_calls(&last_text);
            if calls.is_empty() {
                break;
            }

            debug!(
                "run_tool_calls: iteration {}/{}, {} call(s)",
                iteration + 1,
                MAX_AGENTIC_ITERATIONS,
                calls.len()
            );

            // Execute tools and collect results.
            let tool_registry = self.tool_registry.read().await;
            let mut result_parts: Vec<String> = Vec::new();

            for call in &calls {
                info!("  → [streaming] Executing tool: {}", call.name);
                match tool_registry.execute(call).await {
                    Ok(output) => {
                        info!(
                            "  ✓ [streaming] Tool {} succeeded: {}",
                            call.name, output.message
                        );
                        result_parts
                            .push(format!("Tool `{}` result: {}", call.name, output.message));
                    }
                    Err(e) => {
                        warn!("  ✗ [streaming] Tool {} failed: {}", call.name, e.message);
                        result_parts.push(format!("Tool `{}` error: {}", call.name, e.message));
                    }
                }
            }
            drop(tool_registry);

            // Append Tool result message and generate a follow-up.
            messages.push(ChatMessage::tool(result_parts.join("\n")));

            let follow_up = {
                let mut llm = self.llm_runtime.write().await;
                llm.generate(&messages)
                    .map_err(|e| crate::Error::Model(format!("LLM follow-up failed: {}", e)))?
            };

            last_text = follow_up.text.clone();
            messages.push(ChatMessage::assistant(last_text.clone()));
        }

        Ok(last_text)
    }

    /// Transcribe audio using STT model
    async fn transcribe_audio(&self, audio_data: Vec<f32>) -> crate::Result<Transcription> {
        // We need to handle the sync API in an async context
        // Since ModelRuntime::transcribe is sync, we call it directly
        // The lock ensures thread safety
        let mut stt_model = self.stt_model.write().await;

        // Call synchronous transcribe with 16kHz sample rate
        let result = stt_model.transcribe(&audio_data, 16000)?;

        Ok(result)
    }

    /// Generate LLM response with conversation history.
    ///
    /// Implements a full agentic loop (W8):
    /// 1. Generate LLM response.
    /// 2. Parse tool calls from the response.
    /// 3. Execute every tool call found.
    /// 4. Append the tool result as a `Tool` role message and generate again.
    /// 5. Repeat up to `MAX_AGENTIC_ITERATIONS` times to prevent infinite loops.
    async fn generate_response(&self, user_text: &str) -> crate::Result<LlmResponse> {
        debug!("generate_response: Starting");

        /// Hard cap on how many tool→LLM round-trips we allow per user turn.
        const MAX_AGENTIC_ITERATIONS: usize = 5;

        // Build initial message list once; we'll push Tool messages into it each iteration.
        let mut messages = Vec::new();

        // Build system prompt with tool definitions
        debug!("generate_response: Acquiring system_prompt read lock");
        let base_system_prompt = self.system_prompt.read().await;
        let mut system_prompt = base_system_prompt.clone();
        drop(base_system_prompt);

        // Add tool definitions if tools are enabled
        let tool_registry = self.tool_registry.read().await;
        let definitions = tool_registry.get_definitions();
        if !definitions.is_empty() {
            system_prompt.push_str("\n\n");
            system_prompt.push_str(&format_tool_definitions(&definitions));
        }
        drop(tool_registry);

        messages.push(ChatMessage::system(system_prompt.clone()));
        debug!(
            "generate_response: System prompt added (with {} tools)",
            definitions.len()
        );
        debug!("Full system prompt:\n{}", system_prompt);

        // Add conversation history
        debug!("generate_response: Acquiring history read lock");
        let history = self.history.read().await;
        messages.extend(history.iter().cloned());
        drop(history);
        debug!(
            "generate_response: History added ({} messages total)",
            messages.len()
        );

        // Add current user message
        messages.push(ChatMessage::user(user_text.to_string()));

        debug!("💬 Generating with {} messages in context", messages.len());

        let mut final_response = LlmResponse::new(String::new());

        for iteration in 0..MAX_AGENTIC_ITERATIONS {
            debug!(
                "generate_response: Agentic iteration {}/{}",
                iteration + 1,
                MAX_AGENTIC_ITERATIONS
            );

            // Generate response
            debug!("generate_response: Acquiring LLM write lock");
            let response = {
                let mut llm = self.llm_runtime.write().await;
                debug!("generate_response: LLM lock acquired, calling generate");
                llm.generate(&messages)
                    .map_err(|e| crate::Error::Model(format!("LLM generation failed: {}", e)))?
            };
            debug!("generate_response: LLM generate returned");

            // Check for tool calls in the response (W1 companion — also done in non-streaming path)
            let tool_calls = parse_tool_calls(&response.text);
            debug!("generate_response: Parsed {} tool calls", tool_calls.len());

            if tool_calls.is_empty() {
                // No tools requested — this is the final answer.
                debug!("generate_response: No tool calls, returning final response");
                final_response = response;
                break;
            }

            // Execute all requested tool calls and collect results (W8)
            info!(
                "🔧 Agentic iteration {}: executing {} tool call(s)",
                iteration + 1,
                tool_calls.len()
            );

            let tool_registry = self.tool_registry.read().await;
            let mut result_parts: Vec<String> = Vec::new();

            for call in &tool_calls {
                info!("  → Executing tool: {}", call.name);
                match tool_registry.execute(call).await {
                    Ok(output) => {
                        info!("  ✓ Tool {} succeeded: {}", call.name, output.message);
                        result_parts
                            .push(format!("Tool `{}` result: {}", call.name, output.message));
                    }
                    Err(e) => {
                        warn!("  ✗ Tool {} failed: {}", call.name, e.message);
                        result_parts.push(format!("Tool `{}` error: {}", call.name, e.message));
                    }
                }
            }
            drop(tool_registry);

            // Append the assistant turn (which contained the tool call JSON) so the LLM
            // has a coherent view of what it said, then append the tool results as a
            // `Tool` role message so the LLM can generate a natural follow-up.
            messages.push(ChatMessage::assistant(response.text.clone()));
            messages.push(ChatMessage::tool(result_parts.join("\n")));

            // Remember the last response in case this was the last allowed iteration.
            final_response = response;
        }

        // If we hit the iteration limit without a clean exit, use whatever was generated last.
        if final_response.text.is_empty() {
            warn!("⚠️  Agentic loop produced no usable response");
        }

        // Update history with the final user→assistant pair
        drop(messages); // release borrow before mutating history
        debug!("generate_response: LLM lock dropped, calling update_history");
        self.update_history(user_text.to_string(), final_response.text.clone())
            .await;
        debug!("generate_response: update_history returned");

        debug!("generate_response: Returning response");
        Ok(final_response)
    }

    /// Synthesize speech from text
    async fn synthesize_speech(&self, text: &str) -> crate::Result<TtsSynthesis> {
        debug!("Attempting to acquire TTS write lock for synthesis");
        let mut tts = self.tts_runtime.write().await;
        debug!("TTS write lock acquired, calling synthesize");

        let result = tts
            .synthesize(text)
            .map_err(|e| crate::Error::Model(format!("TTS synthesis failed: {}", e)));

        debug!("Synthesize call returned");
        result
    }

    /// Play synthesized audio
    async fn play_audio(&self, synthesis: &TtsSynthesis) -> crate::Result<()> {
        let player = self.audio_player.write().await;

        player
            .play(&synthesis.samples, synthesis.sample_rate)
            .await
            .map_err(|e| crate::Error::Audio(format!("Audio playback failed: {}", e)))
    }

    /// Update conversation history
    async fn update_history(&self, user_text: String, assistant_text: String) {
        debug!("update_history: Starting");
        debug!("update_history: Acquiring history write lock");
        let mut history = self.history.write().await;
        debug!("update_history: Lock acquired");

        // Add user message
        history.push(ChatMessage::user(user_text));

        // Add assistant message
        history.push(ChatMessage::assistant(assistant_text));

        // W6: Trim history without splitting user/assistant pairs.
        // We always remove the oldest complete pair (2 messages) at once so the
        // history stays coherent (every User turn has a matching Assistant reply).
        // We use saturating arithmetic so this is safe even with an odd count.
        while history.len() > MAX_HISTORY_LENGTH {
            // Remove the oldest two messages (one pair).  If somehow only one
            // message is over the limit, remove just that one to avoid a panic.
            let remove = if history.len() > 1 { 2 } else { 1 };
            history.drain(0..remove);
            debug!(
                "🗑️  Trimmed {} old message(s) from history (pair trim)",
                remove
            );
        }

        debug!("📚 History now has {} messages", history.len());
        debug!("update_history: Returning");
    }

    /// Clear conversation history
    pub async fn clear_history(&self) {
        let mut history = self.history.write().await;
        history.clear();
        info!("🗑️  Conversation history cleared");
    }

    /// Get conversation history
    pub async fn get_history(&self) -> Vec<ChatMessage> {
        let history = self.history.read().await;
        history.clone()
    }

    /// Set system prompt
    pub async fn set_system_prompt(&self, prompt: String) {
        let mut system_prompt = self.system_prompt.write().await;
        *system_prompt = prompt;
        info!("📝 System prompt updated");
    }

    /// Get system prompt
    pub async fn get_system_prompt(&self) -> String {
        let system_prompt = self.system_prompt.read().await;
        system_prompt.clone()
    }

    /// Initialize LLM runtime
    pub async fn init_llm(&self) -> crate::Result<()> {
        info!("🧠 Initializing LLM runtime...");

        let config = self.config.read().await;
        let llm_config = &config.chat.llm;

        let runtime_config = LlmRuntimeConfig {
            model_path: llm_config.model_path.clone(),
            use_gpu: llm_config.device != "cpu",
            context_length: llm_config.context_length,
            temperature: llm_config.temperature,
            max_tokens: llm_config.max_tokens,
            top_p: 0.95,
            top_k: 40,
            repetition_penalty: 1.1,
        };

        let mut llm = self.llm_runtime.write().await;
        llm.load(runtime_config)
            .map_err(|e| crate::Error::Model(format!("Failed to load LLM: {}", e)))?;

        info!("✅ LLM runtime initialized: {}", llm.name());
        Ok(())
    }

    /// Initialize TTS runtime
    pub async fn init_tts(&self) -> crate::Result<()> {
        info!("🔊 Initializing TTS runtime...");

        let config = self.config.read().await;
        let tts_config = &config.chat.tts;

        let runtime_config = TtsRuntimeConfig {
            model_path: tts_config.model_path.clone(),
            use_gpu: tts_config.device != "cpu",
            voice_id: tts_config.voice_id.clone(),
            speech_rate: tts_config.speech_rate,
            pitch: 0.0,
            volume: 1.0,
        };

        let mut tts = self.tts_runtime.write().await;
        tts.load(runtime_config)
            .map_err(|e| crate::Error::Model(format!("Failed to load TTS: {}", e)))?;

        info!("✅ TTS runtime initialized: {}", tts.name());
        Ok(())
    }

    /// Check if chat is ready (all components loaded)
    pub async fn is_ready(&self) -> bool {
        let stt = self.stt_model.read().await;
        let llm = self.llm_runtime.read().await;
        let tts = self.tts_runtime.read().await;

        stt.is_loaded() && llm.is_loaded() && tts.is_loaded()
    }

    /// Get chat engine status
    pub async fn status(&self) -> ChatEngineStatus {
        let stt = self.stt_model.read().await;
        let llm = self.llm_runtime.read().await;
        let tts = self.tts_runtime.read().await;
        let history = self.history.read().await;

        ChatEngineStatus {
            stt_loaded: stt.is_loaded(),
            stt_name: stt.name().to_string(),
            llm_loaded: llm.is_loaded(),
            llm_name: llm.name().to_string(),
            tts_loaded: tts.is_loaded(),
            tts_name: tts.name().to_string(),
            history_length: history.len(),
            ready: stt.is_loaded() && llm.is_loaded() && tts.is_loaded(),
        }
    }

    /// List available TTS voices
    pub async fn list_voices(&self) -> crate::Result<Vec<crate::models::tts_runtime::VoiceInfo>> {
        let tts = self.tts_runtime.read().await;

        if !tts.is_loaded() {
            return Err(crate::Error::Model("TTS not loaded".to_string()));
        }

        Ok(tts.list_voices())
    }

    /// Set TTS voice
    pub async fn set_voice(&self, voice_id: &str) -> crate::Result<()> {
        let mut tts = self.tts_runtime.write().await;

        if !tts.is_loaded() {
            return Err(crate::Error::Model("TTS not loaded".to_string()));
        }

        tts.set_voice(voice_id)
            .map_err(|e| crate::Error::Model(format!("Failed to set voice: {}", e)))?;

        info!("🎤 Voice changed to: {}", voice_id);
        Ok(())
    }

    /// Unload all models (for cleanup)
    pub async fn unload(&self) {
        info!("🔄 Unloading chat engine models...");

        let mut llm = self.llm_runtime.write().await;
        llm.unload();

        let mut tts = self.tts_runtime.write().await;
        tts.unload();

        info!("✅ Chat engine models unloaded");
    }
}

/// Chat engine status
#[derive(Debug, Clone)]
pub struct ChatEngineStatus {
    pub stt_loaded: bool,
    pub stt_name: String,
    pub llm_loaded: bool,
    pub llm_name: String,
    pub tts_loaded: bool,
    pub tts_name: String,
    pub history_length: usize,
    pub ready: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{MockModel, llm_mock::MockLlm, tts_mock::MockTts};

    async fn create_test_engine() -> ChatEngine {
        let config = Arc::new(RwLock::new(Config::default()));

        let stt: Box<dyn ModelRuntime> = Box::new(MockModel::new());
        let stt_model = Arc::new(RwLock::new(stt));

        let llm: Box<dyn LlmRuntime> = Box::new(MockLlm::new());
        let llm_runtime = Arc::new(RwLock::new(llm));

        let tts: Box<dyn TtsRuntime> = Box::new(MockTts::new());
        let tts_runtime = Arc::new(RwLock::new(tts));

        ChatEngine::new(config, stt_model, llm_runtime, tts_runtime).unwrap()
    }

    #[tokio::test]
    async fn test_create_engine() {
        let engine = create_test_engine().await;
        assert!(!engine.is_ready().await); // Models not loaded yet
    }

    #[tokio::test]
    async fn test_init_llm() {
        let engine = create_test_engine().await;
        let result = engine.init_llm().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_init_tts() {
        let engine = create_test_engine().await;
        let result = engine.init_tts().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_history_management() {
        let engine = create_test_engine().await;

        // Initially empty
        let history = engine.get_history().await;
        assert_eq!(history.len(), 0);

        // Add messages
        engine
            .update_history("Hello".to_string(), "Hi there!".to_string())
            .await;
        let history = engine.get_history().await;
        assert_eq!(history.len(), 2);

        // Clear history
        engine.clear_history().await;
        let history = engine.get_history().await;
        assert_eq!(history.len(), 0);
    }

    #[tokio::test]
    async fn test_system_prompt() {
        let engine = create_test_engine().await;

        let prompt = engine.get_system_prompt().await;
        assert!(!prompt.is_empty());

        engine.set_system_prompt("Custom prompt".to_string()).await;
        let prompt = engine.get_system_prompt().await;
        assert_eq!(prompt, "Custom prompt");
    }

    #[tokio::test]
    async fn test_status() {
        let engine = create_test_engine().await;
        let status = engine.status().await;

        assert!(!status.ready); // Models not loaded
        assert_eq!(status.history_length, 0);
        assert!(!status.stt_loaded);
        assert!(!status.llm_loaded);
        assert!(!status.tts_loaded);
    }

    #[tokio::test]
    async fn test_history_trimming() {
        let engine = create_test_engine().await;

        // Add more than MAX_HISTORY_LENGTH messages
        for i in 0..15 {
            engine
                .update_history(
                    format!("User message {}", i),
                    format!("Assistant message {}", i),
                )
                .await;
        }

        let history = engine.get_history().await;
        assert_eq!(history.len(), MAX_HISTORY_LENGTH);
    }
}
