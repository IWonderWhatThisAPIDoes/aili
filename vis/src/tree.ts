/**
 * Describes structure of the visualization tree.
 * 
 * @module
 */

import { Hook, Hookable } from 'aili-hooligan';
import { AttributeMap, ReadonlyAttributeMap } from './attributes';

/**
 * Read-only view over {@link VisElement} that does not permin modifications.
 * Registering observers is still allowed.
 */
export interface ReadonlyVisElement {
    /**
     * Tag name of the element. It does not change after the element is created.
     */
    readonly tagName: string;
    /**
     * Attributes that describe the element.
     */
    readonly attributes: ReadonlyAttributeMap;
    /**
     * Parent element, or `undefined` if the element is the root of its subtree.
     * 
     * Mirrored by the {@link children} property.
     */
    readonly parent: ReadonlyVisElement | undefined;
    /**
     * Child elements. Mirrors the {@link parent} property.
     */
    readonly children: Iterable<ReadonlyVisElement>;
    /**
     * Connector pins that are attached directly to this element.
     * 
     * Mirrors the {@link ReadonlyVisPin.target} property.
     */
    readonly pins: Iterable<ReadonlyVisPin>;
    /**
     * Connector pins whose projections are attached to this element.
     * 
     * Mirrors the {@link ReadonlyVisPin.projectedTarget} property.
     */
    readonly projectedPins: Iterable<ReadonlyVisPin>;
    /**
     * Connectors whose projections belong to this element.
     * 
     * Mirrors the {@link ReadonlyVisConnector.projectedParent} property.
     */
    readonly projectedConnectors: Iterable<ReadonlyVisConnector>;
    /**
     * Triggers when a pin is added to {@link pins}.
     * The added pin is also passed as the argument to the observers.
     * 
     * @event
     */
    readonly onAddPin: Hookable<[ReadonlyVisPin]>;
    /**
     * Triggers when a pin is added to {@link projectedPins}.
     * The added pin is also passed as the argument to the observers.
     * 
     * The hook is triggered after the matching call
     * to {@link ReadonlyVisPin.onProjectedTargetChanged}.
     * 
     * @event
     */
    readonly onAddProjectedPin: Hookable<[ReadonlyVisPin]>;
    /**
     * Triggers when a connector is added to {@link projectedConnectors}.
     * The added connector is also passed as the argument to the observers.
     * 
     * The hook is triggered after the matching call
     * to {@link ReadonlyVisConnector.onProjectedParentChanged},
     * {@link ReadonlyVisPin.onProjectedTargetChanged}, and {@link onAddProjectedPin}.
     * 
     * @event
     */
    readonly onAddProjectedConnector: Hookable<[ReadonlyVisConnector]>;
    /**
     * Triggers when an element is added to {@link children}.
     * The added element is also passed as the argument to the observers.
     * 
     * The hook is triggered after the matching call to {@link onParentChanged}
     * of the child element.
     * 
     * @event
     */
    readonly onAddChild: Hookable<[ReadonlyVisElement]>;
    /**
     * Triggers when the {@link parent} element changes.
     * The new and previous parent elements are also passed as arguments
     * to the observers.
     * 
     * The hook is triggered before the matching call to {@link onAddChild}
     * of the new parent element.
     * 
     * @event
     */
    readonly onParentChanged: Hookable<[ReadonlyVisElement | undefined, ReadonlyVisElement | undefined]>;
}

/**
 * Read-only view over {@link VisConnector} that does not permin modifications.
 * Registering observers is still allowed.
 */
export interface ReadonlyVisConnector {
    /**
     * Attributes that describe the connector.
     */
    readonly attributes: ReadonlyAttributeMap;
    /**
     * The pin that attaches the start of the connector.
     * It never changes.
     * 
     * Mirrored by the {@link ReadonlyVisPin.connector} property.
     */
    readonly start: ReadonlyVisPin;
    /**
     * The pin that attaches the end of the connector.
     * It never changes.
     * 
     * Mirrored by the {@link ReadonlyVisPin.connector} property.
     */
    readonly end: ReadonlyVisPin;
    /**
     * Nearest common ancestor of the target elements of both pins.
     * `undefined` if either pin is detached or their targets are in unrelated subtrees.
     * 
     * Mirrored by the {@link ReadonlyVisElement.projectedConnectors} property.
     */
    readonly projectedParent: ReadonlyVisElement | undefined;
    /**
     * Triggers when the {@link projectedParent} property changes.
     * The new and previous projected parent elements are also passed as arguments
     * to the observers.
     * 
     * The hook is triggered after the matching call to either
     * {@link ReadonlyVisElement.onAddChild} or {@link ReadonlyVisElement.onAddPin},
     * depending on what modification caused the update.
     * 
     * @event
     */
    readonly onProjectedParentChanged: Hookable<[ReadonlyVisElement | undefined, ReadonlyVisElement | undefined]>;
}

