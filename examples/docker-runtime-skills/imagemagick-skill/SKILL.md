---
name: imagemagick
description: Image manipulation and conversion using ImageMagick inside a secure Docker container
runtime: docker
---

# ImageMagick Skill

Powerful image manipulation and conversion using ImageMagick running inside a Docker container. Resize, crop, rotate, apply effects, convert formats, create thumbnails, add watermarks, and more - all without installing ImageMagick locally.

## Overview

This skill provides access to ImageMagick's comprehensive image processing capabilities through a containerized runtime. All processing happens inside an isolated Docker container with resource limits and no network access, ensuring security while maintaining full ImageMagick functionality.

## When to Use This Skill

**Use this skill when you need to**:
- Convert images between formats (JPEG, PNG, GIF, TIFF, WebP, PDF)
- Resize and scale images
- Create thumbnails with cropping
- Rotate, flip, and crop images
- Apply effects (blur, sharpen, sepia, grayscale)
- Add borders and frames
- Composite images (watermarks, overlays)
- Batch process multiple images
- Create image montages and collages
- Extract image metadata
- Generate PDFs from images

## Prerequisites

### Docker

ImageMagick runs inside a Docker container, so Docker must be installed:

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
[skills.imagemagick]
source = "docker:dpokidov/imagemagick:latest"
runtime = "docker"
description = "ImageMagick image processing"

[skills.imagemagick.docker]
image = "dpokidov/imagemagick:latest"
entrypoint = "magick"
volumes = ["${PWD}:/workdir"]
working_dir = "/workdir"
memory = "1g"
network = "none"
rm = true
```

### Configuration Explained

- **image**: `dpokidov/imagemagick:latest` - ImageMagick Docker image (~100MB)
- **entrypoint**: `magick` - Runs ImageMagick 7.x command
- **volumes**: `${PWD}:/workdir` - Mounts current directory as `/workdir` in container
- **working_dir**: `/workdir` - Sets container working directory
- **memory**: `1g` - Limits container to 1GB RAM
- **network**: `none` - Disables network access (processes local files only)
- **rm**: `true` - Automatically removes container after execution

## Usage

ImageMagick skill uses pass-through syntax - all arguments after `--` are passed directly to ImageMagick:

```bash
skill run imagemagick -- [imagemagick command] [arguments]
```

All file paths are relative to your current directory, which is mounted as `/workdir` in the container.

## Common Operations

### Format Conversion

```bash
# PNG to JPEG
skill run imagemagick -- convert input.png output.jpg

# JPEG to PNG
skill run imagemagick -- convert input.jpg output.png

# Convert to WebP
skill run imagemagick -- convert input.jpg output.webp

# Convert to GIF
skill run imagemagick -- convert input.png output.gif

# Convert to TIFF
skill run imagemagick -- convert input.jpg output.tiff
```

### Resize & Scale

```bash
# Resize to exact dimensions (may distort)
skill run imagemagick -- convert input.jpg -resize 800x600! output.jpg

# Resize maintaining aspect ratio (fit within box)
skill run imagemagick -- convert input.jpg -resize 800x600 output.jpg

# Resize by width only (maintain aspect ratio)
skill run imagemagick -- convert input.jpg -resize 800 output.jpg

# Resize by height only
skill run imagemagick -- convert input.jpg -resize x600 output.jpg

# Resize by percentage
skill run imagemagick -- convert input.jpg -resize 50% output.jpg

# Shrink only (never enlarge)
skill run imagemagick -- convert input.jpg -resize 800x600\> output.jpg

# Enlarge only (never shrink)
skill run imagemagick -- convert input.jpg -resize 800x600\< output.jpg
```

### Thumbnails

```bash
# Basic thumbnail
skill run imagemagick -- convert input.jpg -thumbnail 150x150 thumb.jpg

# Square thumbnail (crop to center)
skill run imagemagick -- convert input.jpg -thumbnail 150x150^ -gravity center -extent 150x150 thumb.jpg

# Thumbnail with quality control
skill run imagemagick -- convert input.jpg -thumbnail 150x150 -quality 85 thumb.jpg

# Multiple thumbnails
skill run imagemagick -- convert input.jpg -thumbnail 300x300 large.jpg -thumbnail 150x150 medium.jpg -thumbnail 75x75 small.jpg
```

### Crop

```bash
# Crop to 100x100 pixels starting at position 50,50
skill run imagemagick -- convert input.jpg -crop 100x100+50+50 output.jpg

