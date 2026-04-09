pub mod api;
pub mod claudemd;
pub mod events;
pub mod mcp;
pub mod memory;
pub mod plugins;
pub mod settings;
pub mod skills;

// Re-export the most commonly used types at the crate root for convenience.
pub use settings::{
    ConfigSource, EffectiveValue, HookDefinition, HookGroup, MarketplaceSource,
    MarketplaceSourceInfo, McpServerRef, ModelOverrides, Permissions, SandboxConfig, Settings,
    StatusLine,
};
pub use api::{
    ConfigResponse, ErrorResponse, HealthResponse, ProjectEntry, RegisterProjectRequest,
    UpdateConfigRequest, ValidationError,
};
pub use events::{CommandStream, WsClientMessage, WsEvent, WsValidationError};
