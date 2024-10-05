fn main() {
    window::build().title("my cool app").width(1240).height(720);

    let engine = Engine::new();
    let shell = Shell::new("my cool app", 1240, 720);

    // loading plugins.
    engine.load();
    engine.lood();
    engine.load();

    engine.with(shell)
    // nested Engine::with call on subsell when shell requests a popup.
    // internally this will be a special mpsc channel where the platform
    // channels can communicate with the engine. Since the engine
    // claims the shell it is able to run mutable functions.
}
