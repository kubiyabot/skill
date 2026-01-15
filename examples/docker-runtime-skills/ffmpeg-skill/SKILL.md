---
name: ffmpeg
description: Video and audio processing using FFmpeg inside a secure Docker container
runtime: docker
---

# FFmpeg Skill

Process video and audio files using FFmpeg running inside a Docker container. Convert formats, extract audio, resize videos, create thumbnails, compress media, and more - all without installing FFmpeg locally.

## Overview

This skill provides access to FFmpeg's powerful media processing capabilities through a containerized runtime. All processing happens inside an isolated Docker container with resource limits and no network access, ensuring security while maintaining full FFmpeg functionality.

## When to Use This Skill

**Use this skill when you need to**:
- Convert video/audio between formats (MP4, WebM, AVI, MKV, MP3, WAV, etc.)
- Extract audio from video files
- Resize or scale videos to different resolutions
- Compress videos to reduce file size
- Create thumbnails from videos
- Trim or cut video clips
- Convert videos to animated GIFs
- Merge multiple video files
- Add or extract subtitles
- Adjust video bitrate and quality

## Prerequisites

### Docker

FFmpeg runs inside a Docker container, so Docker must be installed:

```bash
# macOS (using Homebrew)
brew install --cask docker

# Or download from: https://www.docker.com/products/docker-desktop

# Linux (Ubuntu/Debian)
sudo apt-get update
sudo apt-get install docker.io

# Start Docker service
sudo systemctl start docker
sudo systemctl enable docker
```

### Verify Docker Installation

```bash
docker --version
docker ps  # Should list running containers
```

## Configuration

Add to your `.skill-engine.toml`:

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

### Configuration Explained

- **image**: `linuxserver/ffmpeg:latest` - Official FFmpeg Docker image (~200MB)
- **entrypoint**: `ffmpeg` - Directly runs FFmpeg command
- **volumes**: `${PWD}:/workdir` - Mounts current directory as `/workdir` in container
- **working_dir**: `/workdir` - Sets container working directory
- **memory**: `2g` - Limits container to 2GB RAM
- **cpus**: `4` - Limits to 4 CPU cores
- **network**: `none` - Disables network access (processes local files only)
- **rm**: `true` - Automatically removes container after execution

## Usage

FFmpeg skill uses pass-through syntax - all arguments after `--` are passed directly to FFmpeg:

```bash
skill run ffmpeg -- [ffmpeg arguments]
```

All file paths are relative to your current directory, which is mounted as `/workdir` in the container.

## Common Operations

### Format Conversion

```bash
# MP4 to WebM
skill run ffmpeg -- -i input.mp4 output.webm

# AVI to MP4
skill run ffmpeg -- -i input.avi output.mp4

# MOV to MKV
skill run ffmpeg -- -i input.mov output.mkv

# Video to MP3 audio only
skill run ffmpeg -- -i video.mp4 -vn -acodec mp3 audio.mp3
```

### Audio Operations

```bash
# Extract audio from video
skill run ffmpeg -- -i video.mp4 -vn audio.mp3

# Convert audio format
skill run ffmpeg -- -i audio.wav audio.mp3

# Adjust audio bitrate
skill run ffmpeg -- -i input.mp3 -b:a 192k output.mp3

# Merge audio and video
skill run ffmpeg -- -i video.mp4 -i audio.mp3 -c copy output.mp4
```

### Video Resizing

```bash
# Resize to 1280x720 (720p)
skill run ffmpeg -- -i input.mp4 -vf scale=1280:720 output.mp4

# Resize to 1920x1080 (1080p)
skill run ffmpeg -- -i input.mp4 -vf scale=1920:1080 output.mp4

# Resize width to 640, maintain aspect ratio
skill run ffmpeg -- -i input.mp4 -vf scale=640:-1 output.mp4

# Resize height to 480, maintain aspect ratio
skill run ffmpeg -- -i input.mp4 -vf scale=-1:480 output.mp4
```

### Video Compression

```bash
# Compress with CRF (Constant Rate Factor) - lower is higher quality
# CRF range: 0 (lossless) to 51 (worst), 23 is default, 18 is visually lossless
skill run ffmpeg -- -i input.mp4 -crf 28 output.mp4

# Compress with target bitrate
skill run ffmpeg -- -i input.mp4 -b:v 1M output.mp4

# Two-pass encoding for better quality at target size
skill run ffmpeg -- -i input.mp4 -b:v 1M -pass 1 -f mp4 /dev/null
skill run ffmpeg -- -i input.mp4 -b:v 1M -pass 2 output.mp4
```

### Thumbnails & Images

