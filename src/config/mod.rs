use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub app: AppConfig,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub llm: OpenaiCompatibleConfig,
    #[serde(default)]
    pub transcribe: TranscribeConfig,
    #[serde(default)]
    pub tts: TtsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_segment_duration")]
    pub segment_duration: u32,
    #[serde(default = "default_transcribe_parallel")]
    pub transcribe_parallel_num: u32,
    #[serde(default = "default_translate_parallel")]
    pub translate_parallel_num: u32,
    #[serde(default = "default_max_attempts")]
    pub transcribe_max_attempts: u32,
    #[serde(default = "default_max_attempts")]
    pub translate_max_attempts: u32,
    #[serde(default = "default_max_sentence_length")]
    pub max_sentence_length: u32,
    #[serde(default)]
    pub proxy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenaiCompatibleConfig {
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocalModelConfig {
    #[serde(default)]
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AliyunSpeechConfig {
    #[serde(default)]
    pub access_key_id: String,
    #[serde(default)]
    pub access_key_secret: String,
    #[serde(default)]
    pub app_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AliyunOssConfig {
    #[serde(default)]
    pub access_key_id: String,
    #[serde(default)]
    pub access_key_secret: String,
    #[serde(default)]
    pub bucket: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AliyunTranscribeConfig {
    #[serde(default)]
    pub oss: AliyunOssConfig,
    #[serde(default)]
    pub speech: AliyunSpeechConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscribeConfig {
    #[serde(default = "default_transcribe_provider")]
    pub provider: String,
    #[serde(default)]
    pub enable_gpu_acceleration: bool,
    #[serde(default = "default_openai_transcribe")]
    pub openai: OpenaiCompatibleConfig,
    #[serde(default = "default_local_model")]
    pub fasterwhisper: LocalModelConfig,
    #[serde(default = "default_local_model")]
    pub whisperkit: LocalModelConfig,
    #[serde(default = "default_local_model")]
    pub whispercpp: LocalModelConfig,
    #[serde(default)]
    pub aliyun: AliyunTranscribeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AliyunTtsConfig {
    #[serde(default)]
    pub oss: AliyunOssConfig,
    #[serde(default)]
    pub speech: AliyunSpeechConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsConfig {
    #[serde(default = "default_tts_provider")]
    pub provider: String,
    #[serde(default = "default_openai_tts")]
    pub openai: OpenaiCompatibleConfig,
    #[serde(default)]
    pub aliyun: AliyunTtsConfig,
}

// Default value functions
fn default_segment_duration() -> u32 { 5 }
fn default_transcribe_parallel() -> u32 { 1 }
fn default_translate_parallel() -> u32 { 3 }
fn default_max_attempts() -> u32 { 3 }
fn default_max_sentence_length() -> u32 { 70 }
fn default_host() -> String { "127.0.0.1".to_string() }
fn default_port() -> u16 { 8888 }
fn default_transcribe_provider() -> String { "openai".to_string() }
fn default_tts_provider() -> String { "openai".to_string() }

fn default_openai_transcribe() -> OpenaiCompatibleConfig {
    OpenaiCompatibleConfig {
        model: "whisper-1".to_string(),
        ..Default::default()
    }
}

fn default_local_model() -> LocalModelConfig {
    LocalModelConfig {
        model: "large-v2".to_string(),
    }
}

fn default_openai_tts() -> OpenaiCompatibleConfig {
    OpenaiCompatibleConfig {
        model: "gpt-4o-mini-tts".to_string(),
        ..Default::default()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app: AppConfig::default(),
            server: ServerConfig::default(),
            llm: OpenaiCompatibleConfig {
                model: "gpt-4o-mini".to_string(),
                ..Default::default()
            },
            transcribe: TranscribeConfig::default(),
            tts: TtsConfig::default(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            segment_duration: default_segment_duration(),
            transcribe_parallel_num: default_transcribe_parallel(),
            translate_parallel_num: default_translate_parallel(),
            transcribe_max_attempts: default_max_attempts(),
            translate_max_attempts: default_max_attempts(),
            max_sentence_length: default_max_sentence_length(),
            proxy: String::new(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
        }
    }
}

impl Default for TranscribeConfig {
    fn default() -> Self {
        Self {
            provider: default_transcribe_provider(),
            enable_gpu_acceleration: false,
            openai: default_openai_transcribe(),
            fasterwhisper: default_local_model(),
            whisperkit: default_local_model(),
            whispercpp: default_local_model(),
            aliyun: AliyunTranscribeConfig::default(),
        }
    }
}

impl Default for TtsConfig {
    fn default() -> Self {
        Self {
            provider: default_tts_provider(),
            openai: default_openai_tts(),
            aliyun: AliyunTtsConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_path = "./config/config.toml";
        if Path::new(config_path).exists() {
            let content = fs::read_to_string(config_path)?;
            let config: Config = toml::from_str(&content)?;
            tracing::info!("Configuration loaded from {}", config_path);
            Ok(config)
        } else {
            tracing::info!("No config file found, using defaults");
            Ok(Config::default())
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = "./config/config.toml";
        let dir = Path::new(config_path).parent().unwrap();
        fs::create_dir_all(dir)?;
        let content = toml::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        tracing::info!("Configuration saved to {}", config_path);
        Ok(())
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        match self.transcribe.provider.as_str() {
            "openai" => {
                if self.transcribe.openai.api_key.is_empty() {
                    anyhow::bail!("OpenAI transcription requires an API key");
                }
            }
            "fasterwhisper" => {
                let model = &self.transcribe.fasterwhisper.model;
                if !["tiny", "medium", "large-v2"].contains(&model.as_str()) {
                    anyhow::bail!("Invalid fasterwhisper model: {model}. Must be tiny, medium, or large-v2");
                }
            }
            "whisperkit" => {
                #[cfg(not(target_os = "macos"))]
                anyhow::bail!("WhisperKit is only supported on macOS");
            }
            "whispercpp" => {}
            "aliyun" => {
                let speech = &self.transcribe.aliyun.speech;
                if speech.access_key_id.is_empty()
                    || speech.access_key_secret.is_empty()
                    || speech.app_key.is_empty()
                {
                    anyhow::bail!("Aliyun transcription requires access_key_id, access_key_secret, and app_key");
                }
            }
            other => anyhow::bail!("Unsupported transcription provider: {other}"),
        }
        Ok(())
    }
}