# Crop from center
skill run imagemagick -- convert input.jpg -gravity center -crop 200x200+0+0 output.jpg

# Crop specific region
skill run imagemagick -- convert input.jpg -crop 400x300+100+50 output.jpg

# Remove extra canvas space
skill run imagemagick -- convert input.jpg -crop 100x100+50+50 +repage output.jpg
```

### Rotate & Flip

```bash
# Rotate 90 degrees clockwise
skill run imagemagick -- convert input.jpg -rotate 90 output.jpg

# Rotate 180 degrees
skill run imagemagick -- convert input.jpg -rotate 180 output.jpg

# Rotate 270 degrees (90 counter-clockwise)
skill run imagemagick -- convert input.jpg -rotate 270 output.jpg

# Flip horizontally
skill run imagemagick -- convert input.jpg -flop output.jpg

# Flip vertically
skill run imagemagick -- convert input.jpg -flip output.jpg
```

### Color Effects

```bash
# Convert to grayscale
skill run imagemagick -- convert input.jpg -colorspace Gray output.jpg

# Sepia tone
skill run imagemagick -- convert input.jpg -sepia-tone 80% output.jpg

# Negate (invert colors)
skill run imagemagick -- convert input.jpg -negate output.jpg

# Adjust brightness
skill run imagemagick -- convert input.jpg -brightness-contrast 20x0 output.jpg

# Adjust contrast
skill run imagemagick -- convert input.jpg -brightness-contrast 0x30 output.jpg

# Adjust both brightness and contrast
skill run imagemagick -- convert input.jpg -brightness-contrast 10x20 output.jpg

# Colorize
skill run imagemagick -- convert input.jpg -colorize 30,0,0 output.jpg
```

### Image Effects

```bash
# Blur
skill run imagemagick -- convert input.jpg -blur 0x8 output.jpg

# Gaussian blur
skill run imagemagick -- convert input.jpg -gaussian-blur 0x5 output.jpg

# Sharpen
skill run imagemagick -- convert input.jpg -sharpen 0x1 output.jpg

# Edge detection
skill run imagemagick -- convert input.jpg -edge 1 output.jpg

# Emboss
skill run imagemagick -- convert input.jpg -emboss 2 output.jpg

# Charcoal drawing
skill run imagemagick -- convert input.jpg -charcoal 2 output.jpg

# Oil painting
skill run imagemagick -- convert input.jpg -paint 4 output.jpg

# Sketch
skill run imagemagick -- convert input.jpg -sketch 0x20+120 output.jpg
```

### Borders & Frames

```bash
# Simple border
skill run imagemagick -- convert input.jpg -border 10x10 -bordercolor black output.jpg

# Colored border
skill run imagemagick -- convert input.jpg -border 5x5 -bordercolor blue output.jpg

# Frame effect
skill run imagemagick -- convert input.jpg -frame 10x10+5+5 output.jpg

# Rounded corners
skill run imagemagick -- convert input.jpg -alpha set -background none -vignette 0x0 output.png
```

### Watermarks & Compositing

```bash
# Add watermark at bottom right
skill run imagemagick -- composite -gravity southeast watermark.png input.jpg output.jpg

# Add watermark with opacity
skill run imagemagick -- composite -dissolve 30 -gravity southeast watermark.png input.jpg output.jpg

# Center watermark
skill run imagemagick -- composite -gravity center watermark.png input.jpg output.jpg

# Overlay at specific position
skill run imagemagick -- composite -geometry +100+50 overlay.png input.jpg output.jpg
```

### Text Annotation

```bash
# Add text at top
skill run imagemagick -- convert input.jpg -pointsize 30 -fill white -annotate +10+40 "Sample Text" output.jpg

# Add text with background
skill run imagemagick -- convert input.jpg -gravity south -background black -fill white -pointsize 20 -annotate +0+10 "Copyright 2024" output.jpg

# Centered text
skill run imagemagick -- convert input.jpg -gravity center -pointsize 50 -fill red -annotate +0+0 "DRAFT" output.jpg
```

### Quality & Compression

```bash
# Set JPEG quality (0-100, higher = better)
skill run imagemagick -- convert input.jpg -quality 85 output.jpg

# High quality
skill run imagemagick -- convert input.jpg -quality 95 output.jpg

# Web optimized (smaller file)
skill run imagemagick -- convert input.jpg -quality 75 -strip output.jpg

# Strip metadata
skill run imagemagick -- convert input.jpg -strip output.jpg

