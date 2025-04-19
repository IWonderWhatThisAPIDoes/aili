/**
 * Container for promises that get resolved manually later on.
 * 
 * @module
 */

/**
 * Container for promises that can be resolved manually.
 * 
 * This container can be also thought of as an observer-based
 * architecture, wrapped in a promise interface.
 * Promises are implicitly created when they are accessed by name
 * and can be resolved manually by calling the dedicated method.
 * 
 * @typeParam T Type of values that the promises resolve to.
 * 
 * @example
 * ```js
 * const container = new PromiseContainer<string>();
 * 
 * // Construct a promise bound to a key
 * container.whenResolves('my-promise')
 *     .then(result => console.log(result));
 * 
 * // The promise can be manually resolved
 * // with the corresponding key
 * container.resolve('my-promise', 'Hello World!');
 * 
 * // Reject all pending promises when the container is dropped
 * // to avoid leaking pending promises
 * container.rejectAllPending(new Error('Promise will never be fulfiled'));
 * ```
 */
export class PromiseContainer<T> {
    constructor() {
        this.promises = {};
    }
    /**
     * Retrieves or constructs a promist that will be resolved
     * when a certain event is reported.
     * 
     * @param promiseName Name of the event that the promise waits for.
     * @returns Result that the promise will be resolved with.
     * @throws Error that the promise will be rejected with.
     */
    whenResolves(promiseName: string): Promise<T> {
        // Return the existing promise if it is already there
        if (this.promises[promiseName]?.promise) {
            return this.promises[promiseName].promise;
        }
        // Construct a new promise
        // Fill in the resolve callbacks by its executor
        const newPromise: PendingPromise<T> = {};
        newPromise.promise = new Promise((resolve, reject) => {
            newPromise.resolve = resolve;
            newPromise.reject = reject;
        });
        this.promises[promiseName] = newPromise;
        return newPromise.promise;
    }
    /**
     * Signals that an event has occurred that may have
     * resolved a promise stored in this container.
     * 
     * @param promiseName Name of the event that occurred.
     * @param result Result that the pending promise related
     *               to the event should resolve with.
     */
    resolve(promiseName: string, result: T): void {
        const executor = this.promises[promiseName];
        delete this.promises[promiseName];
        executor?.resolve?.(result);
    }
    /**
     * Signals that a failure has occurred that may have
     * rejected a promise stored in this container.
     * 
     * @param promiseName Name of the event that failed.
     * @param error Error that the pending promise related
     *              to the event should reject with.
     */
    reject(promiseName: string, error: Error): void {
        const executor = this.promises[promiseName];
        delete this.promises[promiseName];
        executor?.reject?.(error);
    }
    /**
     * Immediately rejects all pending promises in the container.
     * 
     * This is usualy done when the container is about to be dropped
     * so as to avoid leaking unresolved promises.
     * 
     * @param error Error that all pending promises should reject with.
     */
    rejectAllPending(error: Error): void {
        const promises = this.promises;
        this.promises = {};
        for (const promiseName in promises) {
            promises[promiseName].reject?.(error);
        }
    }
    private promises: Record<string, PendingPromise<T>>;
}

/**
 * Data of a promise that has been requested
 * and not yet resolved or rejected.
 * 
 * @typeParam T Type of values that the promises resolve to.
 */
interface PendingPromise<T> {
    /**
     * Promise that represents the event.
     */
    promise?: Promise<T>;
    /**
     * Resolve callback provided by the promise.
     * 
     * @param result Value that the promise should resolve to.
     */
    resolve?(result: T): void;
    /**
     * Reject callback provided by the promise.
     * 
     * @param error Error that the promise should reject with.
     */
    reject?(error: Error): void;
}
