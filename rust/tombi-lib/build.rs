fn main() {
    // napi-rs needs platform-specific linker arguments (e.g.
    // `-undefined dynamic_lookup` on macOS) to build the `.node` addon; the
    // other features build a plain library and need no setup.
    #[cfg(feature = "node")]
    napi_build::setup();
}
