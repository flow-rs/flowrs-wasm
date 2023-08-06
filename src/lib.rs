mod flow;

use std::panic;

pub use self::flow::app_state;

use flow::app_state::AppState;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn parse_flow(flow: &str) {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let app_state: AppState = serde_json::from_str(flow).unwrap();
    assert!(app_state.nodes.len() == 4);
    app_state.run();
}
