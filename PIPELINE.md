# Composable Pipeline Architecture

## Overview

The image processing pipeline uses a trait-based architecture where each step processes a `Vec<PipelineData>` and returns a transformed `Vec<PipelineData>`. This design enables:

- **Splitting**: One item becoming many (e.g., detecting multiple contours in an image)
- **Filtering**: Many items becoming fewer (e.g., filtering for circular contours)
- **Transformation**: Processing items individually (e.g., grayscale conversion)

## Core Abstractions

### PipelineData Struct

Each `PipelineData` represents a single image region with metadata:

```rust
pub struct PipelineData {
    /// The image data (can be grayscale or color)
    pub image: DynamicImage,

    /// Reference to the original image (shared efficiently via Arc)
    pub original: Arc<DynamicImage>,

    /// Bounding box in the original image (None means full image)
    pub bbox: Option<BoundingBox>,

    /// Metadata for tracking properties (e.g., "is_circle", "brightness")
    pub metadata: HashMap<String, MetadataValue>,
}
```

**Key features**:
- Original image preserved via `Arc` for efficient sharing
- Bounding box tracks where this data came from in the original
- Metadata stores arbitrary properties as key-value pairs

### MetadataValue Enum

```rust
pub enum MetadataValue {
    Bool(bool),      // e.g., "is_circle": true
    Float(f32),      // e.g., "radius": 10.5
    String(String),  // e.g., "ocr_text": "42"
    Int(i32),        // e.g., "pixel_count": 350
}
```

Helper methods on `PipelineData`:
- `get_bool(key)` → `Option<bool>`
- `get_float(key)` → `Option<f32>`
- `get_string(key)` → `Option<&str>`

### PipelineStep Trait

All processing steps implement this trait:

```rust
pub trait PipelineStep: Send + Sync {
    /// Process data and return transformed data
    /// Steps can split data (1 → many), filter (many → fewer), or transform (many → many)
    fn process(&self, data: Vec<PipelineData>, context: &PipelineContext)
        -> Result<Vec<PipelineData>>;

    /// Human-readable name for this step (used in verbose output)
    fn name(&self) -> &str;
}
```

### Pipeline Builder

```rust
let mut pipeline = Pipeline::new()
    .with_verbose(true)
    .add_step(Box::new(SomeStep { param: 1.0 }))
    .add_step(Box::new(AnotherStep))
    // ... more steps
    ;

let results: Vec<PipelineData> = pipeline.run(input_image)?;
```

## How Data Flows

### Example: Standard Detection Pipeline

```
Input: 1 image
  ↓
GrayscaleStep: 1 → 1 (convert to gray)
  ↓
BlurStep: 1 → 1 (apply blur)
  ↓
EdgeDetectionStep: 1 → 1 (detect edges)
  ↓
ContourDetectionStep: 1 → 100 (SPLIT: find 100 contours)
  ↓
CircleFilterStep: 100 → 40 (FILTER: keep only circles)
  ↓
WhiteCircleFilterStep: 40 → 40 (FILTER: all are white)
  ↓
OcrStep: 40 → 31 (FILTER: 31 have recognizable text)
  ↓
Output: 31 detections with OCR text
```

## Available Pipeline Steps

### 1. GrayscaleStep
Converts color images to grayscale.
- Input: Vec of color/grayscale images
- Output: Vec of grayscale images (same count)
- Metadata: Unchanged

### 2. BlurStep
Applies Gaussian blur to reduce noise.
- Input: Vec of grayscale images
- Output: Vec of blurred grayscale images (same count)
- Parameters: `sigma: f32` (blur strength, typically 1.5)
- Metadata: Unchanged

### 3. EdgeDetectionStep
Detects edges using the Canny algorithm.
- Input: Vec of grayscale images
- Output: Vec of edge images (same count)
- Parameters:
  - `low_threshold: f32` (typically 50.0)
  - `high_threshold: f32` (typically 100.0)
- Metadata: Unchanged

### 4. ContourDetectionStep
Finds connected components in edge images. **This is a splitting step** - one edge image becomes many contour regions.

- Input: Vec of edge images
- Output: Vec of contour regions (many more items!)
- Parameters: `min_area: u32` (minimum pixel count, typically 10)
- Metadata added:
  - `contour_min_x`, `contour_min_y`, `contour_max_x`, `contour_max_y` (Int)
  - `pixel_count` (Int)
  - `radius` (Float)
  - `circularity` (Float)
  - `aspect_ratio` (Float)
- Bounding box: Set to contour bounds in original image

### 5. CircleFilterStep
Filters contours to keep only circular shapes. **This is a filtering step**.

