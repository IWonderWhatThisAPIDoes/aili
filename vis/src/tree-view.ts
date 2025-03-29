/**
 * Rendering of the visualization tree.
 * 
 * @module
 */

import { ConnectorView, ConnectorViewContainer } from './connector-view';
import { ElementView, ElementViewContainer, ViewEmbedding } from './element-view';
import { ObserverHandle } from './hook';
import { ElementViewSlot } from './slots';
import { ReadonlyVisConnector, ReadonlyVisElement, ReadonlyVisPin } from './tree';
import { ViewContainer } from './view-container';

/**
 * Tracks the structure of the visualization tree
 * and updates its rendering as needed.
 * 
 * The view is backed by two containers, {@link ElementViewContainer}
 * and {@link ConnectorViewContainer}, which manage the renderings of individual
 * elements and connectors. This class is responsible for responding
 * to updates of the structure of the visualization tree.
 * Views are created, updated, and destroyed as needed.
 * 
 * ### Tracking invariants
 * 
 * A {@link ReadonlyVisElement} is tracked if and only if it is a root
 * element (registered with {@link addRootElement}) or a descendant thereof.
 * 
 * A {@link ReadonlyVisConnector} is tracked if and only if both
 * of its endpoints are attached to tracked elements.
 */
export class TreeView {
    /**
     * Construct an empty tree view.
     * 
     * @param elements Container for storing element views.
     * @param connectors Container for storing connector views.
     */
    constructor(
        elements: ViewContainer<ReadonlyVisElement, ElementView>,
        connectors: ViewContainer<ReadonlyVisConnector, ConnectorView>
    ) {
        this.elementViews = elements;
        this.connectorViews = connectors;
        this.elementObservers = new WeakMap();
        this.connectorObservers = new WeakMap();
    }
    /**
     * Adds a new element that will remain tracked by the view until forcibly removed.
     * All elements tracked by the view are descendants of a root element.
     * 
     * The view will update itself as needed to always track
     * the whole subtree and all connectors within it.
     * 
     * Tracked trees are expected to be disjoint. No guarantees are made
     * on the behavior if a root is an ancestor of another one.
     * Connectors may connect different trees.
     * 
     * @param rootElement The root element that will be an entry point of the view.
     * @param rootSlot Special view slot that the root will be embedded in.
     *                 All descendant elements will be embedded in slots
     *                 provided by their parents.
     */
    addRootElement(rootElement: ReadonlyVisElement, rootSlot: ElementViewSlot): void {
        this.elementMovedToEmbedding(rootElement, { slot: rootSlot });
    }
    /**
     * Revokes an element's root status, which means the element
     * and its whole subtree will be removed from the visualization.
     * 
     * @param expiringElement The root element that should lose its root status.
     */
    removeRootElement(expiringElement: ReadonlyVisElement): void {
        const view = this.elementViews.get(expiringElement);
        // Only proceed if the element is actually a root
        if (view?.hasExplicitEmbedding) {
            this.removeElementWithSubtreeAndConnectors(expiringElement);
        }
    }
    /**
     * Handles any change in the embedding of a tracked element.
     * 
     * @param element The element whose embedding changed.
     * @param embedding New embedding for the element.
     */
    private elementMovedToEmbedding(
        element: ReadonlyVisElement,
        embedding: ViewEmbedding
    ): void {
        const { view, created: isNewElement } = this.elementViews.getOrCreate(element);
        // If an element was embedded explicitly (with addRootElement),
        // it is stuck that way and cannot be moved
        if (!view.hasExplicitEmbedding) {
            view.useEmbedding(embedding);
        }
        // Traverse whole subtree and register observers for new elements
        if (isNewElement) {
            this.afterAddedNewElement(view);
        }
    }
    /**
     * Handles registration of a new element to the view.
     * 
     * Registers all its descendants recursively, registers
     * connectors that are fully attached to the subtree,
     * and attaches mutation observers to the element
     * so its subtree can be kept up-to-date.
     * 
     * @param view View for the newly added element.
     */
    private afterAddedNewElement(view: ElementView): void {
        // Add all child elements to rendering, even ones that appear in the future
        const addChildElement = (child: ReadonlyVisElement) => {
            this.elementMovedToEmbedding(child, { parent: view });
        }
        for (const child of view.element.children) {
            addChildElement(child);
        }
        const addChildObserver = view.element.onAddChild.hook(addChildElement);

        // Add all attached pins to rendering, even ones that appear in the future
        const addAttachedPin = (pin: ReadonlyVisPin) => {
            this.connectorPinAttached(pin.connector);
        }
        for (const pin of view.element.pins) {
            addAttachedPin(pin);
        }
        const addPinObserver = view.element.onAddPin.hook(addAttachedPin);

        // Parents of root elements are out of scope by definition, so we ignore them
        // Otherwise always remove the element when it is detached
        let parentChangedObserver: ObserverHandle | undefined;
        const dependOnParent = !view.hasExplicitEmbedding;
        if (dependOnParent) {
            parentChangedObserver = view.element.onParentChanged.hook(() => {
                this.removeElementWithSubtreeAndConnectors(view.element);
            });
        }

        // Put all observers aside so they can be unhooked when the element is removed
        this.elementObservers.set(view.element, {
            addChild: addChildObserver,
            addPin: addPinObserver,
            parentChanged: parentChangedObserver,
        });
    }
    /**
     * Handles the attachment of a connector pin to a tracked element.
     * 
     * @param connector Connector that has been attached to a tracked element.
     */
    private connectorPinAttached(connector: ReadonlyVisConnector): void {
        const start = connector.start.target && this.elementViews.get(connector.start.target);
        const end = connector.end.target && this.elementViews.get(connector.end.target);
        if (!start || !end) {
            // This function should be called when one pin gets attached
            // to the visualization tree, but the other may still be detached.
            // In that case, ignore the connector
            return;
        }

        // Update the connector's view
        const { view, created: isNewConnector } = this.connectorViews.getOrCreate(connector);
        view.useEndpoints(start, end);

        if (isNewConnector) {
            // Remove the connector if it is detached from the element
            const startObserver = connector.start.onTargetChanged.hook(() => this.removeConnector(connector));
            const endObserver = connector.end.onTargetChanged.hook(() => this.removeConnector(connector));
            // Put the observers aside so they can be unhooked when the connector is removed
            this.connectorObservers.set(connector, {
                startTargetChanged: startObserver,
                endTargetChanged: endObserver
            });
        }
    }
    /**
     * Stops tracking an element, its whole subtree, and all connectors attached to it.
     * 
     * @param element The element to remove.
     */
    private removeElementWithSubtreeAndConnectors(element: ReadonlyVisElement): void {
        // Drop the element itself
        this.removeElement(element);
        // Remove all its children, with subtrees and connectors (recursively)
        for (const child of element.children) {
            this.removeElementWithSubtreeAndConnectors(child);
        }
        // Remove all connectors attached to the element
        // (some of them may actually not have views associated
        // because their other pin is detached)
        for (const pin of element.pins) {
            this.removeConnector(pin.connector);
        }
    }
    /**
     * Stops tracking an element.
     * 
     * Removes all observers attached to the element
     * and destroys its view.
     * 
     * @param element The element to remove.
     */
    private removeElement(element: ReadonlyVisElement): void {
        // Unhook all observers that manage the element
        const observers = this.elementObservers.get(element);
        observers.addChild.unhook();
        observers.addPin.unhook();
        observers.parentChanged?.unhook();
        this.elementObservers.delete(element);
        // Drop the element's view
        this.elementViews.remove(element);
    }
    /**
     * Stops tracking a connector.
     * 
     * Removes all observers attached to the element
     * and destroys its view.
     * 
     * @param connector The connector to remove.
     */
    private removeConnector(connector: ReadonlyVisConnector): void {
        const observers = this.connectorObservers.get(connector);
        if (!observers) {
            return;
        }
        // Unhook all observers that manage the connector
        observers.startTargetChanged.unhook();
        observers.endTargetChanged.unhook();
        this.connectorObservers.delete(connector);
        // Drop the connector's view
        this.connectorViews.remove(connector);
    }
    private readonly elementViews: ViewContainer<ReadonlyVisElement, ElementView>;
    private readonly connectorViews: ViewContainer<ReadonlyVisConnector, ConnectorView>;
    private readonly elementObservers: WeakMap<ReadonlyVisElement, ElementObservers>;
    private readonly connectorObservers: WeakMap<ReadonlyVisConnector, ConnectorObservers>;
}

/**
 * Handles to observers that listen for modifications of an element
 * that is tracked by a {@link TreeView}.
 */
interface ElementObservers {
    addChild: ObserverHandle;
    addPin: ObserverHandle;
    parentChanged?: ObserverHandle;
}

/**
 * Handles to observers that listen for modifications of a connector
 * that is tracked by a {@link TreeView}.
 */
interface ConnectorObservers {
    startTargetChanged: ObserverHandle;
    endTargetChanged: ObserverHandle;
}
