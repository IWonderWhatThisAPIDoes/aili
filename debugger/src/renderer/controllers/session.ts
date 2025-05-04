/**
 * Low-level wrapper for a debug session.
 * 
 * @module
 */

import { Hook, Hookable, Logger, Severity } from 'aili-hooligan';
import { Debugger, DebuggerInputSource, DebuggerStatus } from '../controllers/debugger';
import { GdbMiResultClass } from '../utils/gdbmi-parser';
import { parseGdbMiRecord } from 'aili-jsapi';

/**
 * States of a debug session.
 */
export enum DebugSessionStatus {
    /**
     * No debug session is currently active.
     */
    INACTIVE,
    /**
     * Debug session is active and ready to accept commands.
     */
    READY,
    /**
     * Debug session is active and currently handling a command.
     * 
     * Most operations will fail in this state.
     */
    BUSY,
}

/**
 * How far ahead the debugger can step.
 */
export enum DebugStepRange {
    /**
     * Single step with stepping into function call.
     */
    SINGLE,
    /**
     * Single step, skipping function calls.
     */
    NEXT,
    /**
     * Runs the current function to completion.
     */
    FINISH,
}

/**
 * Converts a debugger status to the corresponding status of a debug session.
 * 
 * @param debuggerStatus Status of the debugger.
 * @returns Corresponding status of a debug session on the same debugger.
 */
export function sessionStatusFromDebuggerStatus(
    debuggerStatus: DebuggerStatus,
    previousStatus: DebugSessionStatus = DebugSessionStatus.INACTIVE,
): DebugSessionStatus {
    switch (debuggerStatus) {
        case DebuggerStatus.EXECUTING:
        case DebuggerStatus.LAUNCHING:
            return DebugSessionStatus.BUSY;
        case DebuggerStatus.STOPPING:
        case DebuggerStatus.FAILED_TO_STOP:
            return previousStatus;
        case DebuggerStatus.PAUSED:
            return DebugSessionStatus.READY;
        default:
            return DebugSessionStatus.INACTIVE;
    }
}

/**
 * Low-level wrapper for a debug session.
 * 
 * The session takes place at debugger level,
 * with status directly copied from debugger.
 */
