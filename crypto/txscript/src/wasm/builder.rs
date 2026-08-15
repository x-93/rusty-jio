use crate::script_builder::ScriptBuilder;

pub struct WasmScriptBuilder {
    inner: ScriptBuilder,
}

impl Default for WasmScriptBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl WasmScriptBuilder {
    pub fn new() -> Self {
        Self {
            inner: ScriptBuilder::new(),
        }
    }

    pub fn add_data(&mut self, data: &[u8]) -> &mut Self {
        self.inner.add_data(data);
        self
    }

    pub fn build(self) -> Vec<u8> {
        self.inner.into_vec()
    }
}
