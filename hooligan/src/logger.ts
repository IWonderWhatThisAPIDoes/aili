/**
 * Logging utilities.
 *
 * @module
 */

import { Hook, Hookable } from './hook';

/**
 * Severity classes of log messages.
 */
export enum Severity {
    /**
     * This message indicates an error.
     */
    ERROR,
    /**
     * This message indicates an unexpected situation
     * that is likely symptomatic of a bug.
     */
    WARNING,
    /**
     * This message indicates a noteworthy situation.
     */
    INFO,
    /**
     * This message contains information that could help diagnose a problem.
     */
    DEBUG,
}

/**
 * Object that can accept log messages.
 */
export interface Logger {
    /**
     * Emits a log message.
     *
     * @param severity Severity class representing the nature of the message.
     * @param summary Short (one-line) description of the message.
     * @param details If necessary, a longer description that contains additional details.
     */
    log(severity: Severity, summary: string, details?: string | undefined): void;
}

/**
 * A {@link Logger} that can collects messages from multiple different sources
 * and keeps track of the source of each message.
 */
export interface TopicLogger extends Logger {
    /**
     * Registers a new topic - a named source of log messages.
     *
     * @param topicName Name of the topic. Should be unique (otherwise the user cannot tell them apart).
     * @return New {@link Logger} for the new topic. Messages logged into this
     *         will be associated with the topic.
     *
     * @example
     * ```
     * let masterLogger = // ...
     * let fooLogger = masterLogger.createTopic('messages from foo');
     * foo.useLogger(fooLogger);
     * ```
     */
    createTopic(topicName: string): TopicLogger;
}

/**
 * A {@link TopicLogger} that forwards all messages to hook observers.
 */
export class HookableLogger implements TopicLogger {
    constructor() {
        this._onLog = new Hook();
    }
    log(severity: Severity, summary: string, details?: string | undefined): void {
        this._onLog.trigger(this.topicPath, severity, summary, details);
    }
    createTopic(topicName: string): TopicLogger {
        const newLogger = new HookableLogger();
        newLogger._onLog = this._onLog;
        newLogger.topicPath = [...this.topicPath, topicName];
        return newLogger;
    }
    /**
     * Triggers whenever a message is logged.
     *
     * Observers will receive the arguments passed to {@link log},
     * plus the topic path (first argument), starting from the most
     * general topic.
     *
     * @event
     */
    get onLog(): Hookable<[readonly string[], Severity, string, string | undefined]> {
        return this._onLog;
    }
    private _onLog: Hook<[readonly string[], Severity, string, string | undefined]>;
    private topicPath: readonly string[] = [];
}
