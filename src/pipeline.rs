use image::DynamicImage;
use std::sync::Arc;
use std::collections::HashMap;
use std::sync::mpsc::{self, Sender, Receiver};
use anyhow::Result;

/// Bounding box in the original image
#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Data that flows through the pipeline
/// Each PipelineData represents a single image region with associated metadata
#[derive(Clone)]
pub struct PipelineData {
    /// The image data (can be grayscale or color)
    pub image: DynamicImage,

    /// Reference to the original image (shared efficiently via Arc)
    pub original: Arc<DynamicImage>,

    /// Bounding box in the original image (None means full image)
    pub bbox: Option<BoundingBox>,

    /// Metadata for tracking properties (e.g., "is_circle", "brightness", etc.)
    pub metadata: HashMap<String, MetadataValue>,
}

/// Metadata value types
#[derive(Debug, Clone)]
pub enum MetadataValue {
    Bool(bool),
    Float(f32),
    String(String),
    Int(i32),
}

impl PipelineData {
    /// Create PipelineData for a full image
    pub fn from_image(image: DynamicImage) -> Self {
        let original = Arc::new(image.clone());
        Self {
            image,
            original,
            bbox: None,
            metadata: HashMap::new(),
        }
    }

    /// Create PipelineData for a region of an image
    pub fn from_region(
        image: DynamicImage,
        original: Arc<DynamicImage>,
        bbox: BoundingBox,
    ) -> Self {
        Self {
            image,
            original,
            bbox: Some(bbox),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: MetadataValue) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Get metadata as bool
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.metadata.get(key) {
            Some(MetadataValue::Bool(v)) => Some(*v),
            _ => None,
        }
    }

    /// Get metadata as float
    pub fn get_float(&self, key: &str) -> Option<f32> {
        match self.metadata.get(key) {
            Some(MetadataValue::Float(v)) => Some(*v),
            _ => None,
        }
    }

    /// Get metadata as string
    pub fn get_string(&self, key: &str) -> Option<&str> {
        match self.metadata.get(key) {
            Some(MetadataValue::String(v)) => Some(v.as_str()),
            _ => None,
        }
    }
}

/// Debug configuration for pipeline execution
#[derive(Clone, Debug)]
pub struct DebugConfig {
    /// Root directory for debug outputs
    pub output_dir: std::path::PathBuf,
    /// Whether debug mode is enabled
    pub enabled: bool,
}

/// Context available to all pipeline steps
#[derive(Clone)]
pub struct PipelineContext {
    pub verbose: bool,
    pub debug: Option<DebugConfig>,
}

/// Trait that all pipeline steps must implement
pub trait PipelineStep: Send + Sync {
    /// Process data and return transformed data
    /// Steps can split data (1 → many), filter (many → fewer), or transform (many → many)
    fn process(&self, data: Vec<PipelineData>, context: &PipelineContext) -> Result<Vec<PipelineData>>;

    /// Human-readable name for this step (used in verbose output)
    fn name(&self) -> &str;
}

/// Work item for pipeline execution
/// Contains data and the remaining steps to execute
#[derive(Clone)]
pub struct WorkItem {
    /// The data to process
    pub data: PipelineData,

    /// Remaining pipeline steps (steps not yet executed)
    pub remaining_steps: Vec<Arc<dyn PipelineStep>>,

    /// Step index (for tracking progress)
    pub current_step_index: usize,

    /// Lineage: IDs from previous steps that led to this item
    /// E.g., [1, 3, 2] means: item 1 from step 0 → item 3 from step 1 → item 2 from step 2
    pub lineage: Vec<usize>,
}

impl WorkItem {
    /// Create a new work item
    pub fn new(data: PipelineData, steps: Vec<Arc<dyn PipelineStep>>) -> Self {
        Self {
            data,
            remaining_steps: steps,
            current_step_index: 0,
            lineage: vec![],
        }
    }

    /// Check if this work item is complete (no more steps)
    pub fn is_complete(&self) -> bool {
        self.remaining_steps.is_empty()
    }

    /// Generate filename from lineage (e.g., "01-03-02.png")
    pub fn lineage_filename(&self, extension: &str) -> String {
        if self.lineage.is_empty() {
            format!("01.{}", extension)
        } else {
            let ids: Vec<String> = self.lineage.iter().map(|id| format!("{:02}", id)).collect();
            format!("{}.{}", ids.join("-"), extension)
        }
    }

