/**
 * Controller for a wrapped debugger session.
 * 
 * @module
 */

import { GdbMiSession, GdbStateGraph, LengthHintSheet } from 'aili-jsapi';
import { Debugger, DebuggerInputSource, isStatusRunning, SourceLocation } from './debugger';
import { DebuggerSession, DebugSessionStatus, DebugStepRange } from './session';
import { Hook, Hookable, Logger } from 'aili-hooligan';

/**
 * Wraps a debugger session, including rendering.
 * 
 * Unlike {@link DebuggerSession}, this wrapper reflects
 * the status of the debugger as well as rendering
 * of the outputs of the debugger.
 * 
 * Whenever a public action is executing, this switches
 * to {@link DebugSessionStatus.BUSY} to prevent interaction.
 * This is not infallible (one could, for example, create
 * multiple instances of this controller over the same debugger
 * instance), but it makes it easier to avoid misuse.
 */
export class DebugSessionManager {
    /**
     * Constructs a new session wrapper for a provided debugger.
     * 
     * @param debuggerContainer Debugger that the session is backed by.
     */
    constructor(debuggerContainer: Debugger) {
        this.debug = debuggerContainer;
        this.session = new DebuggerSession(this.debug);
        this._onStateGraphUpdate = new Hook();
        this._onStatusChanged = new Hook();
        this.debugMi = {
            async sendMiCommand(command: string): Promise<string> {
                return (await debuggerContainer.sendMiCommand(command, DebuggerInputSource.STATE))[0];
            }
        }
        this._status = this.session.status;
        this.session.onStatusChanged.hook(newStatus => {
            // If status is forced to be Busy by this controller,
            // do not touch it
            if (!this.statusBusyOverride) {
                this.updateStatus(newStatus);
            }
        });
    }
    /**
     * Starts a new debug session.
     * 
     * @throws A debug session is already active,
     *         Debugger or debuggee could not be launched,
     *         or the session could not be started.
     */
    async start(): Promise<void> {
        this.assertStatus(DebugSessionStatus.INACTIVE);
        await this.transaction(async () => {
            await this.session.start();
            this.hintsForThisSession = this.hintSheet;
            this.stateGraph = await GdbStateGraph.fromSession(this.debugMi, this.hintsForThisSession);
            this._onStateGraphUpdate.trigger(this.stateGraph);
        });
    }
    /**
     * Stops an ongoing debug session.
     * 
     * @throws No debug session is active
     *         or the debugger could not end the session.
     */
    async stop(): Promise<void> {
        this.assertStatus(DebugSessionStatus.READY);
        await this.transaction(async () => {
            await this.session.stop();
            await this.sessionEnded();
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
        this.assertStatus(DebugSessionStatus.READY);
        await this.transaction(async () => {
            await this.session.step(range);
            // The debuggee may have exited normally,
            // so the debug session would not be active anymore
            if (this.session.status === DebugSessionStatus.INACTIVE) {
                await this.sessionEnded();
            } else if (this.stateGraph) {
                await this.stateGraph.update(this.debugMi, this.hintsForThisSession);
                this._onStateGraphUpdate.trigger(this.stateGraph);
            }
        });
    }
    /**
     * Sets the logger to which log messages from the session should be sent.
     */
    set logger(logger: Logger) {
        this.session.logger = logger;
    }
    /**
     * Retrieves the current path to the debuggee.
     */
    get pathToDebuggee(): string {
        return this.session.pathToDebuggee;
    }
    /**
     * Updates the path to the debuggee.
     */
    set pathToDebuggee(path: string) {
        this.session.pathToDebuggee = path;
    }
    /**
     * Retrieves the current state of the session.
     */
    get status(): DebugSessionStatus {
        return this._status;
    }
    /**
     * The location in the source code where the debuggee has stopped.
     */
    get sourceLocation(): SourceLocation | undefined {
        return this.debug.sourceLocation;
    }
    /**
     * Triggers when the state graph that represents the session updates.
     * 
     * @event
     */
    get onStateGraphUpdate(): Hookable<[GdbStateGraph]> {
        return this._onStateGraphUpdate;
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
     * Changes the session status and triggers the update hook
     * if necessary.
     * 
     * @param newStatus New stateus of the session.
     */
    private updateStatus(newStatus: DebugSessionStatus): void {
        if (this._status !== newStatus) {
            this._status = newStatus;
            this._onStatusChanged.trigger(newStatus);
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
     * Performs an action in a transaction by switching
     * to {@link DebugSessionStatus.BUSY}, and then back
     * when the action is completed.
     * 
     * @typeParam T Return value of the action.
     * @param action The action that should be performed.
     * @returns Return value of `action`.
     */
    private async transaction<T>(action: () => PromiseLike<T>): Promise<T> {
        this.statusBusyOverride = true;
        this.updateStatus(DebugSessionStatus.BUSY);
        try {
            return await action();
        } finally {
            // No matter what, clear the busy flag and switch
            // to whatever status the debugger is reporting
            this.statusBusyOverride = false;
            this.updateStatus(this.session.status);
        }
    }
    /**
     * Cleans up after a session has ended.
     */
    private async sessionEnded(): Promise<void> {
        // If the debugger is still running,
        // clear the state that this session has created in it
        if (isStatusRunning(this.debug.status)) {
            await this.stateGraph?.cleanUp(this.debugMi);
        }
        this.stateGraph = undefined;
    }
    /**
     * Stylesheet that provides hints to help deduce the length of allocated arrays.
     */
    hintSheet: LengthHintSheet = LengthHintSheet.empty();
    private hintsForThisSession: LengthHintSheet;
    private _status: DebugSessionStatus;
    private statusBusyOverride: boolean = false;
    private readonly _onStateGraphUpdate: Hook<[GdbStateGraph]>;
    private readonly _onStatusChanged: Hook<[DebugSessionStatus]>;
    private readonly debug: Debugger;
    private readonly debugMi: GdbMiSession;
    private readonly session: DebuggerSession;
    private stateGraph: GdbStateGraph | undefined = undefined;
}

