use oombak_sim::sim;
use oombak_tui::{components, event, render, tui};
use std::sync::{mpsc, Arc, RwLock};

fn main() {
    let terminal = tui::init_terminal().unwrap();

    let (message_tx, message_rx) = mpsc::channel();
    let mut simulator = sim::Simulator::new().unwrap();

    let command_line =
        components::CommandLine::new(message_tx.clone(), simulator.get_request_channel());
    let command_line = Arc::new(RwLock::new(command_line));

    let command_line_clone = Arc::clone(&command_line);
    simulator.register_listener(command_line_clone);

    let root = components::Root::new(message_tx, simulator.get_request_channel(), command_line);
    let root = Arc::new(RwLock::new(root));

    let root_clone = Arc::clone(&root);
    simulator.register_listener(root_clone);

    let root_clone = Arc::clone(&root);
    event::register_event_listener(root);

    event::spawn_event_loop();
    render::spawn_renderer(root_clone, terminal, message_rx)
        .join()
        .unwrap()
        .unwrap();

    simulator
        .get_request_channel()
        .send(sim::Request::Terminate)
        .unwrap();
    tui::restore_terminal().unwrap();
}
