# Addrslips - Campaign Canvassing Address Management

A Rust application for political campaign canvassing that detects house numbers from map images, enables grouping addresses into routes, and generates printable address slips for door-to-door campaigning.

## Full Vision

**Purpose**: Tool for Die Linke campaign organizers to plan door-to-door canvassing routes.

**Complete Workflow**:
1. **Import**: Load map images exported from Aktivisti (Die Linke's campaign management software)
2. **Detect**: Automatically find and read house numbers from white circular markers on the map
3. **Group**: Interactive UI to divide addresses into canvassing groups/routes
4. **Export**: Generate PDF address slips for each group to print and distribute to canvassers

## Development Status

**Detection Engine**: Phases 1-8 Complete ✅
**GUI Application**: Ready to begin (22-phase plan created)

The detection engine (computer vision pipeline for finding house numbers) is complete and functional. Development is now transitioning to building the full GUI application around it.

### How the Detection Engine Will Be Used

```
Aktivisti Export → Detection Engine → Structured Data → GUI Application
     (PNG/JPG)         (CLI tool)      (JSON/CSV)      (Group & Print)
```

**Detection Engine Output**:
```json
[
  {"house_number": "1", "x": 245, "y": 567, "confidence": 0.95},
  {"house_number": "3", "x": 298, "y": 542, "confidence": 0.89},
  {"house_number": "5", "x": 351, "y": 518, "confidence": 0.92}
]
```

This output will feed into the future GUI where users can group addresses and generate PDFs.

## Current Status

### Detection Engine: Complete ✅

**Phases Completed**: 1-8 (Detection engine with OCR and trait-based pipeline architecture)
- Detects white circular house number markers from map images
- 80% OCR success rate on test images (~32 of 40 circles)
- Pure Rust implementation with no external dependencies

### GUI Application: Implementation Starting

**Plan**: 22 phases organized in 9 groups (see [docs/plans/gui-implementation.md](docs/plans/gui-implementation.md))
**Next Phase**: GUI Phase 1 - iced framework setup and basic window
**Technology**: iced (Rust GUI framework) for cross-platform desktop application

### Usage

```bash
# Run full detection pipeline
cargo run -- image.png

# With verbose output
cargo run -- image.png --verbose

# Skip OCR (fast circle detection only)
cargo run -- image.png --skip-ocr --verbose

# With debug mode (saves all intermediate images with lineage tracking)
cargo run -- image.png --debug-out debug_output
```

**Current Detection Results** (on test image):
- 100 contours detected from edge detection
- 40 circular shapes found (circularity filter)
- 40 white circles identified (brightness filter)
- 32 house numbers successfully recognized with OCR (80% success rate)

## Architecture

**Pure Rust Stack** - No external system dependencies
- `image` + `imageproc`: Image processing
- `ocrs` + `rten`: Pure Rust OCR engine with ONNX runtime
- `clap`: CLI framework
- `anyhow`: Error handling

### Trait-Based Pipeline Architecture

The detection pipeline uses a composable trait-based architecture where each step implements `PipelineStep`:

```rust
pub trait PipelineStep: Send + Sync {
    fn process(&self, data: Vec<PipelineData>, context: &PipelineContext) -> Result<Vec<PipelineData>>;
    fn name(&self) -> &str;
}
```

Steps can:
- **Transform** (1→1): Convert images (grayscale, blur, edges)
- **Split** (1→many): Generate multiple outputs (contour detection: 1 image → 100 contours)
- **Filter** (many→fewer): Remove unwanted items (circle filter: 100 → 40)

### Detection Pipeline (9 Steps)

1. **Grayscale Conversion**: Convert to single-channel grayscale
2. **Gaussian Blur** (σ=1.5): Reduce noise for edge detection
3. **Edge Detection**: Canny edge detector (thresholds: 50/100)
4. **Contour Detection**: Connected component labeling with 10px padding (splits: 1→100)
5. **Circle Filtering**: Circularity (0.7-2.0) + aspect ratio (0.7-1.4) + radius (10-200px) (filters: 100→40)
6. **White Circle Filtering**: Brightness threshold (>200/255) (keeps: 40)
7. **Background Removal**: Circular mask + brightness filter (<150) removes outline
8. **Upscale**: Resize to 100x100px with aspect ratio preservation
9. **OCR Recognition**: Pure Rust OCR with `ocrs` (detects: ~32)

## Detection Engine Development (Phases 1-14)

**Status**: Core functionality complete (Phases 1-8)
**Optional**: Phases 9-14 available for enhancements (JSON output, batch processing, config, testing)

The detection engine was built as a **CLI tool** first to allow testing and validation before GUI integration. This approach is successful - the engine is functional and ready for GUI integration.

### Completed Phases

- ✅ **Phase 1**: Project skeleton with image loading
- ✅ **Phase 2**: Preprocessing pipeline (grayscale, blur)
- ✅ **Phase 3**: Canny edge detection
- ✅ **Phase 4**: Contour detection via connected components
- ✅ **Phase 5**: Circle filtering (circularity, aspect ratio, size)
- ✅ **Phase 6**: White circle validation (brightness filtering)
- ✅ **Phase 7**: OCR integration with `ocrs` (pure Rust)
- ✅ **Phase 8**: Trait-based pipeline architecture refactor + background removal + upscaling

### Upcoming Phases
- **Phase 9**: Output formatting (JSON, CSV)
- **Phase 10**: Batch processing with progress bars
- **Phase 11**: Configuration system (TOML)
- **Phase 12**: Testing & fixtures
- **Phase 13**: Cross-platform builds (Linux, macOS, Windows)
- **Phase 14**: Documentation & polish

Full plan: `~/.claude/plans/refactored-wishing-porcupine.md`

## CLI Options

```
Usage: addrslips [OPTIONS] <IMAGE>

Arguments:
  <IMAGE>  Path to input image file

Options:
  -v, --verbose          Enable verbose output
      --debug-out <DIR>  Save debug outputs to directory (must be empty)
      --skip-ocr         Skip OCR step (faster, for testing circle detection only)
  -h, --help             Print help
```

**Debug Mode**: When `--debug-out` is provided, the pipeline saves all intermediate images to the specified directory (e.g., `00_input/`, `01_grayscale_conversion/`, etc.), organized by step. The executor uses lineage tracking in filenames (e.g., `01-01-01-15-01.png` shows item came from contour 15).

**Pipeline Execution**: The pipeline uses an MPSC channel-based executor that processes items individually and tracks lineage through the pipeline steps.

## Files Structure

```
addrslips/
├── src/
│   └── main.rs           # All logic currently in single file (will be modularized)
├── examples/
│   └── create_test_image.rs  # Utility to generate test images
├── docs/
│   └── plans/
│       └── gui-implementation.md  # 22-phase GUI development plan
├── Cargo.toml            # Dependencies
├── flake.nix             # Nix development environment
├── image.png             # Test image with white circle house numbers
└── CLAUDE.md             # This file
```

**Future structure** (after GUI implementation):
```
addrslips/
├── src/
│   ├── main.rs           # Binary entry point (CLI + GUI modes)
│   ├── lib.rs            # Public API exports
│   ├── gui/              # GUI application (iced)
│   ├── core/             # Data model & persistence
│   ├── pipeline/         # Detection pipeline
│   └── utils/            # Shared utilities
```

See [docs/plans/gui-implementation.md](docs/plans/gui-implementation.md) for complete architecture.

## Key Implementation Details

### Circle Detection Algorithm

**Circularity Formula**: `perimeter² / (4π × area)`
- Perfect circle: 1.0
- Current threshold: 0.7 - 2.0
- Uses bounding box area for edge-based detection

**Filters Applied**:
1. Circularity: 0.7 - 2.0
2. Aspect ratio: 0.7 - 1.4 (roughly square)
3. Radius: 10 - 200 pixels
4. Minimum edge pixels: 10
5. Brightness: ≥ 200/255 (white circles only)

### Debug Output

Running with debug flags creates intermediate images:
- `*_grayscale.jpg`: Grayscale conversion
- `*_blurred.jpg`: After Gaussian blur
- `*_edges.jpg`: Canny edge detection result
- `*_contours.jpg`: All contours (red rectangles)
- `*_circles.jpg`: Filtered circles (green circles)

## Development Workflow

### Starting a New Session

1. Review phase plan: `cat ~/.claude/plans/addrslips/phase-N.md`
2. Build and test: `cargo run -- image.png --detect-circles --verbose`
3. Implement phase changes
4. Test thoroughly
5. Commit with phase completion message

### Git Commit Guidelines

- Write clear, descriptive commit messages
- Reference completed phase numbers when applicable
- **Do NOT include Co-Authored-By lines** (no AI attribution in commits)
- Focus on what was accomplished and why

### To Continue Development

**Detection Engine** (optional improvements):
- Phase 9-14 available for enhancements (JSON output, batch processing, config system, testing)
- Current engine is functional and can be integrated as-is

**GUI Application** (primary focus):
```bash
# Start GUI development:
"Please implement Phase 1 from docs/plans/gui-implementation.md"

# Or ask Claude:
"Let's start implementing the GUI according to the plan"
```

See full implementation plan: [docs/plans/gui-implementation.md](docs/plans/gui-implementation.md)

## Performance Notes

- Image size: 831x1068 pixels
- Processing time: ~3 seconds (including OCR)
- OCR engine: Initialized once and cached in OcrStep for performance
- Pure Rust compile time: ~10-15 seconds from clean
- No runtime dependencies needed

## GUI Application Plan

The full application is being built using the iced Rust GUI framework. See detailed implementation plan: [docs/plans/gui-implementation.md](docs/plans/gui-implementation.md)

### Key Features (22 Phases)

**Foundation** (Phases 1-4):
- Project management with embedded images
- Area color coding for PDF identification
- Save/load projects as self-contained `.addrslips` files

**Detection Integration** (Phases 5-9):
- Image canvas widget with zoom/pan
- Run detection pipeline from GUI
- Manual address correction and addition

**Street Management** (Phases 10-12):
- Manual street drawing tool
- Automatic street detection
- Street-based address grouping

**Team Assignment** (Phases 13-18):
- Automatic assignment of addresses to streets
- Flat count estimation per address
- Geographic clustering algorithm for team splitting
- Manual team boundary adjustment

**Export** (Phases 19-20):
- PDF generation with area color markers
- Printable address slips sorted by street

**Polish** (Phases 21-22):
- Error handling and UX improvements
- Testing and documentation

### Architecture

- **Framework**: iced (Pure Rust, cross-platform)
- **Data Model**: Project → Areas → Addresses/Streets/Teams
- **Persistence**: JSON files with embedded images
- **Binary Size**: ~25-35 MB (release build)

## Build Information

**Rust Edition**: 2024
**Target**: Cross-platform (Linux, macOS, Windows)
**Build**: `cargo build --release`
**Static linking**: Planned for Phase 13

## Test Image

`image.png`: OpenStreetMap-style screenshot showing ~30 white circular house number signs on a turquoise background with streets and buildings.

**Context**: This represents the typical format exported from Aktivisti - campaign management maps with house numbers marked as white circles. The detection engine needs to:
1. Identify these white circles (completed in Phase 5)
2. Filter by color to isolate house numbers (completed in Phase 6)
3. Read the numbers using OCR (Phases 7-8 - next)

**Output**: JSON/CSV with house numbers and their pixel coordinates on the image.

## Notes

- All detection parameters (thresholds, sizes) currently hardcoded
- Configuration system coming in Phase 11
- Code currently in single file; will modularize as it grows
- Using pure Rust for maximum portability
- No OpenCV dependency (intentional design choice)
