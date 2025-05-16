/**
 * Main process of the Electron application.
 * 
 * Window launching and interactions with the operating
 * system happen here.
 * 
 * @module
 */

import { app, BrowserWindow, shell } from 'electron';
import { is } from '@electron-toolkit/utils';
import path from 'path';
import { setupIpcApi, setupIpcEvents } from './ipcapi';
import { DebuggerManager } from './debugger-manager';

/**
 * Creates the application main window.
 * 
 * @returns Main window of the application.
 */
function setupMainWindow(): BrowserWindow {
    const mainWindow = new BrowserWindow({
        webPreferences: {
            preload: path.resolve(app.getAppPath(), 'out/preload/index.js'),
            spellcheck: false,
        },
        icon: path.resolve(app.getAppPath(), 'assets/favicon/favicon.png'),
        show: false,
    });

    mainWindow.on('ready-to-show', () => mainWindow.maximize());
    mainWindow.on('close', () => app.quit());

    // The window's JS context has access to very powerful API,
    // so it is quite unsafe if a random web page were loaded through it.
    // This will force URLs to be opened in an actual browser instead
    // https://stackoverflow.com/a/67409223/15075450
    mainWindow.webContents.on('will-frame-navigate', e => {
        shell.openExternal(e.url);
        e.preventDefault();
    });
    mainWindow.webContents.setWindowOpenHandler(e => {
        shell.openExternal(e.url);
        return { action: 'deny' };
    });

    // https://github.com/alex8088/quick-start/blob/master/packages/create-electron/playground/vanilla-ts/src/main/index.ts
    // Electron-Vite supports hot reloading (like a Vite dev server)
    // To enable this, we load the page from the dev server
    // instead of the static file if available
    if (is.dev && process.env['ELECTRON_RENDERER_URL']) {
        mainWindow.loadURL(process.env['ELECTRON_RENDERER_URL']);
    } else {
        mainWindow.loadFile('out/renderer/index.html');
    }

    return mainWindow;
}

/**
 * Initializes the application.
 */
function setupApp() {
    const debuggerManager = new DebuggerManager();
    const mainWindow = setupMainWindow();
    setupIpcApi(debuggerManager);
    setupIpcEvents(mainWindow.webContents, debuggerManager);
}

app.whenReady().then(setupApp);
