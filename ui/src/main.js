/**
 * This is the minimal code that runs in Electron's main thread.
 *
 * It opens a browser window. In addition to the UI code, the native Rust code
 * runs in the renderer thread, not the main thread.
 */

/* eslint-env node */
/* eslint-disable @typescript-eslint/no-var-requires */

const { app, BrowserWindow, ipcMain } = require("electron");
const path = require("path");

const createWindow = () => {
  const win = new BrowserWindow({
    width: 800,
    height: 600,
    webPreferences: {
      contextIsolation: false,
      nodeIntegration: true,
      preload: path.join(__dirname, "preload.js"),
    },
  });

  if (app.isPackaged) {
    win.loadFile("dist/index.html");
  } else {
    win.loadURL("http://localhost:1234");
  }
};

ipcMain.on("ondragstart", (event, filePath) => {
  event.sender.startDrag({
    file: filePath,
    icon: path.join(__dirname, "drag_and_drop.png"),
  });
});

app.on("web-contents-created", (_event, contents) => {
  contents.on("will-navigate", (event) => {
    // https://www.electronjs.org/docs/latest/tutorial/security#13-disable-or-limit-navigation
    event.preventDefault();
  });

  // https://www.electronjs.org/docs/latest/tutorial/security#14-disable-or-limit-creation-of-new-windows
  contents.setWindowOpenHandler(() => ({ action: "deny" }));
});

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") app.quit();
});

app.whenReady().then(() => {
  createWindow();

  app.on("activate", () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});
