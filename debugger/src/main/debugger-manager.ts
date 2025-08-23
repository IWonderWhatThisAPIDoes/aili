/**
 * Communication with debugger instances in external processes.
 *
 * @module
 */

import { Hook } from 'aili-hooligan';
import { ChildProcess, spawn } from 'child_process';

/**
 * Container for live instances of debugger.
 */
export class DebuggerManager {
    constructor() {
        this.instances = new Map();
        this.onProcessExit = new Hook();
        this.onProcessError = new Hook();
        this.onProcessOutput = new Hook();
    }
    /**
     * Launches a new debugger instance.
     *
     * @returns PID of the new debugger instance.
     * @throws The debugger could not be launched.
     */
    async spawn(): Promise<number> {
        const childProcess = new DebuggerProcess(this.pathToDebugger);
        const pid = await childProcess.start();
        this.instances.set(pid, childProcess);
        childProcess.onExit.hook(exitCode => {
            // Forget the process once it has exited
            this.instances.delete(pid);
            this.onProcessExit.trigger(pid, exitCode);
        });
        childProcess.onError.hook(error => this.onProcessError.trigger(pid, error));
        childProcess.onOutput.hook(output => this.onProcessOutput.trigger(pid, output));
        return pid;
    }
    /**
     * Terminates a running debugger instance.
     *
     * @param pid PID of the debugger instance to kill.
     *            Must have been acquired by previous call
     *            to {@link DebuggerManager.spawn}.
     * @throws `pid` does not correspond to an active
     *         debugger instance or it could not be terminated.
     */
    kill(pid: number): Promise<void> {
        const childProcess = this.instances.get(pid);
        if (!childProcess) {
            throw new Error('Invalid child process handle');
        }
        return childProcess.kill();
    }
    /**
     * Sends data to standard input of a debugger process.
     *
     * @param pid PID of the debugger process.
     *            Must have been acquired by previous call
     *            to {@link DebuggerManager.spawn}.
     * @param input Input to send to the debugger.
     * @throws `pid` does not correspond to an active debugger instance
     *         or it does not accept input.
     */
    sendInput(pid: number, input: string): Promise<void> {
        const childProcess = this.instances.get(pid);
        if (!childProcess) {
            throw new Error('Invalid child process handle');
        }
        return childProcess.writeInput(input);
    }
    /**
     * Path to the debugger executable binary.
     *
     * Used by {@link DebuggerManager.spawn}.
     */
    pathToDebugger: string = 'gdb';
    /**
     * Triggers when a debugger instance exits.
     *
     * PID of the process and its exit code (if available)
     * are forwarded to observers.
     *
     * @event
     */
    onProcessExit: Hook<[number, number | undefined]>;
    /**
     * Triggers when a debugger instance reports an error.
     *
     * PID of the process and the reported error
     * are forwarded to observers.
     *
     * @event
     */
    onProcessError: Hook<[number, Error]>;
    /**
     * Triggers when a debugger instance emits data
     * to its standard output.
     *
     * PID of the process and the data payload
     * are forwarded to observers.
     *
     * @event
     */
    onProcessOutput: Hook<[number, string]>;
    /**
     * Active instances of the debugger, mapped by their PID.
     */
    private instances: Map<number, DebuggerProcess>;
}

/**
 * Wraps a single debugger instance.
 */
class DebuggerProcess {
    /**
     * Launches a new debugger instance.
     *
     * The constructor is synchronous. To get a {@link Promise}
     * that resolves (or fails) when the debugger is initialized,
     * call {@link DebuggerProcess.start} immediately after construction.
     *
     * @param pathToDebugger Path to the debugger executable binary.
     */
    constructor(pathToDebugger: string) {
        this.onExit = new Hook();
        this.onError = new Hook();
        this.onOutput = new Hook();
        this.childProcess = spawn(pathToDebugger, ['--interp', 'mi']);
        this.childProcess.on('exit', exitCode => this.onExit.trigger(exitCode ?? undefined));
        this.childProcess.on('error', error => this.onError.trigger(error));
        // Get data as ASCII-encoded strings
        // https://nodejs.org/api/stream.html#readablesetencodingencoding
        this.childProcess.stdout?.setEncoding('ascii');
        this.childProcess.stdout?.on('data', output => this.onOutput.trigger(output));
    }
    /**
     * Awaits the initialization of the debugger instance.
     *
     * @returns Promise that resolves when the debugger process is initialized.
     *          If successful, resolves with the PID of the debugger process.
     * @throws The debugger could not be launched.
     */
    start(): Promise<number> {
        // The child process is documented to fire exactly one of
        // 'spawn' (if process spawns successfully) or 'error' (if it does not):
        // https://nodejs.org/api/child_process.html#event-spawn
        //
        // We listen for either of the two events and resolve the promise
        // accordingly
        return new Promise((resolve, reject) => {
            const onSpawn = () => {
                // Remove the other listener, we only want to catch one of them
                this.childProcess.off('error', onError);
                if (this.childProcess.pid != undefined) {
                    resolve(this.childProcess.pid);
                } else {
                    // This should not happen, kill the process and report failure
                    this.childProcess.kill();
                    reject(
                        new Error(
                            'Child process reports that it has spawned successfully, but it does not have a PID',
                        ),
                    );
                }
            };
            const onError = (err: Error) => {
                // Remove the other listener, we only want to catch one of them
                this.childProcess.off('spawn', onSpawn);
                reject(err);
            };
            this.childProcess.once('spawn', onSpawn);
            this.childProcess.once('error', onError);
        });
    }
    /**
     * Terminates the wrapped debugger instance.
     *
     * @throws The debugger is already being terminated or it could not be killed.
     */
    kill(): Promise<void> {
        return new Promise((resolve, reject) => {
            if (this.killed) {
                reject(new Error('Child process is already being terminated'));
            }
            if (!this.childProcess.kill()) {
                reject(new Error('Child process could not be terminated'));
            }
            this.killed = true;
            this.childProcess.on('exit', resolve);
        });
    }
    /**
     * Sends data to the standard input of the debugger.
     *
     * @param input Data to send to the debugger.
     * @throws The debugger is not active or it did not accept input.
     */
    async writeInput(input: string): Promise<void> {
        const stdin = this.childProcess.stdin;
        if (!stdin) {
            throw new Error('Child process does not expose its stdin');
        }
        const isDraining = !stdin.write(input, 'ascii');
        // Stream may request that we wait for its buffer to be emptied
        // This should not happen with local OS pipes, but better be safe
        // https://nodejs.org/api/stream.html#writablewritechunk-encoding-callback
        if (isDraining) {
            await new Promise(resolve => stdin.once('drain', resolve));
        }
    }
    /**
     * Triggers when the debugger exits.
     *
     * Exit code of the process is forwarded to observers
     * if it is available.
     *
     * @event
     */
    onExit: Hook<[number | undefined]>;
    /**
     * Triggers when the debugger reports an error.
     *
     * The error object is forwarded to observers.
     *
     * @event
     */
    onError: Hook<[Error]>;
    /**
     * Triggers when the debugger emits data to its standard output.
     *
     * The data payload is forwarded to observers.
     *
     * @event
     */
    onOutput: Hook<[string]>;
    private childProcess: ChildProcess;
    private killed: boolean = false;
}