```bash
# Extract single frame at 5 seconds
skill run ffmpeg -- -i video.mp4 -ss 00:00:05 -vframes 1 thumb.jpg

# Extract frame at specific time
skill run ffmpeg -- -i video.mp4 -ss 00:01:30 -vframes 1 frame.png

# Create thumbnail at 1/4 size
skill run ffmpeg -- -i video.mp4 -ss 00:00:05 -vf scale=iw/4:ih/4 -vframes 1 thumb.jpg

# Generate thumbnail every 10 seconds
skill run ffmpeg -- -i video.mp4 -vf fps=1/10 thumb%04d.jpg
```

### GIF Creation

```bash
# Convert video to GIF
skill run ffmpeg -- -i input.mp4 output.gif

# High quality GIF with palette optimization
skill run ffmpeg -- -i input.mp4 -vf "fps=10,scale=320:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse" output.gif

# Simple GIF with reduced frame rate and size
skill run ffmpeg -- -i input.mp4 -vf "fps=10,scale=320:-1" output.gif
```

### Trim & Cut

```bash
# Trim from 10s to 40s (30 second clip)
skill run ffmpeg -- -i input.mp4 -ss 00:00:10 -t 00:00:30 output.mp4

# Trim from 1 minute to end
skill run ffmpeg -- -i input.mp4 -ss 00:01:00 -c copy output.mp4

# Cut first 10 seconds
skill run ffmpeg -- -i input.mp4 -ss 00:00:10 -c copy output.mp4

# Extract specific time range (from 5s to 15s)
skill run ffmpeg -- -i input.mp4 -ss 00:00:05 -to 00:00:15 -c copy output.mp4
```

### Merge & Concatenate

```bash
# Create file list
echo "file 'video1.mp4'" > filelist.txt
echo "file 'video2.mp4'" >> filelist.txt
echo "file 'video3.mp4'" >> filelist.txt

# Concatenate videos
skill run ffmpeg -- -f concat -safe 0 -i filelist.txt -c copy merged.mp4
```

### Rotate & Flip

```bash
# Rotate 90 degrees clockwise
skill run ffmpeg -- -i input.mp4 -vf "transpose=1" output.mp4

# Rotate 180 degrees
skill run ffmpeg -- -i input.mp4 -vf "transpose=2,transpose=2" output.mp4

# Flip horizontally
skill run ffmpeg -- -i input.mp4 -vf "hflip" output.mp4

# Flip vertically
skill run ffmpeg -- -i input.mp4 -vf "vflip" output.mp4
```

### Speed Adjustment

```bash
# Speed up 2x (double speed)
skill run ffmpeg -- -i input.mp4 -vf "setpts=0.5*PTS" output.mp4

# Slow down 0.5x (half speed)
skill run ffmpeg -- -i input.mp4 -vf "setpts=2.0*PTS" output.mp4
```

### Subtitles

```bash
# Add subtitle file
skill run ffmpeg -- -i video.mp4 -i subtitles.srt -c copy -c:s mov_text output.mp4

# Burn subtitles into video
skill run ffmpeg -- -i video.mp4 -vf subtitles=subtitles.srt output.mp4

# Extract subtitles
skill run ffmpeg -- -i video.mp4 -map 0:s:0 subtitles.srt
```

## Advanced Examples

### Batch Processing

```bash
# Convert all MP4 files to WebM
for file in *.mp4; do
  skill run ffmpeg -- -i "$file" "${file%.mp4}.webm"
done

# Create thumbnails for all videos
for file in *.mp4; do
  skill run ffmpeg -- -i "$file" -ss 00:00:05 -vframes 1 "${file%.mp4}.jpg"
done
```

### Quality Presets

```bash
# High quality (larger file)
skill run ffmpeg -- -i input.mp4 -crf 18 -preset slow output.mp4

# Balanced quality/speed
skill run ffmpeg -- -i input.mp4 -crf 23 -preset medium output.mp4

# Fast encoding (lower quality)
skill run ffmpeg -- -i input.mp4 -crf 28 -preset fast output.mp4

# Very fast (lowest quality)
skill run ffmpeg -- -i input.mp4 -crf 28 -preset ultrafast output.mp4
```

### Social Media Formats

```bash
# Instagram square (1:1, 1080x1080)
skill run ffmpeg -- -i input.mp4 -vf "scale=1080:1080:force_original_aspect_ratio=decrease,pad=1080:1080:(ow-iw)/2:(oh-ih)/2" -c:a copy instagram.mp4

# YouTube (1080p)
skill run ffmpeg -- -i input.mp4 -vf scale=1920:1080 -c:v libx264 -preset slow -crf 18 youtube.mp4

# Twitter (720p, under 512MB)
skill run ffmpeg -- -i input.mp4 -vf scale=1280:720 -b:v 2M twitter.mp4
```

## File Paths

### Working with Files

All file paths are relative to your current directory:

