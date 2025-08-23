/**
 * Controller for debugger process.
 *
 * @module
 */

import { Hook, Hookable, Logger, Severity, TopicLogger } from 'aili-hooligan';
import {
    GdbMiAsyncExecClass,
    GdbMiAsyncNotifyClass,
    GdbMiRecordHeader,
    GdbMiRecordType,
    parseGdbMiRecord,
    parseGdbMiRecordHeader,
} from '../utils/gdbmi-parser';
import { PromiseContainer } from '../utils/promise-container';
import '../../ipc';

export const LOG_TOPIC_FROM_DEBUGGER: string = 'from-debugger';
export const LOG_TOPIC_TO_DEBUGGER: string = 'to-debugger';
export const LOG_TOPIC_DEBUGGER_META: string = 'debugger-meta';
export const LOG_TOPIC_FROM_SESSION: string = 'from-session';
export const LOG_TOPIC_FROM_STATE: string = 'from-state';
export const LOG_TOPIC_FROM_USER: string = 'from-user';

/**
 * State of an external debugger.
 */
export enum DebuggerStatus {
    /**
     * The debugger is not running.
     */
    INACTIVE,
    /**
     * The debugger is being launched.
     */
    STARTING,
    /**
     * The debugger is running and no debug session is in progress.
     */
    IDLE,
    /**
     * The debugger is currently executing the debuggee.
     */
    EXECUTING,
    /**
     * The debugger is running and the debuggee is paused.
     */
    PAUSED,
    /**
     * The debugger is launching the debuggee.
     */
    LAUNCHING,
    /**
     * The debugger is being shut down.
     */
    STOPPING,
    /**
     * The debugger is active and an attempt to kill it failed.
     */
    FAILED_TO_STOP,
}

/**
 * Shorthand for checking whether a debugger state corresponds
 * to the debugger running, regardless of the state of the debuggee.
 *
 * @param status Debugger status to check.
 * @returns True if `status` corresponds to the debugger running.
 */
export function isStatusRunning(status: DebuggerStatus): boolean {
    return (
        status === DebuggerStatus.EXECUTING ||
        status === DebuggerStatus.IDLE ||
        status === DebuggerStatus.PAUSED ||
        status === DebuggerStatus.LAUNCHING
    );
}

/**
 * Sources that can send input to the debugger.
 *
 * Used for logging.
 */
export enum DebuggerInputSource {
    /**
     * Unknown or miscellaneous.
     */
    UNKNOWN,
    /**
     * Input entered manually by the user.
     */
    USER,
    /**
     * Input sent by a session manager
     * to control execution of the debuggee.
     */
    SESSION,
    /**
     * Input sent by a state model
     * to examine the state of the debuggee.
     */
    STATE,
}

/**
 * Describes the location in the source code where
 * the debuggee is currently executing.
 */
export interface SourceLocation {
    /**
     * Name of the source file that the current instruction is mapped to.
     */
    fileName?: string;
    /**
     * Full path to the source file that the current instruction is mapped to.
     */
    filePath?: string;
    /**
     * Number of the line in the source code that the current instruction is mapped to.
     */
    lineNumber?: number;
    /**
     * Name of the function being executed.
     */
    functionName?: string;
}

/**
 * Controls an external debugger instance.
 */
