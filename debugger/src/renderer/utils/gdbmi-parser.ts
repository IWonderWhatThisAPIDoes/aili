/**
 * Parsing of output returned by GDB.
 * 
 * This implementation is in accordance with
 * [GDB Machine Interface](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI.html)
 * as documented.
 * 
 * @module
 */

export { parseGdbMiRecord, GdbMiRecord } from 'aili-jsapi';

/**
 * Types of records returned by GDB.
 * 
 * See [GDB/MI Output Syntax](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Output-Syntax.html)
 * for more information.
 */
export enum GdbMiRecordType {
    /**
     * Synchronous result of an action.
     */
    RESULT,
    /**
     * Asynchronous record related to whether the debuggee is running.
     */
    EXEC,
    /**
     * Asynchronous record related to progression of long operations.
     */
    STATUS,
    /**
     * Asynchronous record related to miscellaneous events.
     */
    NOTIFY,
    /**
     * Output emited by GDB, intended for the user.
     */
    CONSOLE,
    /**
     * Output of the debuggee.
     * 
     * This is not actually used unless the debuggee is remote.
     */
    TARGET,
    /**
     * Log messages emited by GDB.
     */
    LOG,
}

/**
 * Class of a {@link GdbMiRecordType.RESULT} record.
 * 
 * Further specifies the outcome of the operation.
 * 
 * Documented in [GDB/MI Result Records](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Result-Records.html).
 */
export enum GdbMiResultClass {
    /**
     * The operation has completed successfully.
     * 
     * Synonymous with {@link GdbMiResultClass.RUNNING}.
     */
    DONE = 'done',
    /**
     * The operation has completed successfully.
     * 
     * Synonymous with {@link GdbMiResultClass.DONE}.
     */
    RUNNING = 'running',
    /**
     * Successfully connected to a remote target.
     */
    CONNECTED = 'connected',
    /**
     * Operation failed.
     */
    ERROR = 'error',
    /**
     * The debugger has exited.
     */
    EXIT = 'exit',
}

/**
 * Class of a {@link GdbMiRecordType.EXEC} record.
 * 
 * Documented in [GDB/MI Async Records](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Async-Records.html).
 */
export enum GdbMiAsyncExecClass {
    /**
     * Execution of the debuggee has resumed.
     */
    RUNNING = 'running',
    /**
     * Execution of the debuggee has stopped.
     */
    STOPPED = 'stopped',
}

/**
 * Class of a {@link GdbMiRecordType.NOTIFY} record.
 * 
 * Documented in [GDB/MI Async Records](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Async-Records.html).
 * 
 * This enumeration is not exhaustive. The documentation specifies
 * many different notification classes. Only the ones that are relevant
 * to this project are present.
 */
export enum GdbMiAsyncNotifyClass {
    /**
     * A thread group has started.
     * 
     * This means a debuggee has started or been attached to.
     */
    THREAD_GROUP_STARTED = 'thread-group-started',
    /**
     * A thread group has exited.
     * 
     * This means a debuggee has stopped or been detached from.
     */
    THREAD_GROUP_EXITED = 'thread-group-exited',
}

/**
 * The header of a GDB/MI output record,
 * i. e. the part that is not the payload.
 */
export interface GdbMiRecordHeader {
    /**
     * Token that was used with the corresponding command.
     */
    token: string;
    /**
     * Type of the record.
     */
    recordType: GdbMiRecordType;
    /**
     * Class of the record.
     */
    class: GdbMiResultClass | GdbMiAsyncExecClass | string,
}

/**
 * Parses the header of a GDB/MI output record.
 * 
 * @param record The record to parse, as a string.
 * @returns The parsed record, or `undefined` if the syntax is not valid.
 */
export function parseGdbMiRecordHeader(record: string): GdbMiRecordHeader | undefined {
    const matches = /^(\d*)([\^@&*=~+])([a-z\-]*)/.exec(record);
    if (!matches) {
        return undefined;
    }
    return {
        token: matches[1],
        recordType: RECORD_TYPE_FROM_PREFIX[matches[2]],
        class: matches[3],
    };
}

/**
 * Maps record types to the characters that the respective
 * records start with.
 */
const RECORD_TYPE_FROM_PREFIX = {
    '^': GdbMiRecordType.RESULT,
    '*': GdbMiRecordType.EXEC,
    '+': GdbMiRecordType.STATUS,
    '=': GdbMiRecordType.NOTIFY,
    '~': GdbMiRecordType.CONSOLE,
    '@': GdbMiRecordType.TARGET,
    '&': GdbMiRecordType.LOG,
}
