#!/usr/bin/env bash


set -ex

# This script is used to run studio-headless

#parse arguments from command line
# usage: runner.sh -i input_file -v output.mp4 -o output_dir.zip -f 1000 -b path/to/studio-headless

while getopts "i:o:f:c:s:b:" opt; do
  case $opt in
    i) INPUT_FILE=$OPTARG ;;
    o) OUTPUT_DIR=$OPTARG ;;
    f) FRAME_COUNT=$OPTARG ;;
    c) STUDIO_CAMERA_SETTING=$OPTARG ;;
    s) STUDIO_CAMERA_SPEED=$OPTARG ;;
    b) SKYBOX=$OPTARG ;;
    \?) echo "Invalid option -$OPTARG" >&2
    ;;
  esac
done

# error out if input file is not provided
if [ -z "$INPUT_FILE" ]; then
  echo "Input file is required"
  exit 1
fi

# error out if output directory is not provided
if [ -z "$OUTPUT_DIR" ]; then
  echo "Output directory is required"
  exit 1
fi

#source secrets.env
source /home/ubuntu/secrets.env

OUTPUT_DIR=${OUTPUT_DIR:-/tmp/studio-headless}
FPS=${FPS:-24}
RESOLUTION=${RESOLUTION:-1920x1080}
FRAME_COUNT=${FRAME_COUNT:-100}
STUDIO_HEADLESS_PATH=${STUDIO_HEADLESS_PATH:-/home/ubuntu/storyteller-bevy/target/release/studio-headless}
MAX_FRAME_COUNT=240 # 10 seconds @ 24 fps
STUDIO_CAMERA_SPEED=${STUDIO_CAMERA_SPEED:-1}
SKYBOX=${SKYBOX:-ff5555}
STUDIO_TIMEOUT_SECONDS=${STUDIO_TIMEOUT_SECONDS:-120}

mkdir -p target/release/assets/

cp $INPUT_FILE target/release/assets/

# extract input file name without prefix
INPUT_FILE_NAME=$(basename $INPUT_FILE)

mkdir -p $OUTPUT_DIR/frames

# run studio-headless
#$STUDIO_HEADLESS_PATH bvh -o $OUTPUT_DIR/frames --frames $FRAME_COUNT $INPUT_FILE_NAME # NB(bt): commented out
# NB: Output directory might have to end in slash
#RUST_LOG="debug" $STUDIO_HEADLESS_PATH scene \
#RUST_LOG="warn,bevy_gltf::loader=error" $STUDIO_HEADLESS_PATH scene \
# NB(bt,2024-03-25): Removing the camera params since we no longer use the manual non-timeline camera 
	#--camera-motion $STUDIO_CAMERA_SETTING \
	#--camera-motion-speed $STUDIO_CAMERA_SPEED \
#RUST_BACKTRACE=full RUST_LOG="warn,bevy_gltf::loader=error" $STUDIO_HEADLESS_PATH scene \
RUST_BACKTRACE=full RUST_LOG="info" timeout --kill-after 30 $STUDIO_TIMEOUT_SECONDS $STUDIO_HEADLESS_PATH scene \
	-o $OUTPUT_DIR/frames/ \
	--frames $MAX_FRAME_COUNT \
	--skybox $SKYBOX \
	$INPUT_FILE_NAME

# use ffmpeg to create a video from the images

#ffmpeg -framerate $FPS -pattern_type glob -i "$OUTPUT_DIR/frames/*-img.png"  -c:v libx264 -pix_fmt yuv420p $OUTPUT_DIR/../output.mp4
ffmpeg -framerate $FPS -pattern_type glob -i "$OUTPUT_DIR/frames/*.png"  -c:v libx264 -pix_fmt yuv420p $OUTPUT_DIR/../output.mp4

# zip the images and the video
zip -r $OUTPUT_DIR/../output.zip $OUTPUT_DIR/frames
