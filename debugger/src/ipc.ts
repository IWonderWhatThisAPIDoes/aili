/**
 * Common declarations for communication between Electron
 * processes.
 *
 * See [Electron Inter-Process Communication](https://www.electronjs.org/docs/latest/tutorial/ipc)
 * for more information.
 *
 * @module
 */

/**
 * Name of the IPC channel for {@link IpcActionApi.createDebuggerInstance}.
 */
export const START_DEBUGGER: string = 'start-debugger';
/**
 * Name of the IPC channel for {@link IpcActionApi.terminateDebugger}.
 */
export const KILL_DEBUGGER: string = 'kill-debugger';
/**
 * Name of the IPC channel for {@link IpcActionApi.getPathToDebugger}.
 */
export const GET_GDB_PATH: string = 'get-debugger-path';
/**
 * Name of the IPC channel for {@link IpcActionApi.setPathToDebugger}.
 */
export const SET_GDB_PATH: string = 'set-debugger-path';
/**
 * Name of the IPC channel for {@link IpcActionApi.sendInputToDebugger}.
 */
export const GDB_INPUT: string = 'debugger-input';
/**
 * Name of the IPC channel for {@link IpcEventApi.addDebuggerExitHandler}.
 */
export const GDB_EXIT: string = 'debugger-exit';
/**
 * Name of the IPC channel for {@link IpcEventApi.addDebuggerErrorHandler}.
 */
export const GDB_ERROR: string = 'debugger-error';
/**
 * Name of the IPC channel for {@link IpcEventApi.addDebuggerOutputHandler}.
 */
export const GDB_OUTPUT: string = 'debugger-output';
/**
 * Name of the IPC channel for {@link IpcActionApi.getFileContents}.
 */
export const READ_FILE: string = 'read-file';

/**
 * API for communication from renderer to main process.
 *
 * The interface allows static code analysis to ensure
 * all endpoints are implemented.
 */
export interface IpcActionApi {
    /**
     * Launches a new instance of a debugger.
     *
     * @returns PID of the created process.
     * @throws Debugger could not be started.
     */
    createDebuggerInstance(): Promise<number>;
    /**
     * Kills a running debugger process.
     *
     * @param pid PID of the debugger process.
     * @throws `pid` does not correspond to an active
     *         debugger instance or it could not be terminated.
     */
    terminateDebugger(pid: number): Promise<void>;
    /**
     * Queries the currently set path to the debugger executable.
     *
     * @returns Path to the debugger binary.
     */
    getPathToDebugger(): Promise<string>;
    /**
     * Sets the path to the debugger executable.
     *
     * @param path Path to the debugger binary.
     * @throws `path` is not a valid path.
     */
    setPathToDebugger(path: string): void;
    /**
     * Sends data to standard input of a debugger process.
     *
     * @param pid PID of the debugger process.
     * @param input Input to send to the debugger.
     * @throws `pid` does not correspond to an active debugger instance
     *         or it does not accept input.
     */
    sendInputToDebugger(pid: number, input: string): Promise<void>;
    /**
     * Retrieves the contents of a file.
     *
     * @param fileName Name of the file to read.
     */
    getFileContents(fileName: string): Promise<string>;
}

/**
 * API for communication from main to renderer process.
 *
 * The interface allows static code analysis to ensure
 * all endpoints are implemented.
 */
export interface IpcEventApi {
    /**
     * Registers a new handler for the event of debugger process exiting.
     *
     * @param handler Handler that reacts to the event.
     */
    addDebuggerExitHandler(handler: (pid: number, exitCode: number | undefined) => void): void;
    /**
     * Registers a new handler for the event of debugger
     * process reporting an error.
     *
     * @param handler Handler that reacts to the event.
     */
    addDebuggerErrorHandler(handler: (pid: number, error: Error) => void): void;
    /**
     * Registers a new handler for the event of debugger
     * process emiting data on its standard output.
     *
     * @param handler Handler that reacts to the event.
     */
    addDebuggerOutputHandler(handler: (pid: number, output: string) => void): void;
}

/**
 * Name of the global variable in renderer process
 * where IPC API will be exposed.
 */
export const API_KEY: string = 'ipc';

declare global {
    /**
     * Interface for communicating with the main process.
     *
     * This variable is accessible from the renderer process.
     * Import this module to get its declaration for static type analysis.
     */
    const ipc: IpcActionApi & IpcEventApi;
}