/**
 * Read-only view over {@link VisPin} that does not permin modifications.
 * Registering observers is still allowed.
 */
export interface ReadonlyVisPin {
    /**
     * Attributes that describe the connector.
     */
    readonly attributes: ReadonlyAttributeMap;
    /**
     * The connector that owns this pin.
     * 
     * Mirrors the {@link ReadonlyVisConnector.start} and
     * {@link ReadonlyVisConnector.end} properties.
     */
    readonly connector: ReadonlyVisConnector;
    /**
     * The element that the pin is attached to.
     * 
     * Mirrored by the {@link ReadonlyVisElement.pins} property.
     */
    readonly target: ReadonlyVisElement | undefined;
    /**
     * The element that the pin's projection is attached to.
     * 
     * It is the ancestor (inclusive) of {@link target} that is a child
     * of {@link ReadonlyVisConnector.projectedParent} of this pin's
     * {@link connector}. If {@link ReadonlyVisConnector.projectedParent}
     * is the same element as {@link target}, {@link projectedTarget}
     * is the same element too.
     * 
     * Mirrored by the {@link ReadonlyVisElement.projectedPins} property.
     */
    readonly projectedTarget: ReadonlyVisElement | undefined;
    /**
     * Triggers when the {@link target} property changes.
     * The new and previous target elements are also passed as arguments
     * to the observers.
     * 
     * The hook is triggered before the matching call to {@link ReadonlyVisElement.onAddPin}
     * of the new target element.
     * 
     * @event
     */
    readonly onTargetChanged: Hookable<[ReadonlyVisElement | undefined, ReadonlyVisElement | undefined]>;
    /**
     * Triggers when the {@link projectedTarget} property changes.
     * The new and previous projected target elements are also passed
     * as arguments to the observers.
     * 
     * @event
     */
    readonly onProjectedTargetChanged: Hookable<[ReadonlyVisElement | undefined, ReadonlyVisElement | undefined]>;
}

/**
 * Element of the visualization tree.
 */
export class VisElement implements ReadonlyVisElement {
    /**
     * Constructs a new element with a provided tag name.
     * 
     * @param tagName Tag name of the new element.
     */
    constructor(tagName: string) {
        this.tagName = tagName;
        this.attributes = new AttributeMap();
        this.children = new Set();
        this.pins = new Set();
        this.projectedPins = new Set();
        this.projectedConnectors = new Set();
        this.onAddPin = new Hook();
        this.onAddProjectedPin = new Hook();
        this.onAddProjectedConnector = new Hook();
        this.onAddChild = new Hook();
        this.onParentChanged = new Hook();
    }
    readonly tagName: string;
    readonly attributes: AttributeMap;
    readonly children: Set<VisElement>;
    readonly pins: Set<VisPin>;
    readonly projectedPins: Set<VisPin>;
    readonly projectedConnectors: Set<VisConnector>;
    readonly onAddPin: Hook<[VisPin]>;
    readonly onAddProjectedPin: Hook<[VisPin]>;
    readonly onAddProjectedConnector: Hook<[VisConnector]>;
    readonly onAddChild: Hook<[VisElement]>;
    readonly onParentChanged: Hook<[VisElement | undefined, VisElement | undefined]>;
    get parent(): VisElement | undefined {
        return this._parent;
    }
    /**
     * Insert the element into a new parent or detach it from its current parent.
     * 
     * @throws {@link VisStructuralException} - The modification would violate structural invariants.
     */
    set parent(parent: VisElement | undefined) {
        if (parent === this._parent) {
            // No-op
            return;
        } else if (this.isSameOrAncestorOf(parent)) {
            // A tree must remain a tree
            throw new VisStructuralException('Cannot create a circular parent-child link');
        } else if (this._parent) {
            // Previous parent must let go first
            this._parent.children.delete(this);
        }
        const previousParent = this._parent;
        this._parent = parent;
        parent?.children?.add(this);
        this.onParentChanged.trigger(parent, previousParent);
        parent?.onAddChild?.trigger(this);
        this.updateConnectorProjectionsRecursive();
    }
    private isSameOrAncestorOf(other: VisElement | undefined): boolean {
        while (other) {
            if (this === other) {
                return true;
            }
            other = other._parent;
        }
        return false;
    }
    private updateConnectorProjectionsRecursive(): void {
        for (const pin of this.pins) {
            pin.connector._updateProjection();
        }
        for (const child of this.children) {
            child.updateConnectorProjectionsRecursive();
        }
    }
    /**
     * @internal
     */
    _getPathToRoot(): VisElement[] {
        let current: VisElement | undefined = this;
        let path: VisElement[] = [];
        while (current) {
            path.push(current);
            current = current._parent;
        }
        return path;
    }
    private _parent: VisElement | undefined = undefined;
}

/**
 * Represents one endpoint of a {@link VisConnector}.
 */
