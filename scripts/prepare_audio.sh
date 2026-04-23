#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$PROJECT_DIR/assets/audio/intro_music.mp3"
DST="$PROJECT_DIR/assets/audio/intro_music.wav"

if [ ! -f "$SRC" ]; then
  echo "Audio prep: source file not found: $SRC" >&2
  exit 1
fi

# Skip conversion when WAV already exists and is newer than source.
if [ -f "$DST" ] && [ "$DST" -nt "$SRC" ]; then
  echo "Audio prep: intro WAV is up to date."
  exit 0
fi

echo "Audio prep: generating intro WAV from MP3..."

if command -v ffmpeg >/dev/null 2>&1; then
  ffmpeg -y -loglevel error -i "$SRC" -acodec pcm_s16le -ar 44100 -ac 2 "$DST"
  echo "Audio prep: generated with ffmpeg."
  exit 0
fi

if command -v afconvert >/dev/null 2>&1; then
  if afconvert "$SRC" "$DST" -f WAVE -d LEI16 >/dev/null 2>&1; then
    echo "Audio prep: generated with afconvert."
    exit 0
  fi
fi

if [ -f "$DST" ]; then
  echo "Audio prep: conversion tool unavailable, keeping existing WAV."
  exit 0
fi

echo "Audio prep: failed to generate intro WAV (need ffmpeg or afconvert)." >&2
exit 1
