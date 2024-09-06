"use strict";
/* eslint-env node */

import { app, BrowserWindow, ipcMain, Menu, MenuItem } from "electron";
import * as path from "path";

const isDevelopment = process.env.NODE_ENV !== "production";
const autoDevTool = false;

function createNewWindow() {
  const window = new BrowserWindow({
    webPreferences: {
      nodeIntegration: false,
      sandbox: false,
      contextIsolation: true,
      preload: path.resolve(__dirname, "preload.js"), // setup context isolation bridge
    },
    autoHideMenuBar: true,
  });
  window.setMenuBarVisibility(false);

  if (isDevelopment && autoDevTool) {
    window.webContents.openDevTools();
  }

  if (isDevelopment) {
    window.loadURL(`http://localhost:${process.env.ELECTRON_WEBPACK_WDS_PORT}`);
  } else {
    window.loadURL(`file://${path.join(__dirname, "index.html")}`);
  }

  return window;
}

let mainWindow;
function createMainWindow() {
  const window = createNewWindow();
  window.on("closed", () => {
    mainWindow = null;
  });

  window.webContents.on("devtools-opened", () => {
    window.focus();
    setImmediate(() => {
      window.focus();
    });
  });

  // From https://www.electronjs.org/docs/tutorial/spellchecker
  window.webContents.on("context-menu", (event, params) => {
    const menu = new Menu();

    // Add each spelling suggestion
    for (const suggestion of params.dictionarySuggestions) {
      menu.append(
        new MenuItem({
          label: suggestion,
          click: () => mainWindow.webContents.replaceMisspelling(suggestion),
        })
      );
    }

    // Allow users to add the misspelled word to the dictionary
    if (params.misspelledWord) {
      menu.append(
        new MenuItem({
          label: "Add to dictionary",
          click: () =>
            mainWindow.webContents.session.addWordToSpellCheckerDictionary(
              params.misspelledWord
            ),
        })
      );
    }

    menu.popup();
  });

  return window;
}

// quit application when all windows are closed
app.on("window-all-closed", () => {
  // on macOS it is common for applications to stay open until the user explicitly quits
  if (process.platform !== "darwin") {
    app.quit();
  }
});

app.on("activate", () => {
  // on macOS it is common to re-create a window even after all windows have been closed
  if (mainWindow === null) {
    mainWindow = createMainWindow();
  }
});

// create main BrowserWindow when electron is ready
app.on("ready", () => {
  mainWindow = createMainWindow();
});

ipcMain.on("open-popup", (_event, navPath) => {
  const window = createNewWindow();

  window.webContents.on("did-finish-load", () => {
    window.webContents.send("navigate", navPath);
  });
});

ipcMain.on("close-window", (event) => {
  BrowserWindow.fromWebContents(event.sender).close();
});
