"""Qwen3-TTS wrapper for vdub. Uses mlx-audio for Apple Silicon inference.

Usage:
    python qwen3_tts.py --model MODEL --text-file INPUT.txt --output OUTPUT.wav \
        [--ref-audio REF.wav] [--ref-text "reference transcript"] \
        [--lang russian] [--temperature 0.9] [--top-k 50] [--repetition-penalty 1.05]
"""

import argparse
import sys

def main():
    parser = argparse.ArgumentParser(description="Qwen3-TTS via mlx-audio")
    parser.add_argument("--model", required=True, help="HuggingFace model ID")
    parser.add_argument("--text-file", required=True, help="Path to text file")
    parser.add_argument("--output", required=True, help="Output WAV path")
    parser.add_argument("--ref-audio", default=None, help="Reference audio for voice cloning")
    parser.add_argument("--ref-text", default=None, help="Transcript of reference audio")
    parser.add_argument("--lang", default="russian", help="Target language name")
    parser.add_argument("--temperature", type=float, default=0.9)
    parser.add_argument("--top-k", type=int, default=50)
    parser.add_argument("--top-p", type=float, default=1.0)
    parser.add_argument("--repetition-penalty", type=float, default=1.05)
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
        "temperature": args.temperature,
        "top_k": args.top_k,
        "top_p": args.top_p,
        "repetition_penalty": args.repetition_penalty,
        "lang_code": args.lang,
    }
    if args.ref_audio:
        kwargs["ref_audio"] = args.ref_audio
        if args.ref_text:
            kwargs["ref_text"] = args.ref_text

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