- Input: Vec of contours
- Output: Vec of circular contours (fewer items)
- Parameters:
  - `min_radius: f32` (typically 10.0)
  - `max_radius: f32` (typically 200.0)
  - `circularity_threshold: f32` (max circularity, typically 2.0; 1.0 = perfect circle)
- Metadata added:
  - `is_circle` (Bool): true
- Uses metadata: `circularity`, `radius`, `aspect_ratio`

### 6. WhiteCircleFilterStep
Filters circles by brightness/whiteness. **This is a filtering step**.

- Input: Vec of circles
- Output: Vec of white circles (fewer or same items)
- Parameters: `brightness_threshold: f32` (typically 200.0; range 0-255)
- Metadata added:
  - `is_white` (Bool): true
  - `brightness` (Float): average brightness value
- Requires: Original image in `PipelineData::original`

### 7. OcrStep
Recognizes text from detected circles using OCR. **This is a filtering step** - only circles with recognized text are kept.

- Input: Vec of circles
- Output: Vec of circles with recognized text (fewer items)
- Metadata added:
  - `ocr_text` (String): recognized text
  - `ocr_confidence` (Float): OCR confidence (0.0-1.0)
- Requires: Original image and contour metadata

## Usage Examples

### Standard Detection Pipeline

```rust
use addrslips::Pipeline;
use addrslips::detection::steps::*;

let mut pipeline = Pipeline::new()
    .with_verbose(true)
    .add_step(Box::new(GrayscaleStep))
    .add_step(Box::new(BlurStep { sigma: 1.5 }))
    .add_step(Box::new(EdgeDetectionStep {
        low_threshold: 50.0,
        high_threshold: 100.0,
    }))
    .add_step(Box::new(ContourDetectionStep { min_area: 10 }))
    .add_step(Box::new(CircleFilterStep {
        min_radius: 10.0,
        max_radius: 200.0,
        circularity_threshold: 2.0,
    }))
    .add_step(Box::new(WhiteCircleFilterStep {
        brightness_threshold: 200.0,
    }))
    .add_step(Box::new(OcrStep));

let results = pipeline.run(img)?;

// Extract detections
for item in results {
    let text = item.get_string("ocr_text").unwrap_or("?");
    let confidence = item.get_float("ocr_confidence").unwrap_or(0.0);
    let bbox = item.bbox.as_ref().unwrap();

    println!("Found '{}' at ({}, {}) with confidence {:.2}",
             text, bbox.x, bbox.y, confidence);
}
```

### Detection Without OCR (Faster)

```rust
let mut pipeline = Pipeline::new()
    .add_step(Box::new(GrayscaleStep))
    .add_step(Box::new(BlurStep { sigma: 1.5 }))
    .add_step(Box::new(EdgeDetectionStep {
        low_threshold: 50.0,
        high_threshold: 100.0,
    }))
    .add_step(Box::new(ContourDetectionStep { min_area: 10 }))
    .add_step(Box::new(CircleFilterStep {
        min_radius: 10.0,
        max_radius: 200.0,
        circularity_threshold: 2.0,
    }))
    .add_step(Box::new(WhiteCircleFilterStep {
        brightness_threshold: 200.0,
    }));
    // No OcrStep - much faster!

let circles = pipeline.run(img)?;
println!("Found {} white circles", circles.len());

for circle in circles {
    let radius = circle.get_float("radius").unwrap_or(0.0);
    let brightness = circle.get_float("brightness").unwrap_or(0.0);
    println!("  Circle: radius={:.1}, brightness={:.1}", radius, brightness);
}
```

### Custom Parameters

```rust
// More aggressive preprocessing for noisy images
let mut custom_pipeline = Pipeline::new()
    .add_step(Box::new(GrayscaleStep))
    .add_step(Box::new(BlurStep { sigma: 2.5 }))  // Stronger blur
    .add_step(Box::new(EdgeDetectionStep {
        low_threshold: 40.0,   // Lower thresholds
        high_threshold: 90.0,
    }))
    .add_step(Box::new(ContourDetectionStep { min_area: 25 }))  // Larger minimum
    .add_step(Box::new(CircleFilterStep {
        min_radius: 15.0,      // Only larger circles
        max_radius: 150.0,
        circularity_threshold: 1.5,  // Stricter circularity
    }))
    .add_step(Box::new(WhiteCircleFilterStep {
        brightness_threshold: 220.0,  // Very white only
    }));
```

### Debugging: Partial Execution

Stop the pipeline after a few steps to inspect intermediate results:

