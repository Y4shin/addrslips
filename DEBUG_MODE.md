# Pipeline Debug Mode

## Overview

The pipeline supports a debug mode that saves all intermediate outputs to disk with lineage tracking. This allows you to inspect what happens at each step and trace exactly which inputs produced which outputs.

## Enabling Debug Mode

```rust
use std::path::PathBuf;
use addrslips::Pipeline;

let debug_dir = PathBuf::from("debug_output");

let pipeline = Pipeline::new()
    .with_verbose(true)
    .with_debug(debug_dir)?  // Enable debug mode
    .add_step_boxed(Box::new(SomeStep))
    // ... more steps
    ;
```

### Requirements

- The debug directory must be **empty** or **non-existent**
- If non-existent, it will be created automatically
- If it exists and is not empty, an error is returned

## Directory Structure

Debug mode creates a structured directory hierarchy:

```
debug_output/
├── 00_input/                      # Original input image
│   └── 01.png
├── 01_grayscale_conversion/       # After first step
│   └── 01.png
├── 02_gaussian_blur/              # After second step
│   └── 01.png
├── 03_edge_detection/             # After third step
│   └── 01.png
├── 04_contour_detection/          # Step that SPLITS (1 → 100)
│   ├── 01-01-01-01.png           # Lineage: [1,1,1,1]
│   ├── 01-01-01-02.png           # Lineage: [1,1,1,2]
│   ├── 01-01-01-03.png           # Lineage: [1,1,1,3]
│   └── ... (100 files total)
├── 05_circle_filtering/           # Step that FILTERS (100 → 40)
│   ├── 01-01-01-01-01.png        # From contour 1
│   ├── 01-01-01-03-01.png        # From contour 3
│   ├── 01-01-01-05-01.png        # From contour 5
│   └── ... (40 files total)
└── 06_white_circle_filtering/     # Another FILTER (40 → 40)
    ├── 01-01-01-01-01-01.png
    ├── 01-01-01-03-01-01.png
    └── ... (40 files total)
```

## Lineage Tracking

Filenames encode the lineage - the path through the pipeline that produced that output.

### Format

```
<parent1>-<parent2>-<parent3>-...-<current>.png
```

Each number is 1-indexed and represents the item number at that step.

### Example Lineage

Consider a file: `01-01-01-03-01-01.png` in step 6

**Reading the lineage**:
1. `01` - Item 1 from step 1 (grayscale conversion)
2. `01` - Item 1 from step 2 (gaussian blur) - came from item 1 of previous step
3. `01` - Item 1 from step 3 (edge detection) - came from item 1 of previous step
4. `03` - Item 3 from step 4 (contour detection) - this step split 1 image into 100 contours
5. `01` - Item 1 from step 5 (circle filtering) - contour 3 passed the filter (was the 1st circle)
6. `01` - Item 1 from step 6 (white circle filtering) - circle 1 passed the brightness filter

**What this tells you**:
- You can trace back to see which contour (03) produced this final output
- The contour was circular (passed step 5)
- The contour was white (passed step 6)
- To debug why this particular circle was detected, inspect `01-01-01-03.png` in the contour detection folder

## Use Cases

### 1. Debugging Failed Detections

If you expect a circle to be detected but it's not in the final output:

1. Check `04_contour_detection/` - Was the region detected as a contour?
2. Check `05_circle_filtering/` - Did it pass the circularity test?
3. Check `06_white_circle_filtering/` - Did it pass the brightness test?

Each step shows you why an item was filtered out.

### 2. Tuning Parameters

Want to adjust circle detection parameters?

1. Look at `05_circle_filtering/` to see what passed as "circular"
2. Look at files that were filtered out (missing from step 5 but present in step 4)
3. Adjust `circularity_threshold`, `min_radius`, `max_radius` based on what you see

### 3. Understanding Splits

When a step produces multiple outputs from one input (like contour detection):

```
03_edge_detection/01.png
    ↓ (splits)
04_contour_detection/01-01-01-01.png
04_contour_detection/01-01-01-02.png
04_contour_detection/01-01-01-03.png
... (100 contours total)
```

