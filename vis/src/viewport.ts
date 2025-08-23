/**
 * Defines the main rendering class.
 *
 * @module
 */

import { ReadonlyVisElement } from './tree';
import { ElementViewContainer } from './element-view';
import { ConnectorViewContainer } from './connector-view';
import { TreeView } from './tree-view';
import { ViewportDOMRoot } from './viewport-dom';
import { ContextFreeViewModelFactory, ViewModelFactory } from './model-factory';

/**
 * The main rendering class that can be attached to the DOM
 * and renders the visualization tree into it.
 */
export class Viewport {
    /**
     * Constructs a new viewport that renders a provided visualization
     * tree into a provided DOM container.
     *
     * @param container The DOM element that the viewport will render to.
     * @param viewModelFactory Factory that creates view models
     *        for {@link ReadonlyVisElement}s.
     */
    constructor(container: HTMLElement, viewModelFactory: ViewModelFactory) {
        const root = new ViewportDOMRoot(container);
        const modelFactory = new ContextFreeViewModelFactory(viewModelFactory, root.context);
        const elementViews = new ElementViewContainer(modelFactory);
        const connectorViews = new ConnectorViewContainer(root.context);
        const view = new TreeView(elementViews, connectorViews);
        this.treeView = view;
        this.rootDom = root;
    }
    /**
     * Sets the element that is at the root of the viewport.
     *
     * @param newRoot The new root element.
     */
    set root(newRoot: ReadonlyVisElement | undefined) {
        if (newRoot == this.currentRoot) {
            // No-op
            return;
        }
        if (this.currentRoot) {
            // Get rid of the old root so we free up the root slot
            this.treeView.removeRootElement(this.currentRoot);
        }
        if (newRoot) {
            // The root slot is never destroyed, so we can reuse it right away
            this.treeView.addRootElement(newRoot, this.rootDom.slot);
        }
        this.currentRoot = newRoot;
    }
    /**
     * Gets the current root element.
     */
    get root(): ReadonlyVisElement | undefined {
        return this.currentRoot;
    }
    private treeView: TreeView;
    private rootDom: ViewportDOMRoot;
    private currentRoot: ReadonlyVisElement | undefined;
}
