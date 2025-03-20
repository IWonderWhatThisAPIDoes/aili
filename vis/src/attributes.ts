/**
 * Attribute containers with change detection.
 * 
 * @module
 */

import { Hook, Hookable, ObserverHandle } from './hook';

/**
 * Read-only view over {@link AttributeMap} that does not permit
 * modifications. Registering observers is still allowed.
 */
export interface ReadonlyAttributeMap {
    /**
     * Attributes of the element.
     */
    readonly [key: string]: ReadonlyAttributeEntry;
}

/**
 * Read-only view over {@link AttributeEntry} that does not permit
 * modifications. Registering observers is still allowed.
 */
export interface ReadonlyAttributeEntry {
    /**
     * Observer hook that is triggered when the value of the attribute
     * changes. The observers receive the new and previous value
     * as arguments.
     * 
     * The hook is triggered after the value has been updated.
     * 
     * @event
     */
    readonly onChange: Hookable<[string | undefined, string | undefined]>;
    /**
     * Current value of the attribute.
     */
    readonly value: string | undefined;
}

/**
 * Container that holds the attributes of a visualization primitive.
 */
export class AttributeMap implements ReadonlyAttributeMap {
    constructor() {
        // Wrap this in a proxy so we can add attributes as they are called
        return new Proxy(this as { [key: string]: AttributeEntry }, {
            get(self, key: string) {
                return self[key] ??= new AttributeEntry();
            }
        });
    }
    /**
     * Attributes of the element.
     */
    readonly [key: string]: AttributeEntry;
}

/**
 * Single visualization primitive attribute.
 */
export class AttributeEntry implements ReadonlyAttributeEntry {
    constructor() {
        this._onChange = new Hook();
    }
    /**
     * Get current value of the attribute.
     */
    get value(): string | undefined {
        return this._value;
    }
    /**
     * Update value of the attribute.
     * Observers will be triggered if it is different
     * from the current value.
     */
    set value(value: string | undefined) {
        const oldValue = this._value;
        if (value !== oldValue) {
            this._value = value;
            this._onChange.trigger(value, oldValue);
        }
    }
    get onChange(): Hookable<[string | undefined, string | undefined]> {
        return this._onChange;
    }
    private _value: string | undefined = undefined;
    private readonly _onChange: Hook<[string | undefined, string | undefined]>;
}

/**
 * Shorthand for binding {@link ReadonlyAttributeEntry.onChange}
 * observers to multiple attributes in a single container.
 * 
 * For all attributes that have a value set, the observers are called immediately.
 * 
 * @param attributes The attribute container to observe.
 * @param bindings Observers assigned to individual attributes by their names.
 * @returns Handle to the registered observers that can be used to unhook them.
 * 
 * @example
 * ```
 * setAttributeBindings(attributeMap, {
 *     foo() {
 *         console.log('foo changed');
 *     }
 *     bar(newValue) {
 *         console.log('bar changed to ' + newValue);
 *     }
 * });
 * ```
 */
export function setAttributeBindings(attributes: ReadonlyAttributeMap, bindings: Record<string, (newValue: string | undefined, oldValue: string | undefined) => void>): ObserverHandle {
    const observers: ObserverHandle[] = [];
    for (const key in bindings) {
        const record = attributes[key];
        const observer = record.onChange.hook(bindings[key]);
        if (record.value !== undefined)
            bindings[key](record.value, undefined);
        observers.push(observer);
    }
    return {
        unhook(): void {
            observers.forEach(observer => observer.unhook());
        }
    };
}
