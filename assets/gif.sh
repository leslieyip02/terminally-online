#!/bin/bash

INPUT=$1
OUTPUT=$2
PALETTE="palette.png"

# converts a given input video to a gif using ffmpeg
ffmpeg -i "$INPUT" -vf "fps=15,scale=480:-1:flags=lanczos,palettegen" -y "$PALETTE"
ffmpeg -i "$INPUT" -i $PALETTE -filter_complex "fps=15,scale=480:-1:flags=lanczos[x];[x][1:v]paletteuse" -y "$OUTPUT"

rm $PALETTE