export class VisPin implements ReadonlyVisPin {
    /**
     * Constructs a new pin bound to a connector.
     * 
     * This constructor is intended to be called by the owner connector.
     * Users should not need to construct new pins manually.
     * 
     * @internal
     * @param connector Connector that owns the new pin.
     */
    constructor(connector: VisConnector) {
        this.attributes = new AttributeMap();
        this.connector = connector;
        this.onTargetChanged = new Hook();
        this.onProjectedTargetChanged = new Hook();
    }
    readonly attributes: AttributeMap;
    readonly connector: VisConnector;
    get target(): VisElement | undefined {
        return this._target;
    }
    get projectedTarget(): VisElement | undefined {
        return this._projectedTarget;
    }
    readonly onTargetChanged: Hook<[VisElement | undefined, VisElement | undefined]>;
    readonly onProjectedTargetChanged: Hook<[VisElement | undefined, VisElement | undefined]>;
    /**
     * Attach the pin to a new target or detach it from its current parent.
     */
    set target(target: VisElement | undefined) {
        if (target === this._target) {
            // No-op
            return;
        } else if (this._target) {
            // Original target must let go first
            this._target.pins.delete(this);
        }
        const previousTarget = this._target;
        this._target = target;
        target?.pins?.add(this);
        this.onTargetChanged.trigger(target, previousTarget);
        target?.onAddPin?.trigger(this);
        this.connector._updateProjection();
    }
    /**
     * @internal
     */
    _target: VisElement | undefined = undefined;
    /**
     * @internal
     */
    _projectedTarget: VisElement | undefined = undefined;
}

/**
 * A visual connection between two {@link VisElement}s.
 */
export class VisConnector implements ReadonlyVisConnector {
    constructor() {
        this.attributes = new AttributeMap();
        this.start = new VisPin(this);
        this.end = new VisPin(this);
        this.onProjectedParentChanged = new Hook();
    }
    readonly attributes: AttributeMap;
    readonly start: VisPin;
    readonly end: VisPin;
    get projectedParent(): VisElement | undefined {
        return this._projectedParent;
    }
    readonly onProjectedParentChanged: Hook<[VisElement | undefined, VisElement | undefined]>;
    /**
     * Finds the current projection of the connector
     * and updates it if it has changed.
     * 
     * @internal
     */
    _updateProjection(): void {
        // Get previous and current projection
        const { start, end, parent } = this.getCurrentProjectionData() ?? {};
        const previousParent = this.projectedParent;
        const previousStart = this.start.projectedTarget;
        const previousEnd = this.end.projectedTarget;
        // Update projected elements
        this._projectedParent = parent;
        this.start._projectedTarget = start;
        this.end._projectedTarget = end;
        // Update mirror properties
        previousParent?.projectedConnectors?.delete(this);
        parent?.projectedConnectors?.add(this);
        previousStart?.projectedPins?.delete(this.start);
        start?.projectedPins?.add(this.start);
        previousEnd?.projectedPins?.delete(this.end);
        end?.projectedPins?.add(this.end);
        // Trigger observers
        if (previousParent !== parent) {
            this.onProjectedParentChanged.trigger(parent, previousParent);
        }
        if (start !== previousStart) {
            this.start.onProjectedTargetChanged.trigger(start, previousStart);
            start?.onAddProjectedPin?.trigger(this.start);
        }
        if (end !== previousEnd) {
            this.end.onProjectedTargetChanged.trigger(end, previousEnd);
            end?.onAddProjectedPin?.trigger(this.end);
        }
        if (previousParent !== parent) {
            parent?.onAddProjectedConnector?.trigger(this);
        }
    }
    private getCurrentProjectionData(): { start: VisElement, end: VisElement, parent: VisElement } | undefined {
        if (!this.start.target || !this.end.target) {
            // Quick return: we know there is no projection
            // if the connector is not even attached
            return undefined;
        }
        const pathFromStart = this.start.target._getPathToRoot();
        const pathFromEnd = this.end.target._getPathToRoot();
        // Traverse both paths from the root to find the nearest
        // common ancestor of both elements
        let commonAncestor: VisElement | undefined = undefined;
        // Edge case: If a connector connects an element to itself,
        // both paths would start reporting undefined while being equal
        while (pathFromStart.length > 0 && pathFromStart.at(-1) === pathFromEnd.at(-1)) {
            commonAncestor = pathFromStart.at(-1);
            pathFromStart.pop();
            pathFromEnd.pop();
        }
        if (!commonAncestor) {
            // The pins are in two unrelated trees,
            // no projection will be made
            return undefined;
        } else {
            return {
                parent: commonAncestor,
                // Edge case: if a connector goes from an element into its own
                // subtree, use that element as the projected endpoint
                start: pathFromStart.at(-1) ?? commonAncestor,
                end: pathFromEnd.at(-1) ?? commonAncestor,
            };
        }
    }
    private _projectedParent: VisElement | undefined = undefined;
}

/**
 * Exception that indicates a modification was attempted
 * that would violate structural invariants of the visualization tree.
 */
export class VisStructuralException extends Error {}
