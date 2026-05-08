// Widget module
// Provides the MVP widget set built on top of the component contract

pub mod divider;
pub mod label;
pub mod panel;
pub mod spacer;

pub use divider::{Divider, DividerOrientation};
pub use label::Label;
pub use panel::Panel;
pub use spacer::Spacer;