Each contour shows exactly which region of the edge image it came from.

### 4. Tracking Specific Items

Want to know what happened to a specific contour?

1. Find it in `04_contour_detection/` - e.g., `01-01-01-15.png` (contour 15)
2. Look for `01-01-01-15-01.png` in `05_circle_filtering/`
   - If present: It passed the circle filter
   - If absent: It was filtered out (not circular enough)
3. Look for `01-01-01-15-01-01.png` in `06_white_circle_filtering/`
   - If present: It passed the brightness filter
   - If absent: It wasn't white enough

## Debug Mode with Sequential vs Executor

### Sequential Execution (`run()`)

Simple numbering without lineage:
- Files are numbered sequentially: `01.png`, `02.png`, `03.png`, etc.
- Faster, simpler output
- Good for basic debugging

```rust
let mut pipeline = Pipeline::new()
    .with_debug(debug_dir)?
    .add_step_boxed(Box::new(Step1))
    .add_step_boxed(Box::new(Step2));

pipeline.run(img)?;  // Sequential execution
```

### Executor Execution (`run_with_executor()`)

Full lineage tracking:
- Files show complete path: `01-01-01-03-01.png`
- Tracks exactly which parent produced which child
- Essential for understanding complex pipelines

```rust
let pipeline = Pipeline::new()
    .with_debug(debug_dir)?
    .add_step_boxed(Box::new(Step1))
    .add_step_boxed(Box::new(Step2));

pipeline.run_with_executor(img)?;  // Executor with lineage
```

## Example: Full Pipeline Debug

```rust
use addrslips::Pipeline;
use addrslips::detection::steps::*;
use std::path::PathBuf;

let debug_dir = PathBuf::from("debug_output");

let pipeline = Pipeline::new()
    .with_verbose(true)
    .with_debug(debug_dir.clone())?
    .add_step_boxed(Box::new(GrayscaleStep))
    .add_step_boxed(Box::new(BlurStep { sigma: 1.5 }))
    .add_step_boxed(Box::new(EdgeDetectionStep {
        low_threshold: 50.0,
        high_threshold: 100.0,
    }))
    .add_step_boxed(Box::new(ContourDetectionStep { min_area: 10 }))
    .add_step_boxed(Box::new(CircleFilterStep {
        min_radius: 10.0,
        max_radius: 200.0,
        circularity_threshold: 2.0,
    }))
    .add_step_boxed(Box::new(WhiteCircleFilterStep {
        brightness_threshold: 200.0,
    }));

// Use executor for full lineage tracking
let results = pipeline.run_with_executor(img)?;

println!("Results saved to: {}/", debug_dir.display());
```

Output:
```
Debug: saved 00_input/01.png
Running step: Grayscale Conversion (processing 1 items)
Debug: saved 02_grayscale_conversion/01-01.png
Running step: Gaussian Blur (processing 1 items)
Debug: saved 03_gaussian_blur/01-01-01.png
...
Debug: saved 06_white_circle_filtering/01-01-01-15-01-01.png
```

## Tips

1. **Start with verbose mode** (`with_verbose(true)`) to see step-by-step progress
2. **Use sequential execution first** for simple debugging - faster and clearer output
3. **Use executor execution** when you need to trace specific items through the pipeline
4. **Clean up debug directories** - they can get large (hundreds of MB for complex pipelines)
5. **Look at step directories in order** - if an item is missing, it was filtered out at that step
6. **Compare similar items** - why did item 15 pass but item 16 fail? Look at both images

## Limitations

- Debug mode only saves images, not metadata
- Large pipelines can generate thousands of files
- No automatic cleanup - you must manually remove debug directories
- Lineage tracking only works with executor-based execution

## Cleaning Up

```bash
# Remove entire debug directory
rm -rf debug_output/

# Or selectively remove large steps
rm -rf debug_output/04_contour_detection/  # Remove 100 contour images
```
