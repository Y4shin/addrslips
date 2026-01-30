pub mod detection;
pub mod models;
pub mod pipeline;

pub use models::{Contour, HouseNumberDetection};
pub use detection::DetectionPipeline;
pub use pipeline::{
    Pipeline, PipelineData, PipelineStep, PipelineContext,
    BoundingBox, MetadataValue, WorkItem, PipelineExecutor, DebugConfig
};
