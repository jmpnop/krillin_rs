# vdub

Video dubbing and subtitle engine written in Rust. Automatically transcribes, translates, and dubs videos with support for fully on-device processing on Apple Silicon via MLX. All ASR and TTS providers are free and local.

## Features

- **5-stage pipeline**: Download → Transcribe → Translate → TTS → Embed subtitles
- **Auto language detection**: Whisper detects the source language; EN↔RU translation selected automatically
- **Multi-track audio**: Dubbed audio added as a second track with language metadata (original preserved)
- **yt-dlp integration**: Paste a YouTube URL, get a dubbed `.mp4` back
- **Free ASR backends**: faster-whisper, whisper.cpp, WhisperKit, MLX Whisper
- **Voice cloning**: Clone the original speaker's voice from reference audio (Qwen3-TTS, Fish Speech, Chatterbox)
- **Emotion matching**: LLM-based emotion detection per subtitle, injected as inline tags for TTS providers that support them
- **5 TTS backends**: Edge TTS, MLX Audio (Kokoro), Qwen3-TTS, Fish Speech S2 Pro, Chatterbox
- **Any OpenAI-compatible LLM**: OpenAI, DeepSeek, local LM Studio / `mlx_lm.server`
- **On-device Apple Silicon**: MLX Whisper + Qwen3-TTS + local LLM = zero cloud dependencies
- **Two binaries**: `vdub` (CLI) and `vdubd` (web server)

## Requirements

- Rust 1.75+
- macOS with Homebrew (dependencies are auto-installed on first run)

