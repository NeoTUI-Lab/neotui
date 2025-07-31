use neotui_core::components::{Button, Label, VBox};
use neotui_core::component::Component;
use neotui_core::event::{Event, SpecialKey};
use neotui_core::renderer::{BorderStyle, Renderer};

fn main() {
    let label1 = Box::new(Label::new("NeoTUI"));
    let label2 = Box::new(Label::new("Modern Terminal UI"));

    let button = Box::new(
        Button::new("Click me")
            .on_click(|| println!("[Button] Clicked!"))
            .with_style(BorderStyle::Rounded),
    );

    let mut vbox = VBox::with_children(vec![label1, label2, button]);

    let mut renderer = Renderer::new(80, 24);
    let area = renderer.area();

    // Primeira renderização
    renderer.clear();
    vbox.render(renderer.buffer_mut(), area);
    renderer.flush();

    // Simula [Enter] no botão com foco inicial
    let result = vbox.on_event(Event::Special(SpecialKey::Enter));
    println!("\nEvent result: {:?}", result);

    // Simula [Tab] para alternar foco
    println!("\n[Tab]");
    let _ = vbox.on_event(Event::Special(SpecialKey::Tab));
}