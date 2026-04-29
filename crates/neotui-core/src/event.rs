// Event model
// Core abstractions for event handling

pub enum Event {
    Key(char),
    Mouse,
    Resize,
    Quit,
}
