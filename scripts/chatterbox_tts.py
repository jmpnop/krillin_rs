"""Chatterbox TTS wrapper for vdub via mlx-audio (Apple Silicon native).

MIT licensed, emotion exaggeration control, 23 languages incl. Russian.
Uses mlx-audio's MLX port — runs on Metal GPU, no PyTorch needed.

Usage:
    python chatterbox_tts.py --model MODEL --text-file INPUT.txt --output OUTPUT.wav \
        [--ref-audio REF.wav] [--exaggeration 0.5]
"""

import argparse
import sys

def main():
    parser = argparse.ArgumentParser(description="Chatterbox TTS via mlx-audio")
    parser.add_argument("--model", default="mlx-community/chatterbox-fp16",
                        help="HuggingFace model ID")
    parser.add_argument("--text-file", required=True, help="Path to text file")
    parser.add_argument("--output", required=True, help="Output WAV path")
    parser.add_argument("--ref-audio", default=None, help="Reference audio for voice cloning")
    parser.add_argument("--exaggeration", type=float, default=0.5,
                        help="Emotion exaggeration (0.0=monotone, 1.0=dramatic)")
    args = parser.parse_args()

    text = open(args.text_file, encoding="utf-8").read().strip()
    if not text:
        print("Error: empty text file", file=sys.stderr)
        sys.exit(1)

    try:
        from mlx_audio.tts.utils import load_model
        import mlx.core as mx
        import soundfile as sf
    except ImportError:
        print("Error: mlx-audio not installed. Run: uv pip install mlx-audio", file=sys.stderr)
        sys.exit(1)

    model = load_model(args.model)

    kwargs = {
        "text": text,
        "exaggeration": args.exaggeration,
    }
    if args.ref_audio:
        kwargs["ref_audio"] = args.ref_audio

    results = list(model.generate(**kwargs))
    audio_chunks = [r.audio for r in results]
    if len(audio_chunks) == 1:
        audio = audio_chunks[0]
    else:
        audio = mx.concatenate(audio_chunks)

    audio_np = audio.squeeze()
    if hasattr(audio_np, "tolist"):
        import numpy as np
        audio_np = np.array(audio_np.tolist(), dtype=np.float32)

    sf.write(args.output, audio_np, 24000)


if __name__ == "__main__":
    main()
