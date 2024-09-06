import sourceMapSupport from "source-map-support";
import * as electron from "electron";

// see https://gist.github.com/earksiinni/053470a04defc6d7dfaacd5e5a073b15 
// and https://stackoverflow.com/questions/57807459/how-to-use-preload-js-properly-in-electron
electron.contextBridge.exposeInMainWorld("exoElectron", {
  installSourceMap: () => {
    sourceMapSupport.install();
  },

  openPopup: (path) => {
    electron.ipcRenderer.send("open-popup", path.toString());
  },

  openExternal: (url) => {
    electron.shell.openExternal(url);
  },

  closeWindow: () => {
    electron.ipcRenderer.send("close-window");
  },

  onNavigate: (callback) => {
    electron.ipcRenderer.on("navigate", callback);
  },
});
