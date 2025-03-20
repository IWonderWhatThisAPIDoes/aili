/**
 * Container for tracking slots of graph view model
 * and managing their layout.
 * 
 * @module
 */

import { setAttributeBindings } from '../../attributes';
import { ObserverHandle } from '../../hook';
import { ReadonlyVisConnector, ReadonlyVisElement } from '../../tree';
import { ViewportContext } from '../../viewport-dom';
import { GraphLayout, LayoutEdge, LayoutNode } from './layout';
import * as bind from '../../attribute-binds';

/**
 * Identifier of an invalid graph slot.
 */
export const INVALID_GRAPH_SLOT: string = '';

/**
 * Tracks active element slots for a graph view model.
 * 
 * Controls slot layout using a provided layout engine.
 * Layout is recalculated when structure of the visualization tree
 * or the visual models of child elements change, and immediately
 * reflected in the DOM. A layout update can also be triggered manually.
 */
export class GraphSlotManager {
    /**
     * Constructs an empty slot manager and binds it to a DOM element.
     * 
     * @param container DOM element that will contain the slots.
     * @param layout Layout engine that will be used to build the slots' layout.
     * @param context Context for constructing visual objects.
     */
    constructor(container: HTMLElement, layout: GraphLayout, context: ViewportContext) {
        this.container = container;
        this.context = context;
        this.layout = layout;
        this.slots = {};
        this.slotAssignments = new WeakMap();
        this.layoutConnectors = new WeakMap();
        this.slotResizeObserver = new ResizeObserver(e => this.slotsResized(e));
    }
    /**
     * Registers a new slot bound to a visual element and DOM element
     * and inserts it into the DOM.
     * 
     * Layout will not be affected. Call {@link updateLayout} explicitly
     * after calling this to commit structural changes to layout.
     * 
     * @param slotHtml HTMl element that represents the new slot.
     * @param element Visual element for which the slot is intended.
     *                Provides an identity for the slot and uses its
     *                projected connectors to calculate layout.
     * @returns Unique identifier of the new slot. {@link INVALID_GRAPH_SLOT}
     *          if the visual element already has a slot assigned.
     */
    addSlot(slotHtml: HTMLElement, element: ReadonlyVisElement): string {
        const { node, created: isNewNode } = this.getOrAddElementToLayout(element);
        if (!isNewNode) {
            // An element should never have more than one embedding,
            // but in case something goes wrong, make sure nothing else breaks
            return INVALID_GRAPH_SLOT;
        }

        this.container.append(slotHtml);
        slotHtml.dataset.graphSlotId = node.id;

        // Connectors projected into the new element may be relevant to layout
        for (const pin of element.projectedPins) {
            // Do not recalculate layout after every insertion, there may be
            // multiple insertions, and the layout will be recalculated anyway
            // after something is inserted into the new slot
            this.tryAddConnectorToLayout(pin.connector);
        }
        const addProjectedPinObserver = element.onAddProjectedPin.hook(pin => {
            if (this.tryAddConnectorToLayout(pin.connector)) {
                this.updateLayout();
            }
        });

        const attributeBindings = setAttributeBindings(element.attributes, {
            'order-children': value => {
                const ordered = value === 'true';
                node.orderedOutEdges = ordered;
                if (ordered) {
                    this.updateLayout();
                }
            }
        });

        // Update bounding box of the node and recalculate layout whenever the node's size changes
        this.slotResizeObserver.observe(slotHtml);

        this.slots[node.id] = {
            html: slotHtml,
            element,
            addProjectedPinObserver,
            attributeBindings
        };

        return node.id
    }
    /**
     * Stops tracking a slot that was previously registered with {@link addSlot}
     * and removes it from the layout and the DOM.
     * 
     * Layout will be updated asynchronously.
     * 
     * @param slotId Identifier returned by {@link addSlot} when the slot was created.
     */
    removeSlot(slotId: string): void {
        const slot = this.slots[slotId];
        if (!slot) {
            return;
        }
        // Stop updating on every resize
        this.slotResizeObserver.unobserve(slot.html);
        // Stop updating modifications
        slot.addProjectedPinObserver.unhook();
        slot.attributeBindings.unhook();
        // Drop all connectors attached to the element
        for (const pin of slot.element.projectedPins) {
            this.removeConnectorFromLayout(pin.connector);
        }
        // Remove from DOM
        slot.html.remove();
        // Remove slot from map
        delete this.slots[slotId];
        // Remove node from layout
        this.layout.removeNode(slotId);
        // Forget the slot data to make it eligible for GC
        this.slotAssignments.delete(slot.element);
        // Recalculate layout now that we have modified it
        this.updateLayout();
    }
    /**
     * Notifies the manager that it is no longer in use.
     */
    destroy(): void {
        // Stop observing everything if there are still active slots left
        this.slotResizeObserver.disconnect();
    }
    /**
     * Asynchronously recalculates layout using the layout engine
     * received on construction and updates the DOM according to it.
     * 
     * @returns Promise that resolves when the layout has been fully updated.
     */
    async updateLayout(): Promise<void> {
        // Recalculate the layout
        await this.layout.recalculateLayout();
        // Update bounding box dimensions
        this.container.style.width = String(this.layout.width);
        this.container.style.height = String(this.layout.height);
        // Update element positions
        for (const nodeId in this.slots) {
            const { html } = this.slots[nodeId];
            const node = this.layout.getNodeById(nodeId);
            html.style.left = String(node.left);
            html.style.top = String(node.top);
        }
        // Connectors will need to be recalculated too
        this.context.jsplumb.repaintEverything();
    }
    private getOrAddElementToLayout(element: ReadonlyVisElement): { node: LayoutNode, created: boolean } {
        const existingNodeId = this.slotAssignments.get(element);
        if (existingNodeId) {
            return {
                node: this.layout.getNodeById(existingNodeId),
                created: false,
            };
        }

        const node = this.layout.addNode();
        this.slotAssignments.set(element, node.id);

        return { node, created: true };
    }
    private tryAddConnectorToLayout(connector: ReadonlyVisConnector): boolean {
        if (this.layoutConnectors.get(connector) !== undefined) {
            // The connector is already included, so we can skip it
            return false;
        }

        const startSlotId = this.slotAssignments.get(connector.start.projectedTarget);
        const endSlotId = this.slotAssignments.get(connector.end.projectedTarget);
        if (startSlotId === undefined || endSlotId === undefined) {
            // The connector's endpoints are not actually in the layout,
            // so we can skip it
            return false;
        }

        // Add the connector to the layout
        const layoutEdge = this.layout.addEdge(startSlotId, endSlotId);

        // Drop the connector when it moves
        const dropCallback = () => this.removeConnectorFromLayout(connector);
        const projectedParentChanged = connector.onProjectedParentChanged.hook(dropCallback);
        const startProjectedTargetChanged = connector.start.onProjectedTargetChanged.hook(dropCallback);
        const endProjectedTargetChanged = connector.end.onProjectedTargetChanged.hook(dropCallback);

        // Listen for attribute updates
        const attributeBindings = setAttributeBindings(connector.attributes, {
            order: value => {
                layoutEdge.order = bind.getNumeric(bind.integer, value);
                this.updateLayout();
            },
        });

        // Put the layout edge and observer handles aside
        // so we can clean them up later
        this.layoutConnectors.set(connector, {
            layoutEdge,
            projectedParentChanged,
            startProjectedTargetChanged,
            endProjectedTargetChanged,
            attributeBindings,
        });

        // Tell the caller that the layout might need updating
        return true;
    }
    private removeConnectorFromLayout(connector: ReadonlyVisConnector): void {
        // Fetch the entry that is going to be removed
        const entry = this.layoutConnectors.get(connector);
        if (!entry) {
            return;
        }
        // Remove the connector from the layout
        this.layout.removeEdge(entry.layoutEdge.id);
        // Unhook all observers
        entry.projectedParentChanged.unhook();
        entry.startProjectedTargetChanged.unhook();
        entry.endProjectedTargetChanged.unhook();
        entry.attributeBindings.unhook();
        // Drop the expiring entry
        this.layoutConnectors.delete(connector);
    }
    private slotsResized(entries: readonly ResizeObserverEntry[]): void {
        for (const entry of entries) {
            const slotId = (entry.target as HTMLElement).dataset.graphSlotId;
            const slot = this.layout.getNodeById(slotId);
            if (!slot) {
                continue;
            }
            slot.setSize(entry.contentRect.width, entry.contentRect.height);
        }
        // Slots will be moved around, but no new resizes should be triggered
        this.updateLayout();
    }
    private container: HTMLElement;
    private context: ViewportContext;
    private readonly layout: GraphLayout;
    private readonly slots: Record<string, GraphSlot>;
    private readonly slotAssignments: WeakMap<ReadonlyVisElement, string>;
    private readonly layoutConnectors: WeakMap<ReadonlyVisConnector, LayoutActiveConnector>;
    private readonly slotResizeObserver: ResizeObserver;
}

/**
 * Data of a slot tracked by a slot manager.
 */
interface GraphSlot {
    /**
     * HTML element that represents the slot.
     */
    html: HTMLElement;
    /**
     * Visual element bound to the slot.
     */
    element: ReadonlyVisElement;
    /**
     * Handle to the observer attached to {@link ReadonlyVisElement.onAddProjectedPin}
     * of {@link element} that will be used to unhook the observer when the slot expires.
     */
    addProjectedPinObserver: ObserverHandle;
    /**
     * Handle to observers attached to {@link ReadonlyVisElement.attributes}
     * that will be used to unhook the observer when the slot expires.
     */
    attributeBindings: ObserverHandle;
}

/**
 * Data of a connector tracked by a slot manager.
 */
interface LayoutActiveConnector {
    /**
     * Edge of the layout graph that represents the connector.
     */
    layoutEdge: LayoutEdge;
    projectedParentChanged: ObserverHandle,
    startProjectedTargetChanged: ObserverHandle,
    endProjectedTargetChanged: ObserverHandle,
    attributeBindings: ObserverHandle,
}
