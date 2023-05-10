The `egui` documentation isn't super clear about the differences in building for Wayland vs X11.

Do include `cargo.lock` in version control, as this is an application. See [here](https://doc.rust-lang.org/cargo/faq.html#why-do-binaries-have-cargolock-in-version-control-but-not-libraries)

**TODO** : overall GUI integrations:
* Menu bar
* Logos and icons
* Fonts
* Color schemes
* Move / hide / pop-out panels
* etc...

**TODO** : Try to get a [[code editor]] in the window. My first attempt will probably be to copy-modify-paste the code from the [demo](https://github.com/emilk/egui/blob/master/crates/egui_demo_lib/src/demo/code_editor.rs). But I also want to explore getting the editor logic from Lapce.

**TODO** : regarding the [[spec viewer]], for the 3D model, should I use something like [Bevy](https://github.com/bevyengine/bevy)? 