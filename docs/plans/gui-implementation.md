# GUI Implementation Plan for Addrslips Campaign Canvassing Tool

## Full Application Scope

A complete campaign canvassing management application with:
1. **Project Management**: Create/save/load projects with global settings (self-contained files)
2. **Area Management**: Import map screenshots (embedded), detect house numbers, user corrections
3. **Street Detection**: Automatic or manual street line drawing with corrections
4. **Address-Street Assignment**: Link house numbers to streets (auto + manual)
5. **Flat Estimation**: User input for estimated flats per address
6. **Team Assignment**: Algorithm-based team splitting with geographic clustering
7. **Export**: Generate printable address slips (PDF) with area color markers

## Plan Purpose & Usage

This plan is designed to be:
- **Version controlled**: Stored in git repository for cross-PC development
- **Session-independent**: Each phase can be implemented in separate Claude sessions
- **Self-contained**: Future Claude sessions can reference this plan for implementation
- **Collaborator-friendly**: Team members can pick up any phase

**To use this plan in a new Claude session**:
```bash
# Reference specific phases:
"Please implement Phase 5 from docs/plans/gui-implementation.md"

# Or continue from a checkpoint:
"Continue from Phase 12 according to the GUI implementation plan"

# Or ask for the next phase:
"What's the next phase in the GUI implementation plan?"
```

## Framework Recommendation: **iced** (with Tauri as alternative)

### Why iced?

**Pros for this project**:
- ✅ **Elm Architecture**: Perfect for complex multi-step workflows with clear state management
- ✅ **Canvas Widget**: Built-in support for drawing streets on images
- ✅ **Pure Rust**: Single portable binary (Windows/Linux/macOS)
- ✅ **Image Viewer Widget**: Good support for image display with overlays
- ✅ **Declarative UI**: Easier to maintain complex layouts than egui's immediate mode
- ✅ **Type-safe state**: Compile-time guarantees for state transitions
- ✅ **Good widget library**: Forms, buttons, scrollables, text inputs all built-in

**Cons to be aware of**:
- ⚠️ Larger binary than egui (~15-30 MB vs ~5-10 MB)
- ⚠️ Less mature than egui (but stable enough for production)
- ⚠️ Custom drawing requires understanding Canvas API

**Alternative: Tauri** (if comfortable with web technologies):
- Maximum flexibility for complex UIs (React/Vue/Svelte)
- Excellent image manipulation (HTML5 Canvas)
- Larger binary (~40-60 MB) but better UX capabilities
- Requires JavaScript knowledge

**Recommendation**: Start with **iced**. It's the sweet spot between portability and functionality for this use case.

---

## Architecture Overview

### Data Model

```
Project (saved as single .addrslips file)
├── metadata
│   ├── name: String
│   ├── created: DateTime
│   └── settings
│       └── target_flats_per_team: u32
│
└── areas: Vec<Area>
    ├── id: Uuid
    ├── name: String
    ├── color: AreaColor { r, g, b } (for PDF marking)
    ├── image_data: Vec<u8> (embedded PNG/JPEG, base64 encoded in JSON)
    ├── image_width: u32
    ├── image_height: u32
    ├── state: AreaState (enum tracking workflow progress)
    │
    ├── addresses: Vec<Address>
    │   ├── id: Uuid
    │   ├── house_number: String
    │   ├── position: Point { x, y } (pixel coordinates in image)
    │   ├── confidence: f32 (OCR confidence, 1.0 for manual)
    │   ├── verified: bool (user confirmed)
    │   ├── estimated_flats: Option<u32>
    │   └── assigned_street_id: Option<Uuid>
    │
    ├── streets: Vec<Street>
    │   ├── id: Uuid
    │   ├── name: Option<String>
    │   ├── polyline: Vec<Point> (line segments in image coordinates)
    │   ├── detection_method: DetectionMethod (Auto | Manual)
    │   └── verified: bool
    │
    └── teams: Vec<TeamAssignment>
        ├── team_id: u32
        ├── assigned_addresses: Vec<Uuid> (refs to Address.id)
        ├── total_flats: u32
        └── boundary: Option<Polygon> (for visualization)
```

**AreaState enum**:
```rust
enum AreaState {
    Imported,           // Screenshot imported (image embedded)
    AddressesDetected,  // House numbers detected (pipeline ran)
    AddressesCorrected, // User verified addresses
    StreetsDetected,    // Streets identified
    StreetsCorrected,   // User verified streets
    AddressesAssigned,  // Addresses linked to streets
    FlatsEstimated,     // User entered flat counts
    TeamsAssigned,      // Algorithm assigned teams
    Complete,           // Ready for export
}
```

**Key Design Decisions**:
- **Embedded images**: Project file is self-contained, can be shared with collaborators
- **Area colors**: Used to mark PDF slips with colored circles for easy area identification in Aktivisti
- **Terminology**: "Address" instead of "Detection" (more user-friendly)

### File Structure After Refactoring

