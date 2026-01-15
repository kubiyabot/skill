# ImageMagick Skill

Image manipulation and conversion using ImageMagick inside a Docker container.

## Usage

```bash
# Convert format
skill run imagemagick -- convert input.png output.jpg

# Resize image
skill run imagemagick -- convert input.jpg -resize 800x600 output.jpg

# Create thumbnail
skill run imagemagick -- convert input.jpg -thumbnail 150x150^ -gravity center -extent 150x150 thumb.jpg

# Add watermark
skill run imagemagick -- composite -gravity southeast watermark.png input.jpg output.jpg

# Convert to PDF
skill run imagemagick -- convert *.jpg output.pdf

# Get image info
skill run imagemagick -- identify input.jpg
```

## Configuration

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

## Security

- **Network**: `none` - No network access
- **Memory**: 1GB limit
- **Volumes**: Current directory only

## Common Operations

| Operation | Command |
|-----------|---------|
| Convert format | `convert input.png output.jpg` |
| Resize | `convert input.jpg -resize 800x600 output.jpg` |
| Thumbnail | `convert input.jpg -thumbnail 150x150 thumb.jpg` |
| Rotate | `convert input.jpg -rotate 90 output.jpg` |
| Crop | `convert input.jpg -crop 100x100+10+10 output.jpg` |
| Grayscale | `convert input.jpg -colorspace Gray output.jpg` |
| Blur | `convert input.jpg -blur 0x8 output.jpg` |
| Sharpen | `convert input.jpg -sharpen 0x1 output.jpg` |
| Border | `convert input.jpg -border 5x5 -bordercolor black output.jpg` |
| Montage | `montage *.jpg -geometry +2+2 output.jpg` |
| Info | `identify input.jpg` |
| PDF | `convert *.jpg output.pdf` |

## Docker Image

- **Image**: `dpokidov/imagemagick:latest`
- **Size**: ~100MB
- **ImageMagick**: 7.x
- **Formats**: JPEG, PNG, GIF, TIFF, PDF, WebP, and more
