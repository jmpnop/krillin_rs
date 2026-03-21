"""Fish Speech S2 Pro TTS wrapper for vdub.

Usage:
    python fish_speech_tts.py --model MODEL --text-file INPUT.txt --output OUTPUT.wav [--ref-audio REF.wav]
"""

import argparse
import sys

def main():
    parser = argparse.ArgumentParser(description="Fish Speech S2 Pro TTS")
    parser.add_argument("--model", required=True, help="HuggingFace model ID")
    parser.add_argument("--text-file", required=True, help="Path to text file")
    parser.add_argument("--ref-audio", default=None, help="Reference audio for voice cloning")
    parser.add_argument("--output", required=True, help="Output WAV path")
    args = parser.parse_args()

    text = open(args.text_file, encoding="utf-8").read().strip()
    if not text:
        print("Error: empty text file", file=sys.stderr)
        sys.exit(1)

    try:
        from mlx_audio.tts import generate_audio
    except ImportError:
        print("Error: mlx-audio not installed. Run: uv pip install mlx-audio", file=sys.stderr)
        sys.exit(1)

    kwargs = {
        "model": args.model,
        "text": text,
        "output": args.output,
    }
    if args.ref_audio:
        kwargs["ref_audio"] = args.ref_audio

    generate_audio(**kwargs)


if __name__ == "__main__":
    main()