```
addrslips/
├── Cargo.toml
├── README.md
├── CLAUDE.md
│
├── src/
│   ├── main.rs              # Binary entry point (CLI + GUI modes)
│   ├── lib.rs               # Public API exports
│   │
│   ├── cli/                 # CLI-specific code
│   │   ├── mod.rs
│   │   └── commands.rs      # CLI command handlers
│   │
│   ├── gui/                 # GUI application (iced)
│   │   ├── mod.rs
│   │   ├── app.rs           # Main iced Application
│   │   ├── state.rs         # Application state management
│   │   ├── message.rs       # iced Messages (events)
│   │   │
│   │   ├── views/           # Different screens
│   │   │   ├── mod.rs
│   │   │   ├── project_list.rs      # Home screen
│   │   │   ├── project_view.rs      # Project details
│   │   │   ├── area_detection.rs    # Detection correction
│   │   │   ├── street_drawing.rs    # Street drawing tool
│   │   │   ├── address_assignment.rs # Link addresses to streets
│   │   │   ├── flat_estimation.rs   # Enter flat counts
│   │   │   └── team_assignment.rs   # Team splitting UI
│   │   │
│   │   ├── widgets/         # Custom iced widgets
│   │   │   ├── mod.rs
│   │   │   ├── image_canvas.rs      # Image viewer with overlays
│   │   │   ├── address_marker.rs    # Circle markers for addresses
│   │   │   ├── street_polyline.rs   # Street line rendering
│   │   │   └── team_boundary.rs     # Team area visualization
│   │   │
│   │   └── algorithms/      # Business logic
│   │       ├── mod.rs
│   │       ├── street_detection.rs  # Auto street finding
│   │       ├── address_matching.rs  # Link addresses to streets
│   │       └── team_clustering.rs   # Team assignment algorithm
│   │
│   ├── core/                # Shared business logic
│   │   ├── mod.rs
│   │   ├── project.rs       # Project, Area data structures
│   │   ├── address.rs       # Address, Street, Team types
│   │   ├── persistence.rs   # Save/load from JSON
│   │   └── export.rs        # PDF generation
│   │
│   ├── pipeline/            # Detection pipeline (existing)
│   │   ├── mod.rs
│   │   ├── steps.rs
│   │   ├── preprocessing.rs
│   │   ├── contours.rs
│   │   ├── circles.rs
│   │   └── ocr.rs
│   │
│   └── utils/               # Utilities
│       ├── mod.rs
│       └── geometry.rs      # Point, Polygon, distance calculations
│
└── examples/
    └── gui_demo.rs          # GUI demo with sample data
```

### iced Application Structure

```rust
// src/gui/app.rs
use iced::{Application, Command, Element};

pub struct AddrslipsApp {
    current_view: View,
    state: AppState,
}

enum View {
    ProjectList,
    ProjectView { project_id: Uuid },
    AreaWorkflow {
        project_id: Uuid,
        area_id: Uuid,
        step: WorkflowStep
    },
}

enum WorkflowStep {
    AddressCorrection,
    StreetDrawing,
    AddressAssignment,
    FlatEstimation,
    TeamAssignment,
}

#[derive(Debug, Clone)]
enum Message {
    // Project management
    CreateProject,
    LoadProject(PathBuf),
    SaveProject,

    // Area management
    ImportArea(PathBuf),
    RunDetection(Uuid),

    // Address correction
    ToggleAddress(Uuid),
    AddManualAddress(Point, String),
    DeleteAddress(Uuid),

    // Street drawing
    StartDrawingStreet,
    AddStreetPoint(Point),
    FinishStreet,
    DeleteStreet(Uuid),
    RunAutoStreetDetection,

    // Address assignment
    AssignAddressToStreet(Uuid, Uuid),
    RunAutoAssignment,

    // Flat estimation
    UpdateFlatCount(Uuid, u32),

    // Team assignment
    RunTeamAssignment,
    MoveAddressToTeam(Uuid, u32),

    // Export
    ExportToPDF,

    // Navigation
    NavigateTo(View),

    // Canvas interaction
    CanvasClick(Point),
    CanvasDrag(Point, Point),
}

impl Application for AddrslipsApp {
    type Message = Message;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        // Initialize app
    }

    fn title(&self) -> String {
        "Addrslips - Campaign Canvassing Tool".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            // Handle all messages
        }
    }

    fn view(&self) -> Element<Message> {
        match &self.current_view {
            View::ProjectList => views::project_list::view(&self.state),
            View::ProjectView { project_id } =>
                views::project_view::view(&self.state, project_id),
            View::AreaWorkflow { project_id, area_id, step } =>
                self.area_workflow_view(project_id, area_id, step),
        }
    }
}
```

---

## Implementation Phases (Incremental Development)

**How to Use This Plan in Future Sessions**:

This plan is designed to be committed to the git repository and used across different development sessions and machines. Each phase is detailed enough to be implemented independently.

**To implement a specific phase**:
```bash
# In a new Claude session, reference the plan:
"Please implement Phase 5 from docs/plans/gui-implementation.md"

# Or reference by description:
"Implement Phase 12 (Street Correction UI) from the GUI implementation plan"
```

**Phase Structure**:
- **Goal**: What this phase accomplishes
- **Implementation Steps**: Detailed code and file changes
- **Files Created/Modified**: Which files to touch
- **Acceptance Criteria**: Checklist to verify completion

**Note**: Phases 1, 2, and 20 are shown with full implementation detail. All other phases should be implemented with similar level of detail when executing them.

### Phase Group 1: Foundation (Phases 1-4)

#### **Phase 1: Project Setup & iced Skeleton**

**Goal**: Set up iced framework and create a basic window with dual CLI/GUI mode support.

**Implementation Steps**:

1. Add iced dependencies to `Cargo.toml`:
```toml
[dependencies]
# ... existing dependencies ...

# GUI Framework
iced = { version = "0.12", features = ["canvas", "image", "tokio"] }
iced_aw = "0.9"  # Additional widgets

# File Dialogs
rfd = "0.14"
```

