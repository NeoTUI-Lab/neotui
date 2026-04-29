// Component model
// Core abstractions for UI components

pub trait Component {
    fn render(&self);
    fn on_event(&mut self);
}
