// TODO: Only if feature
pub mod json_store;

trait SimpleStore {
    fn read();
    fn write();
}

pub fn get_platform_store() {
    // TODO: If WASM, then get data from somewhere
}
