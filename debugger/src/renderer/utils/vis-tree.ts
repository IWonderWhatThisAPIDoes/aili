/**
 * Concrete implementation of the {@link VisTreeModel}.
 *
 * @module
 */

import { VisTree as VisTreeModel } from 'aili-jsapi';
import { VisConnector, VisElement } from 'aili-vis';
import { Hook, Hookable } from 'aili-hooligan';

/**
 * Implementation of {@link VisTreeModel} backed by
 * {@link VisElement} trees.
 */
export class VisTree implements VisTreeModel {
    constructor() {
        this._onRootChanged = new Hook();
    }
    createElement(tagName: string): VisElement {
        return new VisElement(tagName);
    }
    createConnector(): VisConnector {
        return new VisConnector();
    }
    set root(root: VisElement) {
        this._root = root;
        this._onRootChanged.trigger(root);
    }
    /**
     * Retrieves the root element of the tree if one is set.
     */
    get root(): VisElement | undefined {
        return this._root;
    }
    /**
     * Hook that triggers after {@link root} is updated.
     */
    get onRootChanged(): Hookable<[VisElement]> {
        return this._onRootChanged;
    }
    private _root: VisElement | undefined;
    private _onRootChanged: Hook<[VisElement]>;
}