```bash
# Current directory files
cd /path/to/videos
skill run ffmpeg -- -i video.mp4 output.webm

# Subdirectory files
skill run ffmpeg -- -i input/video.mp4 output/video.webm

# Absolute paths NOT supported - use relative paths
cd /path/to/videos
skill run ffmpeg -- -i video.mp4 result.mp4  # ✓ Works
skill run ffmpeg -- -i /path/to/videos/video.mp4 result.mp4  # ✗ Won't work
```

### Container Path Mapping

- Host: `$PWD/video.mp4` → Container: `/workdir/video.mp4`
- Host: `$PWD/input/file.mp4` → Container: `/workdir/input/file.mp4`

## Supported Formats

### Video Codecs
H.264 (MP4), H.265 (HEVC), VP8, VP9, AV1, MPEG-2, MPEG-4, Theora, DivX, Xvid

### Audio Codecs
AAC, MP3, Opus, Vorbis, FLAC, WAV, AC3, DTS

### Container Formats
MP4, WebM, MKV, AVI, MOV, FLV, 3GP, OGG, M4V, WMV

## Security Features

### Resource Limits
- **Memory**: 2GB limit prevents excessive RAM usage
- **CPU**: 4 cores max prevents CPU hogging
- **Network**: Disabled - no outbound connections
- **Filesystem**: Only current directory mounted (read/write)

### Container Isolation
- Runs in isolated container namespace
- No access to host system files outside mounted directory
- Container auto-removed after execution
- No persistent container state

## Performance Tips

1. **Use appropriate presets**:
   - `ultrafast` - Quick encoding, larger files
   - `medium` - Balanced (default)
   - `slow` - Better compression, slower
   - `veryslow` - Best compression, very slow

2. **Copy streams when possible**:
   ```bash
   skill run ffmpeg -- -i input.mp4 -c copy output.mp4  # No re-encoding
   ```

3. **Hardware acceleration** (if supported by image):
   ```bash
   skill run ffmpeg -- -hwaccel auto -i input.mp4 output.mp4
   ```

4. **Batch process at once** rather than multiple sequential runs

## Troubleshooting

### Docker Not Running

```bash
# Check Docker status
docker ps

# Start Docker Desktop (macOS)
open /Applications/Docker.app

# Start Docker service (Linux)
sudo systemctl start docker
```

### Container Memory Issues

If processing large files, increase memory limit in `.skill-engine.toml`:

```toml
[skills.ffmpeg.docker]
memory = "4g"  # Increase to 4GB
```

### File Not Found

Ensure you're in the correct directory:

```bash
# Check current directory
pwd
ls -la  # Verify files exist

# Change to correct directory
cd /path/to/videos
skill run ffmpeg -- -i video.mp4 output.webm
```

### Permission Errors

Docker container may have permission issues:

```bash
# Check file permissions
ls -la video.mp4

# Make file readable
chmod 644 video.mp4
```

## Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| "No such file" | Wrong directory or path | Use `pwd` and `ls` to verify location |
| "Permission denied" | File permissions | `chmod 644 filename` |
| "Killed" | Out of memory | Increase memory limit in config |
| "Cannot connect to Docker" | Docker not running | Start Docker Desktop/service |
| "Invalid codec" | Unsupported format | Check input format with `ffmpeg -i file` |

## Docker Image Details

- **Image**: `linuxserver/ffmpeg:latest`
- **Base**: Alpine Linux
- **Size**: ~200MB
- **Platforms**: linux/amd64, linux/arm64, linux/arm/v7
- **FFmpeg Version**: Latest stable (6.x)
- **Updates**: Automatic with `:latest` tag

## Resources

- [FFmpeg Official Documentation](https://ffmpeg.org/documentation.html)
- [FFmpeg Wiki](https://trac.ffmpeg.org/)
- [Common FFmpeg Commands](https://ffmpeg.org/ffmpeg.html#Examples)
- [Video Codecs Guide](https://trac.ffmpeg.org/wiki/Encode/H.264)
- [Audio Encoding Guide](https://trac.ffmpeg.org/wiki/Encode/AAC)

## Examples Reference

### Quick Reference Table

| Task | Command |
|------|---------|
| Convert format | `-i input.mp4 output.webm` |
| Extract audio | `-i video.mp4 -vn audio.mp3` |
| Resize | `-i input.mp4 -vf scale=1280:720 output.mp4` |
| Compress | `-i input.mp4 -crf 28 output.mp4` |
| Trim | `-i input.mp4 -ss 00:00:10 -t 00:00:30 output.mp4` |
| Thumbnail | `-i video.mp4 -ss 00:00:05 -vframes 1 thumb.jpg` |
| To GIF | `-i input.mp4 -vf "fps=10,scale=320:-1" output.gif` |
| Merge | `-f concat -safe 0 -i filelist.txt -c copy merged.mp4` |