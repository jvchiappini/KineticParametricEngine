pub mod generator;
pub mod uv;
pub mod instance_vars;
pub mod text_overlay;

pub use generator::MaterialGenerator;
pub use uv::UvMapper;
pub use instance_vars::InstanceVarResolver;
pub use text_overlay::TextOverlayEngine;