    /// Save debug output if debug mode is enabled
    fn save_debug_output(&self, context: &PipelineContext, step_name: &str) -> Result<()> {
        if let Some(debug_config) = &context.debug {
            if !debug_config.enabled {
                return Ok(());
            }

            // Create step directory
            let step_dir_name = format!("{:02}_{}", self.current_step_index + 1,
                step_name.to_lowercase().replace(" ", "_"));
            let step_dir = debug_config.output_dir.join(&step_dir_name);
            std::fs::create_dir_all(&step_dir)?;

            // Save image
            let filename = self.lineage_filename("png");
            let output_path = step_dir.join(&filename);

            self.data.image.save(&output_path)
                .map_err(|e| anyhow::anyhow!("Failed to save debug image: {}", e))?;

            if context.verbose {
                println!("  Debug: saved {}/{}", step_dir_name, filename);
            }
        }

        Ok(())
    }

    /// Get the next step and create new work items for the remaining steps
    pub fn process_next_step(&mut self, context: &PipelineContext) -> Result<Vec<WorkItem>> {
        if self.remaining_steps.is_empty() {
            return Ok(vec![]);
        }

        // Take the first step
        let step = self.remaining_steps[0].clone();
        let remaining_after = self.remaining_steps[1..].to_vec();
        let step_name = step.name();

        // Process the step (this may split 1 item into many)
        let results = step.process(vec![self.data.clone()], context)?;

        // Create new work items for each result and assign IDs
        let mut new_items = Vec::new();
        for (idx, result_data) in results.into_iter().enumerate() {
            // Build new lineage: parent lineage + this item's ID
            let mut new_lineage = self.lineage.clone();
            new_lineage.push(idx + 1); // 1-indexed for readability

            let new_item = WorkItem {
                data: result_data,
                remaining_steps: remaining_after.clone(),
                current_step_index: self.current_step_index + 1,
                lineage: new_lineage,
            };

            // Save debug output for this item
            new_item.save_debug_output(context, step_name)?;

            new_items.push(new_item);
        }

        Ok(new_items)
    }
}

/// Pipeline executor using MPSC channel for work distribution
pub struct PipelineExecutor {
    sender: Sender<WorkItem>,
    receiver: Receiver<WorkItem>,
    context: PipelineContext,
}

impl PipelineExecutor {
    /// Create a new executor
    pub fn new(context: PipelineContext) -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver,
            context,
        }
    }

    /// Execute the pipeline by processing work items from the channel
    pub fn execute(&self, initial_items: Vec<WorkItem>) -> Result<Vec<PipelineData>> {
        // Send all initial work items
        for item in initial_items {
            self.sender.send(item)
                .map_err(|e| anyhow::anyhow!("Failed to send work item: {}", e))?;
        }

        let mut completed_results = Vec::new();
        let mut pending_count = 1; // Start with at least 1 item

        // Process work items until queue is empty
        while pending_count > 0 {
            match self.receiver.try_recv() {
                Ok(mut item) => {
                    pending_count -= 1;

                    if item.is_complete() {
                        // No more steps - this is a final result
                        completed_results.push(item.data);
                    } else {
                        // Process next step
                        let new_items = item.process_next_step(&self.context)?;

                        // Send new work items back to the queue
                        for new_item in new_items {
                            self.sender.send(new_item)
                                .map_err(|e| anyhow::anyhow!("Failed to send work item: {}", e))?;
                            pending_count += 1;
                        }
                    }
                }
                Err(mpsc::TryRecvError::Empty) => {
                    if pending_count == 0 {
                        break;
                    }
                    // Wait a bit if queue is empty but we expect more items
                    std::thread::yield_now();
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    break;
                }
            }
        }

        Ok(completed_results)
    }
}

/// Composable pipeline builder
pub struct Pipeline {
    steps: Vec<Arc<dyn PipelineStep>>,
    context: PipelineContext,
}