```rust
let mut pipeline = Pipeline::new()
    .with_verbose(true)
    .add_step(Box::new(GrayscaleStep))
    .add_step(Box::new(BlurStep { sigma: 1.5 }))
    .add_step(Box::new(EdgeDetectionStep {
        low_threshold: 50.0,
        high_threshold: 100.0,
    }));

let edge_images = pipeline.run_partial(img, 3)?;  // Stop after 3 steps

// Save intermediate result for inspection
if let Some(edge_data) = edge_images.first() {
    edge_data.image.save("debug_edges.png")?;
}
```

## Creating Custom Steps

To add a new processing step:

```rust
use addrslips::pipeline::{PipelineData, PipelineStep, PipelineContext, MetadataValue};

pub struct MyCustomStep {
    pub my_parameter: f32,
}

impl PipelineStep for MyCustomStep {
    fn process(&self, data: Vec<PipelineData>, _context: &PipelineContext)
        -> Result<Vec<PipelineData>>
    {
        let mut result = Vec::new();

        for item in data {
            // Example: Transform each item
            let processed_image = my_algorithm(&item.image, self.my_parameter)?;

            let mut new_item = PipelineData {
                image: processed_image,
                original: item.original.clone(),
                bbox: item.bbox.clone(),
                metadata: item.metadata.clone(),
            };

            // Add custom metadata
            new_item.metadata.insert(
                "my_metric".to_string(),
                MetadataValue::Float(compute_metric(&new_item.image))
            );

            result.push(new_item);
        }

        Ok(result)
    }

    fn name(&self) -> &str {
        "My Custom Processing"
    }
}

// Use it:
pipeline.add_step(Box::new(MyCustomStep { my_parameter: 1.0 }))
```

### Example: Filtering Step

```rust
pub struct MinBrightnessFilter {
    pub threshold: f32,
}

impl PipelineStep for MinBrightnessFilter {
    fn process(&self, data: Vec<PipelineData>, _context: &PipelineContext)
        -> Result<Vec<PipelineData>>
    {
        // Filter: keep only items above brightness threshold
        Ok(data.into_iter()
            .filter(|item| {
                item.get_float("brightness").unwrap_or(0.0) >= self.threshold
            })
            .collect())
    }

    fn name(&self) -> &str {
        "Brightness Filter"
    }
}
```

### Example: Splitting Step

```rust
pub struct RegionSplitStep {
    pub grid_size: u32,
}

impl PipelineStep for RegionSplitStep {
    fn process(&self, data: Vec<PipelineData>, _context: &PipelineContext)
        -> Result<Vec<PipelineData>>
    {
        let mut result = Vec::new();

        for item in data {
            let (width, height) = item.image.dimensions();
            let step = self.grid_size;

            // Split image into grid regions
            for y in (0..height).step_by(step as usize) {
                for x in (0..width).step_by(step as usize) {
                    let w = step.min(width - x);
                    let h = step.min(height - y);

                    let region = item.image.crop_imm(x, y, w, h);

                    let region_data = PipelineData::from_region(
                        region,
                        item.original.clone(),
                        BoundingBox { x, y, width: w, height: h }
                    );

                    result.push(region_data);
                }
            }
        }

        Ok(result)
    }

    fn name(&self) -> &str {
        "Grid Split"
    }
}
```

## Benefits of This Architecture

1. **Flexible Data Flow**: Steps can split (1→many), filter (many→few), or transform (1→1)
2. **Rich Metadata**: Each item carries context and properties through the pipeline
3. **Efficient Memory**: Original image shared via Arc, not copied
4. **Traceability**: Bounding boxes track where data came from
5. **Composability**: Mix and match steps freely
6. **Type Safety**: All data flows through `PipelineData` with runtime type checking
7. **Debuggability**: Inspect data at any pipeline stage
8. **Extensibility**: Easy to add custom steps

## Verbose Output Example

With `.with_verbose(true)`, you see the flow:

```
Running step: Grayscale Conversion (processing 1 items)
  → 1 items
Running step: Gaussian Blur (processing 1 items)
  → 1 items
Running step: Edge Detection (processing 1 items)
  → 1 items
Running step: Contour Detection (processing 1 items)
  → 100 items                    ← SPLIT: 1 image → 100 contours
Running step: Circle Filtering (processing 100 items)
  → 40 items                     ← FILTER: 100 → 40 circles
Running step: White Circle Filtering (processing 40 items)
  → 40 items                     ← FILTER: all 40 are white
Running step: OCR Recognition (processing 40 items)
  → 31 items                     ← FILTER: 31 have text
```

## Backward Compatibility

The old `DetectionPipeline` API is still available:

```rust
use addrslips::DetectionPipeline;

let pipeline = DetectionPipeline::new().with_verbose(true);
let detections = pipeline.detect(&img)?;
```

However, the new `Pipeline` API is recommended for all new code as it provides more flexibility and composability.