# PNG compression
skill run imagemagick -- convert input.png -quality 95 output.png
```

### Batch Processing

```bash
# Convert all PNGs to JPEGs
for file in *.png; do
  skill run imagemagick -- convert "$file" "${file%.png}.jpg"
done

# Resize all images
for file in *.jpg; do
  skill run imagemagick -- convert "$file" -resize 800x600 "resized_${file}"
done

# Create thumbnails for all images
for file in *.jpg; do
  skill run imagemagick -- convert "$file" -thumbnail 150x150^ -gravity center -extent 150x150 "thumb_${file}"
done

# Convert multiple formats to PNG
for file in *.{jpg,gif,bmp}; do
  skill run imagemagick -- convert "$file" "${file%.*}.png"
done
```

### Image Montage

```bash
# Create grid montage
skill run imagemagick -- montage *.jpg -geometry +2+2 -tile 3x3 montage.jpg

# Montage with borders
skill run imagemagick -- montage *.jpg -geometry 200x200+5+5 -tile 4x output.jpg

# Labeled montage
skill run imagemagick -- montage *.jpg -label '%f' -geometry +2+2 labeled_montage.jpg

# Contact sheet
skill run imagemagick -- montage *.jpg -tile 5x -geometry 150x150+2+2 contact_sheet.jpg
```

### PDF Operations

```bash
# Convert images to PDF
skill run imagemagick -- convert image1.jpg image2.jpg image3.jpg output.pdf

# Convert all JPEGs to single PDF
skill run imagemagick -- convert *.jpg document.pdf

# PDF to images (one per page)
skill run imagemagick -- convert input.pdf -quality 100 page-%03d.jpg

# Extract specific page from PDF
skill run imagemagick -- convert input.pdf[0] first_page.jpg
```

### Image Information

```bash
# Get basic image info
skill run imagemagick -- identify input.jpg

# Detailed image info
skill run imagemagick -- identify -verbose input.jpg

# Check image format
skill run imagemagick -- identify -format "%m" input.jpg

# Get dimensions
skill run imagemagick -- identify -format "%wx%h" input.jpg

# Get file size
skill run imagemagick -- identify -format "%b" input.jpg
```

## Advanced Examples

### Create Image with Text

```bash
# Create blank image with text
skill run imagemagick -- convert -size 800x600 xc:white -pointsize 50 -fill black -annotate +100+300 "Hello World" text_image.jpg

# Create gradient background with text
skill run imagemagick -- convert -size 800x600 gradient:blue-white -pointsize 60 -fill white -annotate +50+300 "Welcome" gradient_text.jpg
```

### Batch Resize with Quality Control

```bash
# Resize and optimize for web
for img in *.jpg; do
  skill run imagemagick -- convert "$img" \
    -resize 1200x1200\> \
    -quality 85 \
    -strip \
    "web_${img}"
done
```

### Create Polaroid Effect

```bash
skill run imagemagick -- convert input.jpg \
  -bordercolor white -border 15 \
  -bordercolor grey60 -border 1 \
  -background none -rotate 6 \
  -background white -flatten \
  polaroid.jpg
```

### Vignette Effect

```bash
skill run imagemagick -- convert input.jpg \
  -background black \
  -vignette 0x20 \
  vignette.jpg
```

### Picture Frame

```bash
skill run imagemagick -- convert input.jpg \
  -mattecolor white -frame 20x20+10+10 \
  -mattecolor black -frame 3x3+0+0 \
  framed.jpg
```

## File Paths

### Working with Files

All file paths are relative to your current directory:

```bash
# Current directory files
cd /path/to/images
skill run imagemagick -- convert photo.jpg result.png

# Subdirectory files
skill run imagemagick -- convert input/photo.jpg output/result.png

# Absolute paths NOT supported - use relative paths
cd /path/to/images
skill run imagemagick -- convert photo.jpg result.png  # ✓ Works
skill run imagemagick -- convert /path/to/images/photo.jpg result.png  # ✗ Won't work
```

### Container Path Mapping

- Host: `$PWD/photo.jpg` → Container: `/workdir/photo.jpg`
- Host: `$PWD/images/photo.jpg` → Container: `/workdir/images/photo.jpg`

## Supported Formats

### Input Formats
JPEG, PNG, GIF, TIFF, BMP, WebP, SVG, PDF, PSD, ICO, RAW (many variants)

### Output Formats
JPEG, PNG, GIF, TIFF, BMP, WebP, PDF, ICO, and more

### Special Formats
- **xc:color** - Create solid color image (e.g., `xc:white`, `xc:#FF0000`)
- **gradient:color1-color2** - Create gradient
- **pattern:name** - Built-in patterns