export class DebuggerSession {
    /**
     * Constructs a new session wrapper for a provided debugger.
     * 
     * @param debuggerContainer Debugger that the session is backed by.
     */
    constructor(debuggerContainer: Debugger) {
        this.debuggerContainer = debuggerContainer;
        this.debuggerContainer.onStatusChanged.hook(status => this.debuggerStatusChanged(status));
        this._onStatusChanged = new Hook();
        this._status = sessionStatusFromDebuggerStatus(debuggerContainer.status);
    }
    /**
     * Starts a new debug session.
     * 
     * @throws A debug session is already active,
     *         Debugger or debuggee could not be launched,
     *         or the session could not be started.
     */
    async start(): Promise<void> {
        this.logger?.log(Severity.DEBUG, 'Starting debug session...');
        await this.withErrorLogging(async () => {
            this.assertStatus(DebugSessionStatus.INACTIVE);
            if (!this.pathToDebuggee) {
                throw new Error('Debuggee is not specified');
            }
            // Start the debugger if necessary
            if (this.debuggerContainer.status === DebuggerStatus.INACTIVE) {
                await this.debuggerContainer.start();
            }
            // TODO: Sanitize path to debuggee
            await this.sendMiCommandAndAssertSuccess(`-file-exec-and-symbols "${this.pathToDebuggee}"`);
            await this.sendMiCommandAndAssertSuccess('-exec-run --start');
        });
    }
    /**
     * Stops an ongoing debug session.
     * 
     * @throws No debug session is active
     *         or the debugger could not end the session.
     */
    async stop(): Promise<void> {
        this.logger?.log(Severity.DEBUG, 'Stopping debug session...');
        await this.withErrorLogging(async () => {
            this.assertStatus(DebugSessionStatus.READY);
            await this.sendMiCommandAndAssertSuccess('-interpreter-exec console kill');
        });
    }
    /**
     * Single-steps the debugger session and updates state.
     * 
     * @param range How far the debug session should advance.
     * 
     * @throws No debug session is active
     *         or the debugger could not make the step.
     */
    async step(range: DebugStepRange = DebugStepRange.SINGLE): Promise<void> {
        await this.withErrorLogging(async () => {
            this.assertStatus(DebugSessionStatus.READY);
            await this.sendMiCommandAndAssertSuccess(STEP_RANGE_TO_GDB_COMMAND[range]);
            await this.debuggerContainer.whenDebuggeeStops();
        });
    }
    /**
     * Retrieves the current state of the session.
     */
    get status(): DebugSessionStatus {
        return this._status;
    }
    /**
     * Triggers when the status of the session updates.
     * 
     * @event
     */
    get onStatusChanged(): Hookable<[DebugSessionStatus]> {
        return this._onStatusChanged;
    }
    /**
     * Path to the executable to be debugged.
     */
    pathToDebuggee: string = '';
    /**
     * Logger to which log messages from the session should be sent.
     */
    logger: Logger | undefined = undefined;
    /**
     * Handles the change of state of the underlying debugger.
     * 
     * @param newStatus New status reported by the debugger.
     */
    private debuggerStatusChanged(newStatus: DebuggerStatus): void {
        const sessionStatus = sessionStatusFromDebuggerStatus(newStatus, this._status);
        if (sessionStatus !== this._status) {
            if (sessionStatus === DebugSessionStatus.INACTIVE) {
                this.logger?.log(Severity.INFO, 'Debug session has ended');
            } else if (sessionStatus === DebugSessionStatus.BUSY && this._status === DebugSessionStatus.INACTIVE) {
                this.logger?.log(Severity.INFO, 'Debug session has started');
            }
            this._status = sessionStatus;
            this._onStatusChanged.trigger(sessionStatus);
        }
    }
    /**
     * Sends a GDB/MI command to the debugger, awaits its response,
     * and asserts that the operation was successful,
     * throwing an exception if it was not.
     * 
     * @param command The GDB/MI command to send to the debugger.
     * 
     * @throws The debugger did not accept the input or it responded
     *         with an error.
     */
    private async sendMiCommandAndAssertSuccess(command: string): Promise<void> {
        let [response, header] = await this.debuggerContainer.sendMiCommand(command, DebuggerInputSource.SESSION);
        // Handle the error class separately, it may provide useful information
        if (header.class === GdbMiResultClass.ERROR) {
            const message = parseGdbMiRecord(response)?.results?.msg ?? '[no details provided]';
            throw new Error(message);
        }
        // All other classes except success are equally invalid
        if (header.class !== GdbMiResultClass.DONE && header.class !== GdbMiResultClass.RUNNING) {
            throw new Error('Unexpected response class from debugger: ' + header.class);
        }
    }
    /**
     * Asserts that the session is in a specified status,
     * throwing an exception if it is not.
     * 
     * @param expectedStatus The status that is required for the operation to proceed.
     * 
     * @throws The session is not in the expected status.
     */
    private assertStatus(expectedStatus: DebugSessionStatus): void {
        if (this._status !== expectedStatus) {
            const expectedName = DebugSessionStatus[expectedStatus];
            const gotName = DebugSessionStatus[this._status];
            throw new Error(`Debug pipeline is in invalid state: ${gotName}, expected ${expectedName}`);
        }
    }
    /**
     * Performs an action and logs all errors that have been thrown.
     * 
     * @typeParam T Return value of the action.
     * @param action The action that should be performed.
     * @returns Return value of `action`.
     */
    private async withErrorLogging<T>(action: () => PromiseLike<T>): Promise<T> {
        try {
            return await action();
        } catch (e) {
            this.logger?.log(Severity.ERROR, String(e));
            throw e;
        }
    }
    private _status: DebugSessionStatus;
    private readonly _onStatusChanged: Hook<[DebugSessionStatus]>;
    private readonly debuggerContainer: Debugger;
}

/**
 * Maps stepping ranges to the GDB/MI commands
 * that advance the debuggee by that range.
 */
const STEP_RANGE_TO_GDB_COMMAND: Record<DebugStepRange, string> = {
    [DebugStepRange.SINGLE]: '-exec-step',
    [DebugStepRange.NEXT]: '-exec-next',
    [DebugStepRange.FINISH]: '-exec-finish',
};
