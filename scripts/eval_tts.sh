#!/bin/bash
# Evaluate all TTS providers on the same video for A/B comparison.
# Usage: ./scripts/eval_tts.sh '<youtube-url>'

set -euo pipefail

URL="${1:?Usage: eval_tts.sh '<youtube-url>'}"

PROVIDERS=("edge-tts" "fish-speech" "qwen3-tts" "chatterbox")

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  vdub TTS Provider Evaluation"
echo "  URL: $URL"
echo "  Providers: ${PROVIDERS[*]}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

mkdir -p ./eval

for provider in "${PROVIDERS[@]}"; do
    echo ""
    echo "▶ Testing: $provider"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    outdir="./eval/$provider"
    mkdir -p "$outdir"

    start=$(date +%s)

    cargo run --release --bin vdub -- "$URL" \
        --no-embed \
        --tts-provider "$provider" \
        2>&1 | tee "$outdir/run.log"

    end=$(date +%s)
    elapsed=$((end - start))

    echo "$provider: ${elapsed}s" >> ./eval/timing.txt
    echo "✓ $provider complete in ${elapsed}s"
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Results in ./eval/"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
cat ./eval/timing.txt
