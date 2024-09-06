
type WasmModule = typeof import("../wasm/exocore_client_web");
type ExocoreClient = import("../wasm/exocore_client_web").ExocoreClient;
type Discovery = import("../wasm/exocore_client_web").Discovery;
type LocalNode = import("../wasm/exocore_client_web").LocalNode;
type WatchedQuery = import("../wasm/exocore_client_web").WatchedQuery;

export { WasmModule, ExocoreClient, Discovery, LocalNode, WatchedQuery };

var module: WasmModule = null;

export async function getOrLoadModule(): Promise<WasmModule> {
    if (module == null) {
        module = await import('../wasm/exocore_client_web');
    }

    return module;
}

export function getModule(): WasmModule {
    if (!module) {
        throw 'module is not loaded. call `getOrLoadModule` first';
    }

    return module;
}