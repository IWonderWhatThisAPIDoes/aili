/**
 * Rendering of a {@link VisTree} structure
 * into another tree.
 *
 * @module
 */

import { ObserverHandle } from 'aili-hooligan';
import { GraphLayoutModel, TAG_GRAPH, TAG_KVT, VisConnector, VisElement } from 'aili-vis';
import {
    VisTree as VisTreeModel,
    VisElement as VisElementModel,
    VisConnector as VisConnectorModel,
} from 'aili-jsapi';
import { VisTree } from './vis-tree';
import { generateRandomColor } from './auto-colors';

/**
 * Watches the structural changes of a vis tree
 * and renders its logical structure into another tree.
 */
export class MetaVisTreeRenderer {
    /**
     * Construct a new renderer.
     *
     * @param source The tree that should be visualized.
     * @param target The tree that receives the visualization.
     */
    constructor(source: VisTree, target: VisTreeModel) {
        this.elementModels = new WeakMap();
        this.connectorModels = new WeakMap();
        this.targetTree = target;
        // Create the target root, which is a graph and never changes.
        this.targetRoot = target.createElement(TAG_GRAPH);
        this.targetRoot.attributes.layout.value = GraphLayoutModel.LAYERED;
        target.root = this.targetRoot;
        // Attach the source root if or when there is one
        if (source.root) {
            this.sourceRootChanged(undefined, source.root);
        }
        source.onRootChanged.hook((newRoot, oldRoot) => this.sourceRootChanged(oldRoot, newRoot));
    }
    /**
     * Updates the target tree when the root element of the source tree changes.
     *
     * @param oldRoot Original root element.
     * @param newRoot New root element.
     */
    private sourceRootChanged(
        oldRoot: VisElement | undefined,
        newRoot: VisElement | undefined,
    ): void {
        if (oldRoot) {
            this.removeSourceElementWithRelations(oldRoot);
        }
        if (newRoot) {
            this.addSourceElement(newRoot);
        }
    }
    /**
     * Completely removes a source element from the visualizations,
     * including its subtree and all attached connectors.
     *
     * @param elem The element to remove.
     */
    private removeSourceElementWithRelations(elem: VisElement): void {
        // Drop all children of the removed element recursively
        for (const child of elem.children) {
            this.removeSourceElementWithRelations(child);
        }
        // Drop all connectors on the removed element
        for (const pin of elem.pins) {
            this.removeSourceConnector(pin.connector);
        }
        // Unhook and destroy the visualization of the element itself
        this.removeSourceElement(elem);
    }
    /**
     * Detaches a source element from the renderer.
     *
     * @param elem The element to remove.
     */
    private removeSourceElement(elem: VisElement): void {
        const elemData = this.elementModels.get(elem);
        if (!elemData) {
            return;
        }
        // Unhook observers
        elemData.addChildObserver.unhook();
        elemData.addPinObserver.unhook();
        elemData.parentChangedObserver.unhook();
        // Unlink related objects
        elemData.targetElement.parent = undefined;
        if (elemData.linkFromParent) {
            elemData.linkFromParent.start.target = undefined;
        }
        // Drop the mapped data
        this.elementModels.delete(elem);
    }
    /**
     * Registers a new source element with the renderer
     * and creates visualization for it.
     *
     * @param elem New source element to be added.
     */
    private addSourceElement(elem: VisElement): void {
        // Do nothing if the element is already present
        if (this.elementModels.has(elem)) {
            return;
        }
        // Construct the visualization
        const targetElement = this.targetTree.createElement(TAG_KVT);
        targetElement.attributes.title.value = elem.tagName;
        targetElement.parent = this.targetRoot;
        // Connect it to its parent unless it is root
        let linkFromParent: VisConnectorModel | undefined;
        const parentData = elem.parent && this.elementModels.get(elem.parent);
        if (parentData !== undefined) {
            linkFromParent = this.targetTree.createConnector();
            linkFromParent.start.target = parentData.targetElement;
            linkFromParent.end.target = targetElement;
            linkFromParent.end.attributes.anchor.value = 'north';
        }
        // Set modification hooks
        const addChildObserver = elem.onAddChild.hook(child => this.addSourceElement(child));
        const addPinObserver = elem.onAddPin.hook(pin => this.addSourceConnector(pin.connector));
        const parentChangedObserver = elem.onParentChanged.hook(parent =>
            this.sourceElementParentChanged(elem, parent),
        );
        // Store the data for future use
        this.elementModels.set(elem, {
            targetElement,
            linkFromParent,
            addChildObserver,
            addPinObserver,
            parentChangedObserver,
        });
        // Load children that are currently present
        for (const child of elem.children) {
            this.addSourceElement(child);
        }
        // Load connectors that are already inside the tree
        for (const pin of elem.pins) {
            this.addSourceConnector(pin.connector);
        }
    }
    /**
     * Handles the reassignment of a tracked element to a different parent.
     *
     * @param child The element whose parent is changing.
     * @param newParent New parent element, if any.
     */
    private sourceElementParentChanged(child: VisElement, newParent: VisElement | undefined): void {
        const childData = this.elementModels.get(child);
        // Do not react to parent change on the root element
        if (!childData || !childData.linkFromParent) {
            return;
        }
        const newParentData = newParent && this.elementModels.get(newParent);
        if (newParentData === undefined) {
            // Destroy the child if its new parent is not in the tree
            this.removeSourceElementWithRelations(child);
        } else {
            // Re-link to the new parent
            childData.linkFromParent.start.target = newParentData.targetElement;
        }
    }
    /**
     * Registers a new source connector with the renderer
     * and creates visualization for it.
     *
     * @param conn New source connector to be added.
     */
    private addSourceConnector(conn: VisConnector): void {
        // Do nothing if the connector is already present
        if (this.connectorModels.has(conn)) {
            return;
        }
        // Get both endpoints and do nothing if either is not in the tree
        const startData = conn.start.target && this.elementModels.get(conn.start.target);
        const endData = conn.end.target && this.elementModels.get(conn.end.target);
        if (!startData || !endData) {
            return;
        }
        // Construct the visualization
        const targetConnector = this.targetTree.createConnector();
        targetConnector.start.attributes.decoration.value = 'circle';
        targetConnector.end.attributes.decoration.value = 'arrow';
        targetConnector.attributes.stroke.value = generateRandomColor();
        targetConnector.attributes.shape.value = 'quadratic';
        targetConnector.start.target = startData.targetElement;
        targetConnector.end.target = endData.targetElement;
        // Set modification hooks
        const startChangedObserver = conn.start.onTargetChanged.hook(target =>
            this.sourceConnectorTargetChanged(conn, target, false),
        );
        const endChangedObserver = conn.end.onTargetChanged.hook(target =>
            this.sourceConnectorTargetChanged(conn, target, true),
        );
        // Store the data for future use
        this.connectorModels.set(conn, {
            targetConnector,
            startChangedObserver,
            endChangedObserver,
        });
    }
    /**
     * Handles the reassignment of a tracked connector to a different target element.
     *
     * @param conn The connector whose target is changing.
     * @param newTarget New target element, if any.
     * @param isEnd Whether the pin that was reassigned is the end of the connector,
     * 				rather than the start.
     */
    private sourceConnectorTargetChanged(
        conn: VisConnector,
        newTarget: VisElement | undefined,
        isEnd: boolean,
    ): void {
        const connData = this.connectorModels.get(conn);
        if (!connData) {
            return;
        }
        const elemData = newTarget && this.elementModels.get(newTarget);
        if (!elemData) {
            // Destroy the visualization if the target got moved out of the tree
            this.removeSourceConnector(conn);
        } else {
            // Redirect the visualization to the matching element
            const targetPin = isEnd ? connData.targetConnector.end : connData.targetConnector.start;
            targetPin.target = elemData.targetElement;
        }
    }
    /**
     * Detaches a source connector from the renderer.
     *
     * @param conn The connector to remove.
     */
    private removeSourceConnector(conn: VisConnector): void {
        const connData = this.connectorModels.get(conn);
        if (!connData) {
            return;
        }
        // Unhook the connector from everything
        connData.startChangedObserver.unhook();
        connData.endChangedObserver.unhook();
        connData.targetConnector.start.target = undefined;
        connData.targetConnector.end.target = undefined;
        // Drop the mapped data
        this.connectorModels.delete(conn);
    }
    /**
     * Tree that receives the visualization.
     */
    private readonly targetTree: VisTreeModel;
    /**
     * Root element of the target visualization.
     */
    private readonly targetRoot: VisElementModel;
    /**
     * Accompanying data for tracked elements.
     */
    private readonly elementModels: WeakMap<VisElement, ElementMetaVisualization>;
    /**
     * Accompanying data for trached connectors.
     */
    private readonly connectorModels: WeakMap<VisConnector, ConnectorMetaVisualization>;
}

/**
 * Data related to the meta-visualization of an element.
 */
interface ElementMetaVisualization {
    addChildObserver: ObserverHandle;
    addPinObserver: ObserverHandle;
    parentChangedObserver: ObserverHandle;
    targetElement: VisElementModel;
    linkFromParent: VisConnectorModel | undefined;
}

/**
 * Data related to the meta-visualization of a connector.
 */
interface ConnectorMetaVisualization {
    startChangedObserver: ObserverHandle;
    endChangedObserver: ObserverHandle;
    targetConnector: VisConnectorModel;
}
