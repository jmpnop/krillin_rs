#!/usr/bin/env python3
"""CLI wrapper for faster-whisper — called by krillin_rs."""
import argparse, json, sys
from pathlib import Path

def main():
    p = argparse.ArgumentParser()
    p.add_argument("audio", help="Audio file to transcribe")
    p.add_argument("--model", default="medium")
    p.add_argument("--model_dir", default="./models/")
    p.add_argument("--language", default=None)
    p.add_argument("--output_dir", required=True)
    p.add_argument("--output_format", default="json")
    p.add_argument("--compute_type", default="int8")
    p.add_argument("--one_word", type=int, default=2)
    args = p.parse_args()

    from faster_whisper import WhisperModel

    model = WhisperModel(
        args.model,
        download_root=args.model_dir,
        compute_type=args.compute_type,
    )

    segments, info = model.transcribe(
        args.audio,
        language=args.language,
        word_timestamps=True,
    )

    result_segments = []
    full_text = ""
    for seg in segments:
        words = []
        if seg.words:
            for w in seg.words:
                words.append({"start": w.start, "end": w.end, "word": w.word})
        result_segments.append({"text": seg.text, "words": words})
        full_text += seg.text

    output = {
        "text": full_text,
        "language": info.language,
        "segments": result_segments,
    }

    audio_path = Path(args.audio)
    out_path = Path(args.output_dir) / f"{audio_path.stem}.json"
    out_path.write_text(json.dumps(output, ensure_ascii=False, indent=2))

if __name__ == "__main__":
    main()