export class Debugger {
    constructor() {
        this._onStatusChanged = new Hook();
        this.ioPromises = new PromiseContainer();
        ipc.addDebuggerExitHandler((pid, exitCode) => this.processExited(pid, exitCode));
        ipc.addDebuggerErrorHandler((pid, error) => {
            if (pid == this._pid) {
                this.fromDebuggerLogger?.log(Severity.ERROR, String(error));
            }
        });
        ipc.addDebuggerOutputHandler((pid, output) => this.processEmitedOutput(pid, output));
    }
    /**
     * Launches the debugger asynchronously.
     *
     * @throws The debugger is already running or it could not be started.
     */
    async start(): Promise<void> {
        if (this._status != DebuggerStatus.INACTIVE) {
            this.metaLogger?.log(Severity.ERROR, 'Debugger has already been started');
            throw new Error('Debugger has already been started');
        }
        this.updateStatus(DebuggerStatus.STARTING);
        this.metaLogger?.log(Severity.INFO, 'Sending start request to main process...');
        try {
            this._pid = await ipc.createDebuggerInstance();
        } catch (e) {
            this.updateStatus(DebuggerStatus.INACTIVE);
            this.metaLogger?.log(Severity.ERROR, String(e));
            throw e;
        }
        this.metaLogger?.log(
            Severity.INFO,
            `Debugger has started successfully (PID: ${this._pid})`,
        );
        this.nextToken = 0;
        this.updateStatus(DebuggerStatus.IDLE);
    }
    /**
     * Terminates the debugger asynchronously.
     *
     * @throws The debugger is not running or it could not be stopped.
     */
    async stop(): Promise<void> {
        if (!isStatusRunning(this._status) || this._pid === undefined) {
            this.metaLogger?.log(Severity.ERROR, 'Debugger is not active');
            throw new Error('Debugger is not active');
        }
        this.updateStatus(DebuggerStatus.STOPPING);
        this.metaLogger?.log(Severity.INFO, 'Sending stop request to main process...');
        try {
            await ipc.terminateDebugger(this._pid);
        } catch (e) {
            this.updateStatus(DebuggerStatus.FAILED_TO_STOP);
            throw e;
        }
    }
    /**
     * Forgets the current debugger instance, but leaves it running.
     */
    detach(): void {
        this._pid = undefined;
        this._sourceLocation = undefined;
        this.updateStatus(DebuggerStatus.INACTIVE);
    }
    /**
     * Sends a raw input line to the debugger.
     *
     * @param input The input to send to the debugger.
     * @param source Type of the source of the command.
     *
     * @throws The debugger is not running or it could not accept input.
     */
    async sendInputLine(
        input: string,
        source: DebuggerInputSource = DebuggerInputSource.UNKNOWN,
    ): Promise<void> {
        this.toDebuggerLoggers?.[source]?.log(Severity.DEBUG, `${input}`);
        if (!isStatusRunning(this._status) || this._pid === undefined) {
            this.metaLogger?.log(Severity.ERROR, 'Debugger is not running');
            throw new Error('Cannot send input to debugger when it is not running');
        }
        try {
            await ipc.sendInputToDebugger(this._pid, input + '\n');
        } catch (e) {
            this.metaLogger?.log(Severity.ERROR, String(e));
            throw e;
        }
    }
    /**
     * Sends a GDB/MI command to the debugger
     * and awaits the response asynchronously.
     *
     * @param input The input to send to the debugger.
     * @param source Type of the source of the command.
     * @returns Output returned by GDB, as both raw string and parsed header.
     *
     * @throws The debugger is not running or it could not accept input.
     */
    async sendMiCommand(
        input: string,
        source: DebuggerInputSource = DebuggerInputSource.UNKNOWN,
    ): Promise<[string, GdbMiRecordHeader]> {
        // TODO: Make sure it is an MI input line,
        // otherwise we would be waiting for the response indefinitely
        const token = String(this.nextToken++);
        await this.sendInputLine(token + input, source);
        return await this.ioPromises.whenResolves(token);
    }
    /**
     * Creates a promise that resolves when the debuggee stops.
     *
     * @throws The debugger is not running
     */
    async whenDebuggeeStops(): Promise<void> {
        if (
            this._status !== DebuggerStatus.EXECUTING &&
            this._status !== DebuggerStatus.PAUSED &&
            this._status !== DebuggerStatus.LAUNCHING
        ) {
            throw new Error('Cannot wait for debuggee because it is not currently executing');
        } else if (this._status === DebuggerStatus.PAUSED) {
            // If the debuggee is currently paused, resolve immediately
            return Promise.resolve();
        } else {
            await this.ioPromises.whenResolves(STOP_PROMISE_TOKEN);
        }
    }
    /**
     * Status of the debugger.
     */
    get status(): DebuggerStatus {
        return this._status;
    }
    /**
     * PID of the debugger instance, if it is running.
     */
    get pid(): number | undefined {
        return this._pid;
    }
    /**
     * The location in the source code where the debuggee has stopped.
     */
    get sourceLocation(): SourceLocation | undefined {
        return this._sourceLocation;
    }
    /**
     * Triggers when {@link status} changes.
     *
     * @event
     */
    get onStatusChanged(): Hookable<
        [DebuggerStatus, number | undefined, SourceLocation | undefined]
    > {
        return this._onStatusChanged;
    }
    /**
     * Sets the logger to which logs will be sent.
     */
    set logger(logger: TopicLogger | undefined) {
        if (logger) {
            this.fromDebuggerLogger = logger.createTopic(LOG_TOPIC_FROM_DEBUGGER);
            const toDebuggerMainLogger = logger.createTopic(LOG_TOPIC_TO_DEBUGGER);
            this.toDebuggerLoggers = {
                [DebuggerInputSource.UNKNOWN]: toDebuggerMainLogger,
                [DebuggerInputSource.SESSION]:
                    toDebuggerMainLogger.createTopic(LOG_TOPIC_FROM_SESSION),
                [DebuggerInputSource.STATE]: toDebuggerMainLogger.createTopic(LOG_TOPIC_FROM_STATE),
                [DebuggerInputSource.USER]: toDebuggerMainLogger.createTopic(LOG_TOPIC_FROM_USER),
            };
            this.metaLogger = logger.createTopic(LOG_TOPIC_DEBUGGER_META);
        } else {
            this.fromDebuggerLogger = undefined;
            this.toDebuggerLoggers = undefined;
            this.metaLogger = undefined;
        }
    }
    /**
     * Sets the path to the debugger.
     */
    static set pathToDebugger(path: string) {
        ipc.setPathToDebugger(path);
        this._pathToDebugger = path;
    }
    /**
     * Path to the debugger.
     */
    static get pathToDebugger(): string | undefined {
        return this._pathToDebugger;
    }
    /**
     * Queues a callback to be called when {@link pathToDebugger}
     * becomes available.
     *
     * @param callback Callback to call when {@link pathToDebugger}
     *                 becomes known. It receives the path as an argument.
     */
    static onPathToDebuggerAvailable(callback: (path: string) => void): void {
        if (this._onPathToDebuggerAvailable) {
            this._onPathToDebuggerAvailable.hook(callback);
        } else if (this._pathToDebugger) {
            callback(this._pathToDebugger);
        } else {
            // Unreachable - we initialize the path as we remove the hook
            throw new Error(
                'Debugger path initialization hook has been removed, but path is not available',
            );
        }
    }
    private updateStatus(status: DebuggerStatus) {
        this._status = status;
        this._onStatusChanged.trigger(this._status, this._pid, this._sourceLocation);
    }
    private processExited(pid: number, exitCode: number | undefined): void {
        if (pid == this._pid) {
            if (exitCode === undefined) {
                this.metaLogger?.log(Severity.INFO, 'Debugger has been terminated');
            } else {
                this.metaLogger?.log(Severity.INFO, 'Debugger has exited with code ' + exitCode);
            }
            this._pid = undefined;
            this._sourceLocation = undefined;
            this.ioPromises.rejectAllPending(new Error('Debugger has exited'));
            this.updateStatus(DebuggerStatus.INACTIVE);
        }
    }
    private processEmitedOutput(pid: number, output: string): void {
        if (pid == this._pid) {
            for (const line of output.split('\n').map(l => l.trim())) {
                if (!line) {
                    continue;
                }
                this.fromDebuggerLogger?.log(Severity.DEBUG, line);
                if (line === '(gdb)') {
                    continue;
                }
                const parseResult = parseGdbMiRecordHeader(line);
                if (parseResult === undefined) {
                    this.metaLogger?.log(Severity.WARNING, 'Debugger returned invalid output');
                    continue;
                }
                if (parseResult.recordType === GdbMiRecordType.RESULT && parseResult.token) {
                    this.ioPromises.resolve(parseResult.token, [line, parseResult]);
                } else if (parseResult.recordType === GdbMiRecordType.EXEC) {
                    if (parseResult.class === GdbMiAsyncExecClass.RUNNING) {
                        this.updateStatus(DebuggerStatus.EXECUTING);
                    } else if (parseResult.class === GdbMiAsyncExecClass.STOPPED) {
                        const frameInfo = parseGdbMiRecord(line)?.results?.frame;
                        this._sourceLocation =
                            frameInfo === undefined
                                ? undefined
                                : {
                                      functionName: frameInfo?.func,
                                      lineNumber: Number.parseInt(frameInfo?.line),
                                      fileName: frameInfo?.file,
                                      filePath: frameInfo?.fullname,
                                  };
                        if (this._status === DebuggerStatus.EXECUTING) {
                            this.updateStatus(DebuggerStatus.PAUSED);
                        }
                        this.ioPromises.resolve(STOP_PROMISE_TOKEN, [line, parseResult]);
                    } else {
                        this.metaLogger?.log(
                            Severity.WARNING,
                            `Did not recognize class of an async exec record: ${parseResult.class}`,
                        );
                    }
                } else if (parseResult.recordType === GdbMiRecordType.NOTIFY) {
                    if (parseResult.class === GdbMiAsyncNotifyClass.THREAD_GROUP_STARTED) {
                        if (this._status !== DebuggerStatus.IDLE) {
                            this.metaLogger?.log(
                                Severity.WARNING,
                                'Received thread group start notification when debugger is not idle',
                            );
                        } else {
                            this.updateStatus(DebuggerStatus.LAUNCHING);
                        }
                    } else if (parseResult.class === GdbMiAsyncNotifyClass.THREAD_GROUP_EXITED) {
                        if (
                            this._status !== DebuggerStatus.PAUSED &&
                            this._status !== DebuggerStatus.EXECUTING &&
                            this._status !== DebuggerStatus.LAUNCHING
                        ) {
                            this.metaLogger?.log(
                                Severity.WARNING,
                                'Received thread group exit notification outside of a debug session',
                            );
                        }
                        this._sourceLocation = undefined;
                        this.updateStatus(DebuggerStatus.IDLE);
                    }
                }
            }
        }
    }
    private _status = DebuggerStatus.INACTIVE;
    private _pid: number | undefined = undefined;
    private _sourceLocation: SourceLocation | undefined = undefined;
    private _onStatusChanged: Hook<
        [DebuggerStatus, number | undefined, SourceLocation | undefined]
    >;
    private ioPromises: PromiseContainer<[string, GdbMiRecordHeader]>;
    private nextToken: number = 0;
    private fromDebuggerLogger: Logger | undefined;
    private toDebuggerLoggers: Record<DebuggerInputSource, Logger> | undefined;
    private metaLogger: Logger | undefined;
    private static _onPathToDebuggerAvailable? = new Hook<[string]>();
    private static _pathToDebugger: string | undefined = (() => {
        ipc.getPathToDebugger().then(path => {
            this._pathToDebugger = path;
            this._onPathToDebuggerAvailable?.trigger(path);
            delete this._onPathToDebuggerAvailable;
        });
        return undefined;
    })();
}

/**
 * Sentinel token for identifying a stop event
 * in {@link Debugger.ioPromises}.
 *
 * No other promises in the container are identified by this key.
 */
const STOP_PROMISE_TOKEN: string = '*';
