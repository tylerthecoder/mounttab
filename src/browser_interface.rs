use wasm_bindgen::prelude::*;
use crate::model::Action;
use serde_wasm_bindgen::to_value;

#[wasm_bindgen]
pub struct BroswerInterface {
    action_listener: js_sys::Function,
}

impl BroswerInterface {
    pub fn handle_action(&self, action: Action) {
        let this = JsValue::null();
        let value = to_value(&action).unwrap();
        let _ = self.action_listener.call1(&this, &value);
    }
}