2. Update `src/main.rs` for dual mode:
```rust
use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "addrslips")]
#[command(about = "Campaign canvassing address management tool")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch GUI application
    Gui,
    /// Run CLI detection (existing functionality)
    Detect {
        #[arg(help = "Path to input image file")]
        image: String,
        #[arg(short, long, help = "Enable verbose output")]
        verbose: bool,
        #[arg(long, help = "Save debug outputs to directory")]
        debug_out: Option<String>,
        #[arg(long, help = "Skip OCR step")]
        skip_ocr: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Gui) | None => {
            // Default to GUI if no command specified
            #[cfg(feature = "gui")]
            {
                gui::run()?;
            }
            #[cfg(not(feature = "gui"))]
            {
                eprintln!("GUI not available in this build");
                std::process::exit(1);
            }
        }
        Some(Commands::Detect { image, verbose, debug_out, skip_ocr }) => {
            // Existing CLI logic (will be refactored in Phase 5)
            run_cli_detection(image, verbose, debug_out, skip_ocr)?;
        }
    }

    Ok(())
}

#[cfg(feature = "gui")]
mod gui {
    use iced::{Application, Settings};
    use anyhow::Result;

    pub fn run() -> Result<()> {
        crate::gui::AddrslipsApp::run(Settings::default())?;
        Ok(())
    }
}

fn run_cli_detection(image: String, verbose: bool, debug_out: Option<String>, skip_ocr: bool) -> Result<()> {
    // Existing CLI detection code (will be refactored in Phase 5)
    println!("CLI mode - existing functionality (to be refactored)");
    Ok(())
}
```

3. Create `src/gui/mod.rs`:
```rust
mod app;
mod message;
mod state;

pub use app::AddrslipsApp;
pub use message::Message;
pub use state::AppState;
```

4. Create `src/gui/message.rs`:
```rust
#[derive(Debug, Clone)]
pub enum Message {
    // Placeholder messages for Phase 1
    None,
}
```

5. Create `src/gui/state.rs`:
```rust
use crate::core::Project;

#[derive(Debug, Clone)]
pub struct AppState {
    pub current_project: Option<Project>,
    pub recent_projects: Vec<String>,  // File paths
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_project: None,
            recent_projects: Vec::new(),
        }
    }
}
```

6. Create `src/gui/app.rs`:
```rust
use iced::{Application, Command, Element, Settings, Theme};
use iced::widget::{column, container, text};
use super::{Message, AppState};

pub struct AddrslipsApp {
    state: AppState,
}

impl Application for AddrslipsApp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self {
                state: AppState::default(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Addrslips - Campaign Canvassing Tool".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::None => Command::none(),
        }
    }

    fn view(&self) -> Element<Message> {
        let content = column![
            text("Addrslips").size(32),
            text("Campaign Canvassing Address Management"),
            text("GUI coming soon!"),
        ]
        .spacing(20)
        .padding(20);

        container(content)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
```

7. Update `Cargo.toml` to add GUI feature flag:
```toml
[features]
default = ["gui"]
gui = ["iced", "iced_aw", "rfd"]
```

8. Update `src/lib.rs` to export GUI module:
```rust
#[cfg(feature = "gui")]
pub mod gui;

pub mod core;  // Will be created in Phase 2
```

**Testing**:
```bash
# Test GUI mode
cargo run

# Test CLI mode (existing)
cargo run -- detect image.png

# Test without GUI feature
cargo run --no-default-features
```

**Files Created**:
- `src/gui/mod.rs`
- `src/gui/app.rs`
- `src/gui/message.rs`
- `src/gui/state.rs`

**Files Modified**:
- `src/main.rs` - Dual mode support
- `Cargo.toml` - iced dependencies + feature flags
- `src/lib.rs` - GUI module export

**Acceptance Criteria**:
- [ ] `cargo run` opens an iced window with "GUI coming soon!" message
- [ ] `cargo run -- detect image.png` runs CLI detection (existing code)
- [ ] Window has title "Addrslips - Campaign Canvassing Tool"
- [ ] No compilation errors or warnings
- [ ] Binary compiles for Windows (primary target)

#### **Phase 2: Data Model & Persistence**

**Goal**: Create the core data structures with embedded images and implement save/load functionality.

**Implementation Steps**:

1. Create `src/core/mod.rs`:
```rust
pub mod address;
pub mod project;
pub mod persistence;

pub use address::{Address, Street, TeamAssignment};
pub use project::{Project, Area, AreaState, AreaColor};
pub use persistence::{save_project, load_project};
```

2. Create `src/core/address.rs`:
```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub id: Uuid,
    pub house_number: String,
    pub position: Point,
    pub confidence: f32,
    pub verified: bool,
    pub estimated_flats: Option<u32>,
    pub assigned_street_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Street {
    pub id: Uuid,
    pub name: Option<String>,
    pub polyline: Vec<Point>,
    pub detection_method: DetectionMethod,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DetectionMethod {
    Auto,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamAssignment {
    pub team_id: u32,
    pub assigned_addresses: Vec<Uuid>,
    pub total_flats: u32,
    pub boundary: Option<Vec<Point>>,
}
```

