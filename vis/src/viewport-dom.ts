/**
 * DOM content of a viewport.
 *
 * @module
 */

import { ElementViewSlot, ViewSlotPopulator } from './slots';
import * as jsplumb from '@jsplumb/browser-ui';
import './viewport-dom.css';

/**
 * CSS class for the viewport root container.
 */
export const CLASS_VIEWPORT: string = 'aili-vp';
/**
 * CSS class for the inner container that contains rendering of the tree.
 */
export const CLASS_VIEWPORT_INNER: string = 'aili-vp-inner';
/**
 * CSS class for the viewport-wide container that a JSPlumb instance attaches to.
 */
export const CLASS_VIEWPORT_JTK: string = 'aili-jtk-container';

/**
 * Context data for constructing visuals within a given {@link ViewportDOMRoot}.
 */
export interface ViewportContext {
    /**
     * Document that owns the viewport DOM, and, by proxy,
     * must own all elements that are going to be rendered in the viewport.
     */
    ownerDocument: Document;
    /**
     * Instance of the JSPlumb library for rendering connectors.
     */
    jsplumb: jsplumb.BrowserJsPlumbInstance;
}

/**
 * Provides basic DOM content of a viewport.
 */
export class ViewportDOMRoot {
    /**
     * Constructs a viewport root over a provided DOM container.
     *
     * @param container The element that will contain the viewport.
     */
    constructor(container: HTMLElement) {
        const scrollbox = container.ownerDocument.createElement('div');
        const inner = container.ownerDocument.createElement('div');
        scrollbox.className = CLASS_VIEWPORT;
        inner.className = `${CLASS_VIEWPORT_INNER} ${CLASS_VIEWPORT_JTK}`;
        scrollbox.append(inner);
        container.append(scrollbox);
        this.slot = new ViewportRootSlot(inner);
        this.context = {
            ownerDocument: container.ownerDocument,
            jsplumb: jsplumb.newInstance({ container: inner, elementsDraggable: false }),
        };
    }
    /**
     * Special view slot that renders its content into the viewport root.
     */
    readonly slot: ElementViewSlot;
    /**
     * Context for construction of new visuals under the viewport.
     */
    readonly context: ViewportContext;
}

class ViewportRootSlot implements ElementViewSlot {
    constructor(slotElement: HTMLElement) {
        this.populator = {
            insertFlowHtml(html: HTMLElement) {
                slotElement.append(html);
            },
        };
    }
    readonly populator: ViewSlotPopulator;
    destroy(): void {
        // Root slot cannot be destroyed
    }
}
