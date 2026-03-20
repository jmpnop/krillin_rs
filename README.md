# KrillinAI

Video dubbing and subtitle engine written in Rust. Automatically transcribes, translates, and dubs videos with support for fully on-device processing on Apple Silicon via MLX.

## Features

- **5-stage pipeline**: Download → Transcribe → Translate → TTS → Embed subtitles
- **Auto language detection**: Whisper detects the source language; EN↔RU translation selected automatically
- **Multi-track audio**: Dubbed audio added as a second track with language metadata (original preserved)
- **yt-dlp integration**: Paste a YouTube URL, get a dubbed `.mp4` back
- **Multiple ASR backends**: OpenAI Whisper API, whisper.cpp, WhisperKit, faster-whisper, MLX Whisper, Aliyun
- **Multiple TTS backends**: OpenAI TTS, Edge TTS, MLX Audio (Kokoro), Aliyun
- **Any OpenAI-compatible LLM**: OpenAI, DeepSeek, local `mlx_lm.server`
- **On-device Apple Silicon**: MLX Whisper + MLX Audio + local LLM = zero cloud dependencies
- **Web UI**: Built-in browser interface at `http://localhost:8888`
- **Hot-reloadable config**: Update settings via API without restarting

## Requirements

- Rust 1.75+
- ffmpeg / ffprobe
- yt-dlp (for URL downloads)

Optional (for on-device processing on macOS):
- `pip install mlx-whisper mlx-audio mlx-lm`

## Quick Start

```bash
# Clone and build
git clone https://github.com/jmpnop/krillin_rs.git
cd krillin_rs
cargo build --release

# Copy and edit config
cp config/config-example.toml config/config.toml
# Edit config/config.toml with your API keys or MLX settings

# Run
./target/release/krillin_rs
```

Open `http://127.0.0.1:8888` in your browser.

## Apple Silicon (fully local)

Run the entire pipeline on-device with zero cloud dependencies:

```bash
# Install MLX tools
pip install mlx-whisper mlx-audio mlx-lm

# Start local LLM
mlx_lm.server --model mlx-community/Qwen2.5-7B-Instruct-4bit --port 8080
```

Set in `config/config.toml`:

```toml
[llm]
base_url = "http://localhost:8080/v1"
api_key = "not-needed"
model = "mlx-community/Qwen2.5-7B-Instruct-4bit"

[transcribe]
provider = "mlx-whisper"

[tts]
provider = "mlx-audio"
```

## API

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/capability/subtitleTask` | POST | Start a dubbing task |
| `/api/capability/subtitleTask?taskId=...` | GET | Get task status |
| `/api/file` | POST | Upload a file |
| `/api/file/*` | GET | Download output files |
| `/api/config` | GET | Get current config |
| `/api/config` | POST | Update config (hot-reload) |

### Start a task

```json
POST /api/capability/subtitleTask
{
  "link": "https://youtube.com/watch?v=...",
  "origin_language": "",
  "target_language": "",
  "subtitle_result_type": 3,
  "enable_tts": true,
  "tts_voice_code": "alloy",
  "multi_track": true
}
```

Leave `origin_language` empty for auto-detection. Leave `target_language` empty to auto-select (EN→RU, RU→EN, other→EN).

## Providers

### Transcription (ASR)
| Provider | Config value | Notes |
|----------|-------------|-------|
| OpenAI Whisper API | `openai` | Cloud, requires API key |
| faster-whisper | `fasterwhisper` | Local, Python |
| whisper.cpp | `whispercpp` | Local, C++ |
| WhisperKit | `whisperkit` | macOS only, CoreML |
| MLX Whisper | `mlx-whisper` | macOS only, Metal GPU |
| Aliyun | `aliyun` | Cloud, Chinese market |

### Text-to-Speech
| Provider | Config value | Notes |
|----------|-------------|-------|
| OpenAI TTS | `openai` | Cloud, requires API key |
| Edge TTS | `edge-tts` | Free, Microsoft voices |
| MLX Audio (Kokoro) | `mlx-audio` | macOS only, 82M params |
| Aliyun | `aliyun` | Cloud, Chinese market |

### Translation LLM
Any OpenAI-compatible API. Point `llm.base_url` at your provider.

## Project Structure

```
src/
  config/       # TOML config with provider enums
  dto/          # API request/response types
  handler/      # Axum HTTP handlers
  provider/     # ASR, TTS, LLM provider implementations
    openai/     # OpenAI Whisper, TTS, Chat
    aliyun/     # Aliyun ASR, TTS, OSS
    local/      # whisper.cpp, WhisperKit, faster-whisper,
                # edge-tts, MLX Whisper, MLX Audio
  service/      # Pipeline steps (split, transcribe, translate, TTS, embed)
  storage/      # Task store, binary path detection
  types/        # Subtitles, ASS headers, prompts, language maps
  util/         # ffmpeg/ffprobe wrappers, text processing, CLI art
static/         # Embedded web UI
config/         # Example configuration
```

## License

MIT
