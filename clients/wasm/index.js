
export * from './proto/exocore/index/entity_pb';
export * from './proto/exocore/index/mutation_pb';
export * from './proto/exocore/index/query_pb';
export * from './proto/exocore/test/test_pb';

export function getClient() {
  return import("exocore-client-wasm").then((module) => {
    console.log("Wasm client loaded");
    return module;
  });
}