3. Create `src/core/project.rs`:
```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use super::address::{Address, Street, TeamAssignment};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub metadata: ProjectMetadata,
    pub areas: Vec<Area>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub name: String,
    pub created: DateTime<Utc>,
    pub settings: ProjectSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub target_flats_per_team: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Area {
    pub id: Uuid,
    pub name: String,
    pub color: AreaColor,
    pub image_data: Vec<u8>,  // PNG/JPEG bytes
    pub image_width: u32,
    pub image_height: u32,
    pub state: AreaState,
    pub addresses: Vec<Address>,
    pub streets: Vec<Street>,
    pub teams: Vec<TeamAssignment>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AreaColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AreaState {
    Imported,
    AddressesDetected,
    AddressesCorrected,
    StreetsDetected,
    StreetsCorrected,
    AddressesAssigned,
    FlatsEstimated,
    TeamsAssigned,
    Complete,
}

impl Project {
    pub fn new(name: String, target_flats_per_team: u32) -> Self {
        Self {
            metadata: ProjectMetadata {
                name,
                created: Utc::now(),
                settings: ProjectSettings { target_flats_per_team },
            },
            areas: Vec::new(),
        }
    }
}

impl Area {
    pub fn from_image(name: String, image_bytes: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            color: Self::generate_random_color(),
            image_data: image_bytes,
            image_width: width,
            image_height: height,
            state: AreaState::Imported,
            addresses: Vec::new(),
            streets: Vec::new(),
            teams: Vec::new(),
        }
    }

    fn generate_random_color() -> AreaColor {
        // Generate distinct colors (can be improved with better palette)
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        Uuid::new_v4().hash(&mut hasher);
        let hash = hasher.finish();
        AreaColor {
            r: ((hash >> 16) & 0xFF) as u8,
            g: ((hash >> 8) & 0xFF) as u8,
            b: (hash & 0xFF) as u8,
        }
    }
}
```

4. Create `src/core/persistence.rs`:
```rust
use anyhow::{Context, Result};
use std::path::Path;
use std::fs;
use super::project::Project;

pub fn save_project(project: &Project, path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(project)
        .context("Failed to serialize project")?;

    fs::write(path, json)
        .context("Failed to write project file")?;

    Ok(())
}

pub fn load_project(path: &Path) -> Result<Project> {
    let json = fs::read_to_string(path)
        .context("Failed to read project file")?;

    let project: Project = serde_json::from_str(&json)
        .context("Failed to deserialize project")?;

    Ok(project)
}
```

