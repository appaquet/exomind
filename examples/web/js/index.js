import("../../../clients/wasm/pkg").then(module => {
    window.exocore_client = new module.ExocoreClient("ws://127.0.0.1:3340");
});