## Security Features

### Resource Limits
- **Memory**: 1GB limit prevents excessive RAM usage
- **Network**: Disabled - no outbound connections
- **Filesystem**: Only current directory mounted (read/write)

### Container Isolation
- Runs in isolated container namespace
- No access to host system files outside mounted directory
- Container auto-removed after execution
- No persistent container state

## Performance Tips

1. **Use appropriate quality settings**:
   - Web: `-quality 75-85`
   - Print: `-quality 90-95`
   - Archive: `-quality 95-100`

2. **Strip metadata** for smaller files:
   ```bash
   skill run imagemagick -- convert input.jpg -strip output.jpg
   ```

3. **Batch process** efficiently:
   ```bash
   skill run imagemagick -- mogrify -resize 800x600 -quality 85 *.jpg
   ```

4. **Use correct format** for the use case:
   - Photos: JPEG
   - Graphics/logos: PNG
   - Animations: GIF
   - Web: WebP (smaller than JPEG/PNG)

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

### Out of Memory

If processing large images, increase memory limit in `.skill-engine.toml`:

```toml
[skills.imagemagick.docker]
memory = "2g"  # Increase to 2GB
```

### File Not Found

Ensure you're in the correct directory:

```bash
# Check current directory
pwd
ls -la  # Verify files exist

# Change to correct directory
cd /path/to/images
skill run imagemagick -- convert photo.jpg result.png
```

### Permission Errors

```bash
# Check file permissions
ls -la photo.jpg

# Make file readable
chmod 644 photo.jpg
```

### Image Quality Issues

```bash
# Use higher quality setting
skill run imagemagick -- convert input.jpg -quality 95 output.jpg

# Avoid multiple conversions (quality degrades)
# Bad: JPG → PNG → JPG (loses quality twice)
# Good: JPG → PNG (one conversion)
```

## Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| "No such file" | Wrong directory or path | Use `pwd` and `ls` to verify location |
| "Permission denied" | File permissions | `chmod 644 filename` |
| "Memory allocation failed" | Out of memory | Increase memory limit in config |
| "Cannot connect to Docker" | Docker not running | Start Docker Desktop/service |
| "Unable to open image" | Corrupted or unsupported file | Check file format with `identify` |

## ImageMagick Commands Reference

### Main Commands

- **convert** - Convert image format or apply operations
- **identify** - Get image information
- **mogrify** - Modify image in-place (overwrites original)
- **composite** - Overlay images
- **montage** - Create image grid/collage
- **compare** - Compare two images

### Usage Examples

```bash
# convert (creates new file)
skill run imagemagick -- convert input.jpg -resize 800x600 output.jpg

# mogrify (modifies original - BE CAREFUL!)
skill run imagemagick -- mogrify -resize 800x600 image.jpg

# composite (overlay/watermark)
skill run imagemagick -- composite watermark.png input.jpg output.jpg
```

## Docker Image Details

- **Image**: `dpokidov/imagemagick:latest`
- **Base**: Alpine Linux
- **Size**: ~100MB
- **ImageMagick Version**: 7.x
- **Platforms**: linux/amd64, linux/arm64

## Resources

- [ImageMagick Official Documentation](https://imagemagick.org/index.php)
- [Command-Line Options Reference](https://imagemagick.org/script/command-line-options.php)
- [Usage Examples](https://imagemagick.org/Usage/)
- [Format Support](https://imagemagick.org/script/formats.php)
- [Color Names](https://imagemagick.org/script/color.php)

## Quick Reference

### Essential Operations Table

| Operation | Command |
|-----------|---------|
| Convert format | `convert input.png output.jpg` |
| Resize | `convert input.jpg -resize 800x600 output.jpg` |
| Thumbnail | `convert input.jpg -thumbnail 150x150^ -gravity center -extent 150x150 thumb.jpg` |
| Rotate | `convert input.jpg -rotate 90 output.jpg` |
| Crop | `convert input.jpg -crop 100x100+10+10 output.jpg` |
| Grayscale | `convert input.jpg -colorspace Gray output.jpg` |
| Blur | `convert input.jpg -blur 0x8 output.jpg` |
| Sharpen | `convert input.jpg -sharpen 0x1 output.jpg` |
| Border | `convert input.jpg -border 5x5 -bordercolor black output.jpg` |
| Watermark | `composite -gravity southeast watermark.png input.jpg output.jpg` |
| Get info | `identify input.jpg` |
| To PDF | `convert *.jpg output.pdf` |
