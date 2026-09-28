fn main() {
    // napi-rs needs platform-specific linker arguments (e.g.
    // `-undefined dynamic_lookup` on macOS) to build the `.node` addon.
    napi_build::setup();
}
