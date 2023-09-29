use wasm_bindgen::prelude::*;
use serde_wasm_bindgen::to_value;

#[wasm_bindgen]
pub struct BroswerInterface {
    action_listener: js_sys::Function,
}

#[wasm_bindgen]
impl BroswerInterface {
    pub fn new(action_listener: js_sys::Function) -> BroswerInterface {
        println!("BroswerInterface::new");
        utils::set_panic_hook();
        BroswerInterface {
            action_listener
        }
    }
}


impl BroswerInterface {
    pub fn handle_action(&self, action: Action) {
        let this = JsValue::null();
        let value = to_value(&action).unwrap();
        let _ = self.action_listener.call1(&this, &value);
    }
}

