/**
 * Provides basic functionality for opt-in observers.
 * 
 * @module
 */

/**
 * Alias for the type of a parametrized observer function.
 * An observer function can be defined with any parameters.
 * 
 * @typeParam T Array type that represents the parameter list
 *              of the observer.
 * @param args Parameters passed to the observer.
 */
export type Observer<T extends any[] = []> = (...args: T) => void;

/**
 * Represents an object or event that can be observed.
 * 
 * @typeParam T Array type that represents the parameter list
 *              of the observer.
 */
export interface Hookable<T extends any[] = []> {
    /**
     * Attaches an observer to the object.
     * This can be called multiple times on the same observer.
     * 
     * @param callback The observer function.
     * @returns A handle that can be used to unhook the observer.
     */
    hook(callback: Observer<T>): ObserverHandle;
}

/**
 * Represents an observer that has been registered with
 * a {@link Hookable}. This interface can be used to unhook
 * the observer when it is no longer needed.
 */
export interface ObserverHandle {
    /**
     * Removes the observer. The observer will not receive
     * notifications anymore.
     */
    unhook(): void;
}

/**
 * A simple hook that can be triggered from the outside.
 * It implements {@link Hookable}, so it can be exposed
 * to allow others to register their observers
 * without allowing them to trigger the hook.
 * 
 * @typeParam T Array type that represents the parameter list
 *              of the observer.
 * 
 * @example
 * ```
 * // Attach a callback to the hook
 * let hook = new Hook();
 * let registration = hook.hook(() => alert('Hello World'));
 * 
 * // Trigger the hook to call all registered callbacks
 * hook.trigger();
 * 
 * // Remove a callback once it is no longer needed
 * registration.unhook();
 * 
 * // Pass arguments to the hook callback
 * let hook = new Hook<[string]>();
 * hook.hook(message => alert(message));
 * hook.trigger('Hello World');
 * ```
 */
export class Hook<T extends any[] = []> implements Hookable<T> {
    private readonly callbacks: Set<Observer<T>>;

    constructor() {
        this.callbacks = new Set();
    }

    hook(callback: Observer<T>): ObserverHandle {
        // Wrap the observer to create a unique object
        // each time it is called, even with the same observer
        let wrappedCallback = (...args: T) => callback(...args);
        let callbacks: Set<Observer<T>> | undefined = this.callbacks;
        callbacks.add(wrappedCallback);
        return {
            unhook: () => {
                if (callbacks) {
                    callbacks.delete(wrappedCallback);
                    callbacks = undefined;
                }
            }
        }
    }
    /**
     * Notifies all observers that are currently registered with
     * the hook.
     * 
     * @param args Arguments that get passed to all observers.
     */
    trigger(...args: T): void {
        this.callbacks.forEach(c => c(...args));
    }
}

/**
 * A {@link Hookable} that never triggers.
 * 
 * @example
 * ```
 * interface HookableVariable {
 *   value: any;
 *   onChange: Hookable;
 * }
 * 
 * let hookableConstant: HookableVariable = {
 *   value: 42,
 *   onChange: NULL_HOOK, // Value never changes
 * }
 * ```
 */
export const NULL_HOOK: Hookable<any[]> = {
    hook: () => NULL_UNHOOK,
}

/**
 * The {@link ObserverHandle} that pairs with {@link NULL_HOOK}.
 */
const NULL_UNHOOK: ObserverHandle = {
    unhook: () => {},
}
