import("../../pkg").then(module => {
  window.exocore_client = new module.ExocoreClient();
});
