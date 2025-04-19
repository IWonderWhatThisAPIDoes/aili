/**
 * Implementation of the IPC API from the side of the main process.
 * 
 * See [Electron Inter-Process Communication](https://www.electronjs.org/docs/latest/tutorial/ipc)
 * for more information.
 * 
 * @module
 */

import * as ipc from '../ipc';
import { DebuggerManager } from './debugger-manager';
import { ipcMain, WebContents } from 'electron';

/**
 * Sets up {@link ipc.IpcEventApi | IpcEventApi} for a single renderer process.
 * 
 * @param webProcess Handle to the renderer process.
 * @param debuggerManager Debugger manager whose events should be forwarded
 *                        to the renderer process.
 */
export function setupIpcEvents(webProcess: WebContents, debuggerManager: DebuggerManager) {
    debuggerManager.onProcessExit.hook((pid, exitCode) => webProcess.send(ipc.GDB_EXIT, pid, exitCode));
    debuggerManager.onProcessError.hook((pid, error) => webProcess.send(ipc.GDB_ERROR, pid, error));
    debuggerManager.onProcessOutput.hook((pid, output) => webProcess.send(ipc.GDB_OUTPUT, pid, output));
}

/**
 * Sets up {@link ipc.IpcActionApi | IpcActionApi} listeners
 * for renderer processes to call.
 * 
 * @param debuggerManager Debugger manager that should be used to implement
 *                        the action API.
 */
export function setupIpcApi(debuggerManager: DebuggerManager) {
    // Use the type annotation so we can get static analysis in IDE
    const implementation: ipc.IpcActionApi = {
        createDebuggerInstance: () => debuggerManager.spawn(),
        terminateDebugger: pid => debuggerManager.kill(pid),
        setPathToDebugger: path => debuggerManager.pathToDebugger = path,
        getPathToDebugger: async () => debuggerManager.pathToDebugger,
        sendInputToDebugger: (pid, input) => debuggerManager.sendInput(pid, input),
    };

    ipcMain.handle(ipc.START_DEBUGGER, implementation.createDebuggerInstance);
    ipcMain.handle(ipc.KILL_DEBUGGER, (_, h) => implementation.terminateDebugger(h));
    ipcMain.handle(ipc.GET_GDB_PATH, implementation.getPathToDebugger);
    ipcMain.on(ipc.SET_GDB_PATH, (_, path) => implementation.setPathToDebugger(path));
    ipcMain.handle(ipc.GDB_INPUT, (_, pid, input) => implementation.sendInputToDebugger(pid, input));
}