impl Pipeline {
    /// Create a new empty pipeline
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            context: PipelineContext {
                verbose: false,
                debug: None,
            },
        }
    }

    /// Enable verbose output
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.context.verbose = verbose;
        self
    }

    /// Enable debug mode with output directory
    /// The directory must be empty or non-existent
    pub fn with_debug(mut self, output_dir: std::path::PathBuf) -> Result<Self> {
        // Check if directory exists and is empty
        if output_dir.exists() {
            let entries = std::fs::read_dir(&output_dir)?;
            if entries.count() > 0 {
                return Err(anyhow::anyhow!(
                    "Debug directory is not empty: {}",
                    output_dir.display()
                ));
            }
        } else {
            // Create directory if it doesn't exist
            std::fs::create_dir_all(&output_dir)?;
        }

        self.context.debug = Some(DebugConfig {
            output_dir,
            enabled: true,
        });

        Ok(self)
    }

    /// Add a processing step to the pipeline
    pub fn add_step(mut self, step: Arc<dyn PipelineStep>) -> Self {
        self.steps.push(step);
        self
    }

    /// Helper method to add a step from a Box (for convenience)
    pub fn add_step_boxed(mut self, step: Box<dyn PipelineStep>) -> Self {
        self.steps.push(Arc::from(step));
        self
    }

    /// Run the pipeline sequentially on an input image (simple execution)
    pub fn run(&mut self, input: DynamicImage) -> Result<Vec<PipelineData>> {
        // Save initial input in debug mode
        if let Some(debug_config) = &self.context.debug {
            if debug_config.enabled {
                let input_dir = debug_config.output_dir.join("00_input");
                std::fs::create_dir_all(&input_dir)?;
                let input_path = input_dir.join("01.png");
                input.save(&input_path)
                    .map_err(|e| anyhow::anyhow!("Failed to save debug input: {}", e))?;
                if self.context.verbose {
                    println!("  Debug: saved 00_input/01.png");
                }
            }
        }

        // Start with a single PipelineData containing the full image
        let mut data = vec![PipelineData::from_image(input)];

        for (step_idx, step) in self.steps.iter().enumerate() {
            if self.context.verbose {
                println!("Running step: {} (processing {} items)", step.name(), data.len());
            }

            let step_name = step.name();
            data = step.process(data, &self.context)?;

            // Save debug outputs for this step
            if let Some(debug_config) = &self.context.debug {
                if debug_config.enabled {
                    let step_dir_name = format!("{:02}_{}", step_idx + 1,
                        step_name.to_lowercase().replace(" ", "_"));
                    let step_dir = debug_config.output_dir.join(&step_dir_name);
                    std::fs::create_dir_all(&step_dir)?;

                    for (idx, item) in data.iter().enumerate() {
                        let filename = format!("{:02}.png", idx + 1);
                        let output_path = step_dir.join(&filename);
                        item.image.save(&output_path)
                            .map_err(|e| anyhow::anyhow!("Failed to save debug image: {}", e))?;
                    }

                    if self.context.verbose {
                        println!("  Debug: saved {} images to {}/", data.len(), step_dir_name);
                    }
                }
            }

            if self.context.verbose {
                println!("  → {} items", data.len());
            }
        }

        Ok(data)
    }

    /// Run the pipeline using the executor with work queue
    /// This allows for more sophisticated execution patterns in the future
    pub fn run_with_executor(&self, input: DynamicImage) -> Result<Vec<PipelineData>> {
        // Save initial input in debug mode
        if let Some(debug_config) = &self.context.debug {
            if debug_config.enabled {
                let input_dir = debug_config.output_dir.join("00_input");
                std::fs::create_dir_all(&input_dir)?;
                let input_path = input_dir.join("01.png");
                input.save(&input_path)
                    .map_err(|e| anyhow::anyhow!("Failed to save debug input: {}", e))?;
                if self.context.verbose {
                    println!("  Debug: saved 00_input/01.png");
                }
            }
        }

        let initial_data = PipelineData::from_image(input);
        let initial_item = WorkItem::new(initial_data, self.steps.clone());

        let executor = PipelineExecutor::new(self.context.clone());
        executor.execute(vec![initial_item])
    }

    /// Run the pipeline but stop at an intermediate step (useful for debugging)
    pub fn run_partial(&mut self, input: DynamicImage, num_steps: usize) -> Result<Vec<PipelineData>> {
        let mut data = vec![PipelineData::from_image(input)];

        for (i, step) in self.steps.iter().enumerate() {
            if i >= num_steps {
                break;
            }
            if self.context.verbose {
                println!("Running step {}: {} (processing {} items)", i + 1, step.name(), data.len());
            }
            data = step.process(data, &self.context)?;
            if self.context.verbose {
                println!("  → {} items", data.len());
            }
        }

        Ok(data)
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}
