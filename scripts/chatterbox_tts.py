"""Chatterbox TTS wrapper for vdub. MIT licensed, emotion exaggeration control.

Usage:
    python chatterbox_tts.py --text-file INPUT.txt --output OUTPUT.wav \
        [--ref-audio REF.wav] [--exaggeration 0.5]
"""

import argparse
import sys

def main():
    parser = argparse.ArgumentParser(description="Chatterbox TTS")
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
        from chatterbox.tts import ChatterboxTTS
        import torch
        import soundfile as sf
    except ImportError:
        print("Error: chatterbox-tts not installed. Run: uv pip install chatterbox-tts", file=sys.stderr)
        sys.exit(1)

    device = "mps" if torch.backends.mps.is_available() else "cpu"
    model = ChatterboxTTS.from_pretrained(device=device)

    kwargs = {
        "text": text,
        "exaggeration": args.exaggeration,
    }
    if args.ref_audio:
        kwargs["audio_prompt_path"] = args.ref_audio

    audio = model.generate(**kwargs)

    if isinstance(audio, torch.Tensor):
        audio = audio.squeeze().cpu().numpy()

    sf.write(args.output, audio, model.sr)


if __name__ == "__main__":
    main()
