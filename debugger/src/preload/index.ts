/**
 * Preload script of the renderer process.
 * 
 * This script runs before the renderer process is launched.
 * IPC API are declared here.
 * 
 * See [Electron Inter-Process Communication](https://www.electronjs.org/docs/latest/tutorial/ipc)
 * for more information.
 * 
 * @module
 */

import { contextBridge, ipcRenderer } from 'electron';
import * as ipc from '../ipc';

/**
 * Implementation of the IPC API for use by the renderer process.
 */
const api: ipc.IpcActionApi & ipc.IpcEventApi = {
    // We use the two-way communication pattern here
    // (https://www.electronjs.org/docs/latest/tutorial/ipc#pattern-2-renderer-to-main-two-way)
    // The matching handler should be registered with `handle`
    createDebuggerInstance: () => ipcRenderer.invoke(ipc.START_DEBUGGER),
    terminateDebugger: h => ipcRenderer.invoke(ipc.KILL_DEBUGGER, h),
    getPathToDebugger: () => ipcRenderer.invoke(ipc.GET_GDB_PATH),
    setPathToDebugger(path: string) {
        // TODO: Sanitize
        ipcRenderer.send(ipc.SET_GDB_PATH, path);
    },
    sendInputToDebugger(pid: number, input: string) {
        // TODO: Sanitize
        return ipcRenderer.invoke(ipc.GDB_INPUT, pid, input);
    },
    addDebuggerExitHandler(handler: (pid: number, exitCode: number | undefined) => void) {
        ipcRenderer.on(ipc.GDB_EXIT, (_, pid, exitCode) => handler(pid, exitCode));
    },
    addDebuggerErrorHandler(handler: (pid: number, error: Error) => void) {
        ipcRenderer.on(ipc.GDB_ERROR, (_, pid, error) => handler(pid, error));
    },
    addDebuggerOutputHandler(handler: (pid: number, output: string) => void) {
        ipcRenderer.on(ipc.GDB_OUTPUT, (_, pid, output) => handler(pid, output));
    }
};

contextBridge.exposeInMainWorld(ipc.API_KEY, api);
