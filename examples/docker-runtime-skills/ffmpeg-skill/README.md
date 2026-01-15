# FFmpeg Skill

Video and audio processing using FFmpeg inside a Docker container.

## Usage

```bash
# Convert video format
skill run ffmpeg -- -i input.mp4 output.webm

# Extract audio
skill run ffmpeg -- -i video.mp4 -vn -acodec mp3 audio.mp3

# Resize video
skill run ffmpeg -- -i input.mp4 -vf scale=1280:720 output.mp4

# Create thumbnail
skill run ffmpeg -- -i video.mp4 -ss 00:00:05 -vframes 1 thumb.jpg

# Convert to GIF
skill run ffmpeg -- -i input.mp4 -vf "fps=10,scale=320:-1" output.gif
```

## Configuration

```toml
[skills.ffmpeg]
source = "docker:linuxserver/ffmpeg:latest"
runtime = "docker"
description = "FFmpeg video/audio processing"

[skills.ffmpeg.docker]
image = "linuxserver/ffmpeg:latest"
entrypoint = "ffmpeg"
volumes = ["${PWD}:/workdir"]
working_dir = "/workdir"
memory = "2g"
cpus = "4"
network = "none"
rm = true
```

## Security

- **Network**: `none` - No network access (processes local files only)
- **Memory**: 2GB limit
- **CPU**: 4 cores max
- **Volumes**: Current directory mounted as `/workdir`

## Common Operations

| Operation | Command |
|-----------|---------|
| Convert format | `-i input.mp4 output.webm` |
| Extract audio | `-i video.mp4 -vn audio.mp3` |
| Resize | `-i input.mp4 -vf scale=1280:720 output.mp4` |
| Compress | `-i input.mp4 -crf 28 output.mp4` |
| Trim | `-i input.mp4 -ss 00:00:10 -t 00:00:30 output.mp4` |
| Thumbnail | `-i video.mp4 -ss 00:00:05 -vframes 1 thumb.jpg` |
| To GIF | `-i input.mp4 -vf "fps=10,scale=320:-1" output.gif` |
| Audio convert | `-i audio.wav audio.mp3` |

## Docker Image

- **Image**: `linuxserver/ffmpeg:latest`
- **Size**: ~200MB
- **Platform**: linux/amd64, linux/arm64
