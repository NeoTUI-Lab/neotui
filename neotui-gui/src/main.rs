use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};
use vte4::{Terminal, PtyFlags};
use vte4::prelude::*;
use gtk::glib;

fn main() {
    let app = Application::builder()
        .application_id("dev.neotui.gui")
        .build();

    app.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("NeoTUI GUI")
            .default_width(960)
            .default_height(600)
            .build();

        let terminal = Terminal::new();
        let argv = ["/bin/bash"];
        let envv: [&str; 0] = [];

        terminal.spawn_async(
            PtyFlags::DEFAULT,
            None,
            &argv,
            &envv,
            glib::SpawnFlags::DEFAULT,
            || {},
            -1,
            None::<&gtk::gio::Cancellable>,
            |res| {
                if let Err(e) = res {
                    eprintln!("Erro ao iniciar terminal: {e}");
                }
            },
        );

        window.set_child(Some(&terminal));
        window.present();
    });

    app.run();
}
