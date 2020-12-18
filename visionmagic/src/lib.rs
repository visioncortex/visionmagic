//! Collection of vision & graphics algorithms
pub use visioncortex;

pub mod aggregation;
pub mod cluster_stat;
pub mod clustering;
mod pipeline;
pub mod segmentation;
pub mod simplification;

pub use aggregation::Processor as Aggregation;
pub use cluster_stat::Processor as ClusterStat;
pub use clustering::Processor as Clustering;
pub use pipeline::*;
pub use segmentation::Processor as Segmentation;
pub use simplification::Processor as Simplification;