All other dependencies (ffmpeg, yt-dlp, ASR/TTS backends) are **automatically installed** on startup based on your config. Python packages are managed via [uv](https://docs.astral.sh/uv/) in an isolated venv at `./venv/`.

## Quick Start

```bash
# Build
git clone https://github.com/jmpnop/vdub.git
cd vdub
cargo build --release

# Set up config
cp config/config-example.toml config/config.toml
# Edit config/config.toml — set your LLM API key
```

### CLI — dub a video in one command

```bash
# Just a URL — auto-detects language, dubs, embeds subtitles
vdub https://youtube.com/watch?v=VIDEO_ID

# Specify languages
vdub https://youtube.com/watch?v=VIDEO_ID --from en --to ru

# Subtitles only, no dubbing
vdub https://youtube.com/watch?v=VIDEO_ID --no-tts

# Local file
vdub local:./my_video.mp4

# Voice-cloning TTS with emotion
vdub https://youtube.com/watch?v=VIDEO_ID --tts-provider qwen3-tts

# A/B test providers
vdub https://youtube.com/watch?v=VIDEO_ID --tts-provider fish-speech
vdub https://youtube.com/watch?v=VIDEO_ID --tts-provider chatterbox

# All options
vdub https://youtube.com/watch?v=VIDEO_ID \
  --from en --to ru \
  --tts-provider qwen3-tts \
  --no-bilingual \
  --replace-audio \
  --vertical
```

#### CLI flags

| Flag | Description |
|------|-------------|
| `--from`, `-f` | Source language (auto-detected if omitted) |
| `--to`, `-t` | Target language (auto-selected EN↔RU if omitted) |
| `--tts-provider` | Override TTS provider (`edge-tts`, `mlx-audio`, `fish-speech`, `qwen3-tts`, `chatterbox`) |
| `--no-tts` | Subtitles only, skip dubbing |
| `--no-embed` | Skip subtitle embedding into video |
| `--no-voice-clone` | Disable voice cloning, use default TTS voice |
| `--no-emotion` | Disable emotion detection for TTS |
| `--voice` | TTS voice code |
| `--no-bilingual` | Target language subtitles only |
| `--replace-audio` | Replace original audio instead of adding second track |
| `--vertical` | Also generate vertical (9:16) video |

### Web server

```bash
vdubd
```

Open `http://127.0.0.1:8888` in your browser.

### What gets installed automatically

| Tool | Install method | When |
|------|---------------|------|
| uv | curl installer | Always (Python package manager) |
| ffmpeg / ffprobe | Homebrew | Always |
| yt-dlp | Homebrew | Always |
| faster-whisper | uv (venv) | When `transcribe.provider = "fasterwhisper"` |
| whisper.cpp | Homebrew | When `transcribe.provider = "whispercpp"` |
| WhisperKit | Homebrew | When `transcribe.provider = "whisperkit"` |
| mlx-whisper | uv (venv) | When `transcribe.provider = "mlx-whisper"` |
| edge-tts | uv (venv) | When `tts.provider = "edge-tts"` |
| mlx-audio | uv (venv) | When `tts.provider = "mlx-audio"`, `"fish-speech"`, `"qwen3-tts"`, or `"chatterbox"` |

## Apple Silicon (fully local)

Run the entire pipeline on-device with zero cloud dependencies. Qwen3-TTS provides voice cloning + emotion matching via MLX:

```bash
# Start local LLM (LM Studio recommended, or mlx_lm.server)
# LM Studio: download Qwen2.5-72B-Instruct-abliterated, start server on port 1234
```

Set in `config/config.toml`:

```toml
[llm]
base_url = "http://localhost:1234/v1"
api_key = "not-needed"
model = "qwen2.5-72b-instruct-abliterated"

[transcribe]
provider = "mlx-whisper"

[tts]
provider = "qwen3-tts"
```

This runs transcription (MLX Whisper), translation (local LLM), and voice-cloned TTS (Qwen3-TTS) entirely on-device using Metal GPU.

## API (vdubd)

The server accepts JSON, form-urlencoded, or just a `?url=` query param:

```bash
# Simplest — just a URL
curl -d "url=https://youtube.com/watch?v=..." localhost:8888/api/capability/subtitleTask

# Or as query param
curl "localhost:8888/api/capability/subtitleTask?url=https://youtube.com/watch?v=..."

# Full JSON (all fields optional except url)
curl localhost:8888/api/capability/subtitleTask \
  -H "Content-Type: application/json" \
  -d '{"url": "https://youtube.com/watch?v=..."}'
```

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/capability/subtitleTask` | POST | Start a dubbing task |
| `/api/capability/subtitleTask?taskId=...` | GET | Get task status |
| `/api/file` | POST | Upload a file |
| `/api/file/*` | GET | Download output files |
| `/api/config` | GET/POST | View or update config (hot-reload) |

## Providers

### Transcription (ASR) — all free, local
| Provider | Config value | Notes |
|----------|-------------|-------|
| faster-whisper | `fasterwhisper` | Local, Python, default |
| whisper.cpp | `whispercpp` | Local, C++ |
| WhisperKit | `whisperkit` | macOS only, CoreML |
| MLX Whisper | `mlx-whisper` | macOS only, Metal GPU |

### Text-to-Speech — all free, local
| Provider | Config value | Voice Clone | Emotion | Russian | Notes |
|----------|-------------|:-----------:|:-------:|:-------:|-------|
| Edge TTS | `edge-tts` | | | yes | Free Microsoft voices, default |
| MLX Audio (Kokoro) | `mlx-audio` | | | | macOS, 82M params, fast |
| Qwen3-TTS | `qwen3-tts` | yes | implicit | tier 1 | macOS MLX native, 1.7B, recommended for dubbing |
| Fish Speech S2 Pro | `fish-speech` | yes | 15K+ tags | tier 2 | macOS MLX, 6.72 GB 8-bit |
| Chatterbox | `chatterbox` | yes | exaggeration | yes | MIT license, macOS MLX, 23 languages |

### Translation LLM
Any OpenAI-compatible API. Point `llm.base_url` at your provider.

## Project Structure

```
src/
  bin/          # vdub (CLI) and vdubd (web server)
  config/       # TOML config with provider enums
  dto/          # API request/response types
  handler/      # Axum HTTP handlers
  provider/     # ASR, TTS, LLM provider implementations
    openai/     # OpenAI-compatible Chat (for translation LLM)
    local/      # whisper.cpp, WhisperKit, faster-whisper, edge-tts,
                # MLX Whisper, MLX Audio, Qwen3-TTS, Fish Speech, Chatterbox
  service/      # Pipeline steps (split, transcribe, translate, TTS, embed)
  storage/      # Task store, binary path detection
  types/        # Subtitles, ASS headers, prompts, language maps
  util/         # ffmpeg/ffprobe wrappers, dependency management, CLI art,
                # voice extraction, emotion detection
scripts/        # Python TTS wrappers (qwen3_tts.py, fish_speech_tts.py, chatterbox_tts.py)
static/         # Embedded web UI
config/         # Example configuration
```

## License

MIT
