# Rust Motion Detection with OpenCV

This project is a simple motion detection program written in Rust using OpenCV bindings. I wrote it to familiarize myself with OpenCV in Rust.

## Overview

The program captures video frames from a webcam, compares consecutive frames to detect motion, and highlights areas and regions of motion using colored rectangles. It divides frames into a grid of cells, detects motion per cell, and groups connected motion cells into regions of interest (ROIs).

## Features

- Capture live video from webcam or video file
- Convert frames to grayscale and calculate frame differences to detect motion
- Divide frame into grid cells and track motion per cell
- Identify connected motion cells to form motion regions
- Draw bounding boxes around cells with motion and around groups of motion cells (ROIs)

## Dependencies

- Rust toolchain (1.65+ recommended)
- OpenCV library installed on your system
- Rust `opencv` crate for OpenCV bindings
- `anyhow` crate for error handling

## Installation

1. For installing OpenCV on your system, please refer to [opencv-rust](https://github.com/twistedfall/opencv-rust/blob/master/INSTALL.md).

2. Ensure Rust is installed: https://rust-lang.org

3. Add dependencies in your `Cargo.toml`:

## Usage

Build and run the program with:
```
cargo run
```

## Customization

- Modify the `cells` parameter in the `run` function call to change the grid cell size and granularity.
- Adjust the motion detection threshold (currently set to 40.0) for sensitivity.
- Swap the camera input with a video file by replacing the `VideoCapture::new(0, ...)` with `VideoCapture::from_file("file.mp4", ...)` in `main`.

The license for the original works [MIT](https://opensource.org/license/MIT).