5. Add dependencies to `Cargo.toml`:
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["serde", "v4"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"  # Already exists
```

6. Create unit test in `src/core/persistence.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::project::{Project, Area};
    use std::path::PathBuf;

    #[test]
    fn test_save_load_roundtrip() {
        let mut project = Project::new("Test Project".to_string(), 20);

        // Add a test area with fake image data
        let area = Area::from_image(
            "Test Area".to_string(),
            vec![0x89, 0x50, 0x4E, 0x47], // PNG magic bytes
            800,
            600,
        );
        project.areas.push(area);

        // Save
        let temp_path = PathBuf::from("test_project.addrslips");
        save_project(&project, &temp_path).expect("Failed to save");

        // Load
        let loaded = load_project(&temp_path).expect("Failed to load");

        // Verify
        assert_eq!(loaded.metadata.name, "Test Project");
        assert_eq!(loaded.metadata.settings.target_flats_per_team, 20);
        assert_eq!(loaded.areas.len(), 1);
        assert_eq!(loaded.areas[0].name, "Test Area");
        assert_eq!(loaded.areas[0].image_width, 800);

        // Cleanup
        std::fs::remove_file(temp_path).ok();
    }
}
```

**Files Created**:
- `src/core/mod.rs`
- `src/core/project.rs`
- `src/core/address.rs`
- `src/core/persistence.rs`

**Acceptance Criteria**:
- [ ] All structs compile without errors
- [ ] Test `test_save_load_roundtrip` passes
- [ ] Can serialize/deserialize project with embedded images
- [ ] Area colors are generated and stored

#### **Phase 3: Project List View**
- Home screen with recent projects
- "New Project" button
- "Open Project" file dialog (rfd crate)
- Project metadata display

**Files**: `src/gui/views/project_list.rs`

#### **Phase 4: Project View & Area List**
- Display project details
- List of areas in sidebar
- "Import Area" button (file picker for screenshots)
- Settings panel (target flats per team)

**Files**: `src/gui/views/project_view.rs`, `src/gui/state.rs`

---

### Phase Group 2: Detection Integration (Phases 5-7)

#### **Phase 5: Refactor Existing Pipeline as Library**
- Move pipeline code from main.rs to pipeline/ module
- Export clean API: `run_detection(image: &DynamicImage) -> Vec<Detection>`
- Keep CLI working with new structure
- Update lib.rs exports

**Files**: Refactor `src/main.rs` → `src/pipeline/*`, update `src/lib.rs`

#### **Phase 6: Image Canvas Widget**
- Custom iced widget to display image
- Zoom and pan controls
- Click detection (screen coords → image coords)
- Basic rendering with iced Canvas

**Files**: `src/gui/widgets/image_canvas.rs`

#### **Phase 7: Run Detection & Display Results**
- "Run Detection" button in area view
- Call existing pipeline from GUI
- Display addresses as circles on image
- Show address list in sidebar (number, confidence)

**Files**: `src/gui/views/area_detection.rs`, integrate with pipeline

---

### Phase Group 3: Address Correction (Phases 8-9)

#### **Phase 8: Manual Address Correction**
- Click circle to toggle verified/unverified
- Delete address (right-click or button)
- Edit house number (text input)
- Visual feedback (colors: green=verified, yellow=unverified, red=low confidence)

**Files**: Update `area_detection.rs`, `src/gui/widgets/address_marker.rs`

#### **Phase 9: Add Manual Addresses**
- "Add Address" mode
- Click image to place new address marker
- Dialog to enter house number
- Save to project

**Files**: Update `area_detection.rs` with drawing mode

---

### Phase Group 4: Street Detection (Phases 10-12)

#### **Phase 10: Manual Street Drawing**
- Street drawing mode (toggle button)
- Click to add points to polyline
- Double-click or button to finish
- Render streets as lines on canvas
- Delete street functionality

**Files**: `src/gui/views/street_drawing.rs`, `src/gui/widgets/street_polyline.rs`

#### **Phase 11: Street Detection Algorithm**
- Research: Line detection on map images (Hough transform? Edge clustering?)
- Implement basic automatic street detection
- Filter to only streets with nearby house numbers
- Save as Street objects with detection_method = Auto

**Files**: `src/gui/algorithms/street_detection.rs`

#### **Phase 12: Street Correction UI**
- Display auto-detected streets
- User can delete/modify polylines
- Drag points to adjust street lines
- Mark street as verified

**Files**: Update `street_drawing.rs` with edit mode

---

### Phase Group 5: Address-Street Assignment (Phases 13-14)

#### **Phase 13: Manual Address Assignment**
- Side-by-side view: Addresses list + Streets list
- Drag-and-drop address onto street (or click-based)
- Visual indication (color-code by street)
- Unassign functionality

**Files**: `src/gui/views/address_assignment.rs`

#### **Phase 14: Automatic Assignment Algorithm**
- Nearest-street algorithm (Euclidean distance)
- Consider street orientation and house number positions
- Match house numbers to most likely street
- User can override

**Files**: `src/gui/algorithms/address_matching.rs`

---

### Phase Group 6: Flat Estimation (Phase 15)

#### **Phase 15: Flat Count Input**
- List view of all addresses
- Text input next to each address for flat count
- Quick-fill options (e.g., "Most are single-family = 1")
- Validation (must be > 0)

**Files**: `src/gui/views/flat_estimation.rs`

---

### Phase Group 7: Team Assignment (Phases 16-18)

#### **Phase 16: Team Assignment Algorithm**
- Graph-based clustering algorithm
- Input: Target flats per team
- Strong bias for geographic connectivity (minimize split streets)
- Output: Team assignments with boundaries

**Algorithm approach**:
- Build graph: nodes = addresses, edges = proximity
- Weighted by: distance (lower = stronger edge), same street (bonus weight)
- Clustering: k-means or spectral clustering with connectivity constraint
- Post-process: Balance team sizes while respecting geographic constraints

**Files**: `src/gui/algorithms/team_clustering.rs`, `src/utils/geometry.rs`

#### **Phase 17: Team Boundary Visualization**
- Display team assignments with color-coded boundaries
- Convex hull or polygon around each team's addresses
- Team statistics panel (team ID, total flats, # addresses)

**Files**: `src/gui/widgets/team_boundary.rs`, update `views/team_assignment.rs`

#### **Phase 18: Manual Team Adjustment**
- Select address and reassign to different team
- Drag address between teams
- Real-time update of statistics
- Visual feedback for connectivity issues

**Files**: Update `team_assignment.rs` with edit mode

---

### Phase Group 8: Export (Phases 19-20)

#### **Phase 19: PDF Generation Setup**
- Research: printpdf vs genpdf vs typst
- Choose library (recommendation: **genpdf** for simplicity)
- Basic PDF structure: One page per team
- Address list with house numbers and estimated flats

**Files**: `src/core/export.rs`, add PDF dependency

#### **Phase 20: Address Slip Template with Area Color Markers**

**Goal**: Create PDF export with colored circles for area identification in Aktivisti app.

**Implementation Steps**:

1. Update `src/core/export.rs` with template rendering:
```rust
use genpdf::{Document, Element, SimplePageDecorator};
use genpdf::elements::{Paragraph, Text, Break};
use genpdf::fonts::FontFamily;
use genpdf::style::{Color, Style};
use anyhow::Result;
use std::path::Path;

use super::project::{Project, Area, AreaColor};
use super::address::Address;

pub fn export_to_pdf(project: &Project, output_path: &Path) -> Result<()> {
    // Load font
    let font_family = genpdf::fonts::from_files("./fonts", "LiberationSans", None)?;
    let mut doc = Document::new(font_family);

    doc.set_title(&project.metadata.name);
    doc.set_page_decorator(SimplePageDecorator::new());

    // Generate one page per team per area
    for area in &project.areas {
        for team in &area.teams {
            generate_team_page(&mut doc, area, team, &project.metadata.name)?;
        }
    }

    // Render to PDF file
    doc.render_to_file(output_path)?;
    Ok(())
}

fn generate_team_page(
    doc: &mut Document,
    area: &Area,
    team: &crate::core::address::TeamAssignment,
    project_name: &str,
) -> Result<()> {
    let mut page = doc.new_page();

    // Header with area color circle
    page.push(
        Paragraph::new(format!(
            "{}  - {} - Team {}",
            colored_circle_symbol(&area.color),
            area.name,
            team.team_id
        ))
        .styled(Style::new().with_font_size(16).bold())
    );

    page.push(Break::new(0.5));

    // Team statistics
    page.push(Paragraph::new(format!(
        "Total flats: {} | Addresses: {}",
        team.total_flats,
        team.assigned_addresses.len()
    )));

    page.push(Break::new(1.0));

    // Get addresses for this team
    let mut team_addresses: Vec<&Address> = area.addresses
        .iter()
        .filter(|addr| team.assigned_addresses.contains(&addr.id))
        .collect();

    // Sort by street, then by house number
    team_addresses.sort_by(|a, b| {
        // Group by street ID first
        match (a.assigned_street_id, b.assigned_street_id) {
            (Some(street_a), Some(street_b)) if street_a != street_b => {
                street_a.cmp(&street_b)
            }
            _ => {
                // Within same street, sort by house number (numeric if possible)
                let num_a = a.house_number.parse::<u32>().ok();
                let num_b = b.house_number.parse::<u32>().ok();
                match (num_a, num_b) {
                    (Some(n_a), Some(n_b)) => n_a.cmp(&n_b),
                    _ => a.house_number.cmp(&b.house_number),
                }
            }
        }
    });

    // Group addresses by street
    let mut current_street_id = None;
    for address in team_addresses {
        if address.assigned_street_id != current_street_id {
            current_street_id = address.assigned_street_id;

            // Street header
            if let Some(street_id) = address.assigned_street_id {
                if let Some(street) = area.streets.iter().find(|s| s.id == street_id) {
                    page.push(Break::new(0.5));
                    page.push(
                        Paragraph::new(
                            street.name.clone().unwrap_or_else(|| "Unnamed Street".to_string())
                        )
                        .styled(Style::new().with_font_size(12).bold())
                    );
                }
            }
        }

        // Address line
        let flats_text = address.estimated_flats
            .map(|f| format!(" ({} flats)", f))
            .unwrap_or_default();

        page.push(Paragraph::new(format!(
            "  {} {}{}",
            colored_circle_symbol(&area.color),
            address.house_number,
            flats_text
        )));
    }

    Ok(())
}

/// Returns a colored circle Unicode character
/// This is a simplified version - in production you might want to use actual
/// colored shapes or images in the PDF
fn colored_circle_symbol(color: &AreaColor) -> &'static str {
    // For now, use Unicode circles
    // TODO: Phase 21 - render actual colored circles using genpdf shapes
    "●"  // Could map colors to different circle styles
}

// Alternative implementation with actual colored circles:
fn add_colored_circle(doc: &mut impl Element, color: &AreaColor) -> Result<()> {
    // Use genpdf's drawing API to add a filled circle with RGB color
    // This would be implemented in Phase 21 polish phase
    // doc.add_shape(Circle::new(x, y, radius).fill(Color::Rgb(color.r, color.g, color.b)));
    Ok(())
}
```

2. Add export button to team assignment view (`src/gui/views/team_assignment.rs`):
```rust
// In the view() function:
use iced::widget::button;

let export_button = button("Export to PDF")
    .on_press(Message::ExportToPDF);
```

3. Add message handler in `src/gui/app.rs`:
```rust
Message::ExportToPDF => {
    if let Some(ref project) = self.state.current_project {
        // Open file save dialog
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name(&format!("{}.pdf", project.metadata.name))
            .add_filter("PDF", &["pdf"])
            .save_file()
        {
            match crate::core::export::export_to_pdf(project, &path) {
                Ok(_) => {
                    // Show success message (implement in Phase 21)
                    println!("PDF exported to {:?}", path);
                }
                Err(e) => {
                    // Show error message (implement in Phase 21)
                    eprintln!("Export failed: {}", e);
                }
            }
        }
    }
    Command::none()
}
```

4. Add PDF generation dependency to `Cargo.toml`:
```toml
[dependencies]
# ... existing dependencies ...
genpdf = "0.2"
```

5. Download font files (Liberation Sans is open source):
```bash
# Create fonts directory
mkdir fonts
# Download Liberation Sans from https://github.com/liberationfonts/liberation-fonts
# Place .ttf files in fonts/ directory
```

**Address Slip Example Output**:
```
● Weststraße - Team 1
Total flats: 20 | Addresses: 15

Hauptstraße
  ● 42 (2 flats)
  ● 44 (3 flats)
  ● 46 (1 flat)

Seitenstraße
  ● 1 (2 flats)
  ● 3 (2 flats)
  ● 5 (1 flat)
...
```

**Enhancement for Phase 21**:
- Replace Unicode circles with actual colored PDF shapes
- Add mini-map thumbnail showing team boundary
- Custom fonts and styling
- Company/party logo support

**Files Created/Modified**:
- `src/core/export.rs` - PDF generation logic
- `src/gui/views/team_assignment.rs` - Export button
- `src/gui/app.rs` - Export message handler
- `Cargo.toml` - genpdf dependency

**Acceptance Criteria**:
- [ ] Export button appears in team assignment view
- [ ] Clicking export opens native file save dialog
- [ ] Generated PDF contains one page per team
- [ ] Each address has area color indicator (● symbol)
- [ ] Addresses sorted by street and number
- [ ] Flat counts displayed where estimated
- [ ] PDF can be opened and printed
- [ ] Area name clearly visible on each slip for Aktivisti reference

---

### Phase Group 9: Polish & Testing (Phases 21-22)

#### **Phase 21: Error Handling & UX Polish**
- Loading spinners during detection
- Error messages (detection failed, file not found, etc.)
- Undo/redo for major operations
- Keyboard shortcuts
- Progress bars for batch operations

**Files**: Update all views with error handling

#### **Phase 22: Testing & Documentation**
- Integration tests for full workflow
- Example project with sample data
- User guide (README with screenshots)
- Cross-platform testing (Windows, Linux, macOS)
- Release build configuration

**Files**: `examples/gui_demo.rs`, update `README.md`

---

## Dependencies to Add

```toml
[dependencies]
# Existing
image = "0.25"
imageproc = "0.25"
ocrs = "0.12"
rten = "0.24"
anyhow = "1.0"
clap = { version = "4.5", features = ["derive"] }

# GUI Framework
iced = { version = "0.12", features = ["canvas", "image", "tokio"] }
iced_aw = "0.9"  # Additional widgets (file picker, tabs, etc.)

# Data Persistence
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["serde", "v4"] }

# File Dialogs
rfd = "0.14"  # Native file picker

# PDF Generation
genpdf = "0.2"  # Simple PDF generation

# Geometry & Algorithms
geo = "0.28"  # Geometric calculations (polygons, distances)
geo-types = "0.7"

# Utilities
chrono = { version = "0.4", features = ["serde"] }
```

**Binary size estimate**: ~25-35 MB (release build with optimizations)

---

## State Management Strategy

**iced Elm Architecture**:
- **Model** (State): `AppState` struct containing projects, current view, UI state
- **Update** (Logic): `update()` function handles `Message` enum variants
- **View** (UI): `view()` function renders current state to iced Elements

**Persistence**:
- Auto-save on major operations (detect, assign, etc.)
- Manual "Save" button for explicit saves
- Store projects as `.addrslips` JSON files
- Images embedded in project file (base64 encoded) for portability

**Undo/Redo** (Phase 21):
- Command pattern: Store operations as reversible commands
- Undo stack (limited to last 20 operations)
- Only for major operations (not every UI interaction)

---

## UI Layout (Main Views)

### 1. Project List View
```
┌─────────────────────────────────────────┐
│  Addrslips - Campaign Canvassing        │
├─────────────────────────────────────────┤
│                                          │
│  Recent Projects:                        │
│  ┌────────────────────────────────────┐ │
│  │ My Campaign 2026         [Open]    │ │
│  │ 3 areas, 245 addresses              │ │
│  │ Last modified: 2026-01-28           │ │
│  └────────────────────────────────────┘ │
│                                          │
│  ┌────────────────────────────────────┐ │
│  │ Test Project             [Open]    │ │
│  │ 1 area, 45 addresses                │ │
│  │ Last modified: 2026-01-20           │ │
│  └────────────────────────────────────┘ │
│                                          │
│  [New Project]  [Open Project...]       │
│                                          │
└─────────────────────────────────────────┘
```

### 2. Project View (Area List)
```
┌─────────────────────────────────────────────────────────────┐
│  My Campaign 2026                             [Save] [Export]│
├──────────────┬──────────────────────────────────────────────┤
│ Areas        │  Project Settings                            │
│              │                                               │
│ ✓ Weststraße │  Target flats per team: [20] ▼              │
│ ○ Ostplatz   │                                               │
│ ○ Nordpark   │  [Add Area...]                               │
│              │                                               │
│ [+ Import]   │  Area: Weststraße                            │
│              │  Status: ✓ Complete                          │
│              │  Addresses: 82 detected                      │
│              │  Teams: 4 assigned                           │
│              │                                               │
│              │  [View Workflow →]                           │
│              │                                               │
└──────────────┴──────────────────────────────────────────────┘
```

### 3. Address Detection & Correction View
```
┌─────────────────────────────────────────────────────────────┐
│  Weststraße - Step 1: Detect & Correct House Numbers        │
├──────────────┬──────────────────────────────────────────────┤
│ Addresses    │  [Image Canvas with overlays]                │
│              │                                               │
│ [Run Detect] │     ○ 42  ○ 44  ○ 46                        │
│ [Add Manual] │                                               │
│              │  ○ 41   [MAP IMAGE]   ○ 48                  │
│ ✓ 42 (0.95)  │                                               │
│ ✓ 44 (0.89)  │     ○ 43  ○ 45  ○ 47                        │
│ ○ 46 (0.65)  │                                               │
│ ✓ 48 (0.92)  │  Green = Verified                            │
│              │  Yellow = Unverified                         │
│ 82 total     │  Red = Low confidence                        │
│              │                                               │
│ [Next: Draw Streets →]                                       │
└──────────────┴──────────────────────────────────────────────┘
```

### 4. Street Drawing View
```
┌─────────────────────────────────────────────────────────────┐
│  Weststraße - Step 2: Draw Streets                          │
├──────────────┬──────────────────────────────────────────────┤
│ Streets      │  [Image Canvas with street lines]            │
│              │                                               │
│ [Auto Detect]│     ─────────────────────                   │
│ [Draw Manual]│     │ Hauptstraße    │                       │
│              │     ─────────────────────                   │
│ Hauptstr.    │                                               │
│  [Edit] [Del]│  ○ 42  ○ 44  ○ 46                          │
│              │                                               │
│ Seitenstr.   │     ─────────                                │
│  [Edit] [Del]│     │ Side│                                  │
│              │     ─────────                                │
│ 5 streets    │                                               │
│              │  Click to add points, double-click to finish │
│ [← Back]  [Next: Assign Addresses →]                        │
└──────────────┴──────────────────────────────────────────────┘
```

### 5. Team Assignment View
```
┌─────────────────────────────────────────────────────────────┐
│  Weststraße - Step 5: Assign Teams                          │
├──────────────┬──────────────────────────────────────────────┤
│ Teams        │  [Image Canvas with colored team boundaries] │
│              │                                               │
│ [Auto Assign]│   ╔═══════╗  ┌───────┐                      │
│              │   ║ Team 1║  │Team 2 │                      │
│ Team 1 (Red) │   ║ 18 ◯  ║  │ 21 ◯  │                      │
│  20 flats    │   ╚═══════╝  └───────┘                      │
│  15 addrs    │                                               │
│              │   ╭───────╮  ╔═══════╗                      │
│ Team 2 (Blue)│   │Team 4 │  ║ Team 3║                      │
│  22 flats    │   │ 19 ◯  │  ║ 24 ◯  ║                      │
│  16 addrs    │   ╰───────╯  ╚═══════╝                      │
│              │                                               │
│ Target: 20   │  Click address to reassign to another team   │
│              │                                               │
│ [← Back]  [Export to PDF →]                                 │
└──────────────┴──────────────────────────────────────────────┘
```

---

## Critical Files to Create/Modify

**New files** (~30 files):
- `src/gui/` directory (app, views, widgets, algorithms)
- `src/core/` directory (data model, persistence, export)
- `src/pipeline/` (refactor from main.rs)
- `src/utils/geometry.rs`

**Modified files**:
- `main.rs` - Dual CLI/GUI mode
- `Cargo.toml` - Add dependencies
- `lib.rs` - Export public API
- `CLAUDE.md` - Update with GUI architecture

---

## Verification & Testing

**End-to-end workflow test**:
1. Launch GUI: `cargo run --release`
2. Create new project "Test Campaign"
3. Import area (image.png)
4. Run detection → verify ~32 addresses shown
5. Manually add/correct addresses
6. Draw streets (auto or manual)
7. Assign addresses to streets
8. Enter flat estimates
9. Run team assignment → verify 4 teams suggested
10. Adjust team boundaries
11. Export PDF → verify address slips with area color markers generated

**Unit tests**:
- Persistence (save/load roundtrip)
- Team clustering algorithm
- Address-street matching
- Geometry calculations

**Platform testing**:
- Build on Windows, Linux, macOS
- Verify binary size and startup time
- Test file dialogs work natively

---

## Risks & Mitigation

**Risk 1: iced learning curve**
- Mitigation: Start simple, incremental complexity
- Reference: iced examples repo, tour example

**Risk 2: Team assignment algorithm complexity**
- Mitigation: Start with simple k-means, iterate if needed
- Can use external geo/clustering crates

**Risk 3: Large binary size**
- Mitigation: Optimize with `strip = true`, `lto = true` in release profile
- Accept 25-35 MB as reasonable for desktop app

**Risk 4: Street detection accuracy**
- Mitigation: Manual drawing as fallback, auto-detect is convenience feature
- Can improve algorithm iteratively

---

## Timeline Estimate

**Phases 1-4** (Foundation): ~3-4 days
**Phases 5-9** (Detection): ~4-5 days
**Phases 10-12** (Streets): ~3-4 days
**Phases 13-15** (Assignment & Flats): ~2-3 days
**Phases 16-18** (Teams): ~4-5 days
**Phases 19-20** (Export): ~2-3 days
**Phases 21-22** (Polish): ~2-3 days

**Total**: ~20-27 days of development

(Note: These are rough estimates assuming a few hours per day)

---

## Using This Plan Across Sessions and Machines

This plan is version-controlled in the git repository at `docs/plans/gui-implementation.md`.

**On any machine** (after cloning/pulling the repository):
```bash
# Reference the plan in a Claude session:
"Please implement Phase 1 from docs/plans/gui-implementation.md"

# Or ask Claude to continue where you left off:
"Continue with the next phase of the GUI implementation plan"

# Or check progress:
"Which phases of the GUI implementation are complete?"
```

**Keeping track of progress**:
- Update the phase checklist below as you complete phases
- Commit checklist updates to track progress across sessions
- Share progress with collaborators via git

**Phase Checklist** (track progress in project):
```markdown
## GUI Implementation Progress

### Foundation
- [ ] Phase 1: iced skeleton
- [ ] Phase 2: Data model & persistence
- [ ] Phase 3: Project list view
- [ ] Phase 4: Project view & area list

### Detection
- [ ] Phase 5: Refactor pipeline as library
- [ ] Phase 6: Image canvas widget
- [ ] Phase 7: Run detection & display
- [ ] Phase 8: Manual address correction
- [ ] Phase 9: Add manual addresses

### Streets
- [ ] Phase 10: Manual street drawing
- [ ] Phase 11: Auto street detection
- [ ] Phase 12: Street correction UI

### Assignment & Estimation
- [ ] Phase 13: Manual address assignment
- [ ] Phase 14: Auto assignment algorithm
- [ ] Phase 15: Flat count input

### Teams
- [ ] Phase 16: Team assignment algorithm
- [ ] Phase 17: Team boundary visualization
- [ ] Phase 18: Manual team adjustment

### Export
- [ ] Phase 19: PDF generation setup
- [ ] Phase 20: Address slip template with area colors

### Polish
- [ ] Phase 21: Error handling & UX
- [ ] Phase 22: Testing & documentation
```

**Key Benefits of This Approach**:
- ✅ Plan lives in git repository
- ✅ Can continue work on any machine
- ✅ Collaborators can pick up any phase
- ✅ Each phase is independently implementable
- ✅ Clear acceptance criteria for each phase
- ✅ Detailed enough for Claude to execute in future sessions

---

## Next Steps

1. ✅ Plan created and committed to git repository
2. Start with Phase 1: Basic iced app structure
3. Verify iced works on Windows (primary target platform)
4. Incrementally build through phases, checking off progress in the checklist above
5. Commit completed phases to track progress across sessions
