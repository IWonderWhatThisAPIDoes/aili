/**
 * Slots for embedding renderings.
 *
 * @module
 */

/**
 * A placement into which an element view may render itself.
 */
export interface ElementViewSlot {
    /**
     * Populator that views can insert themselves into.
     */
    readonly populator: ViewSlotPopulator;
    /**
     * Notifies the slot or its owner that the slot
     * is no longer going to be used.
     *
     * Cleanup code goes here.
     */
    destroy(): void;
}

/**
 * Provides an interface for views to insert themselves
 * into a slot.
 *
 * Implementors can subclass this to add their own
 * client-specific embedding methods.
 * The default is to insert the content as
 * [flow HTML](https://developer.mozilla.org/en-US/docs/Web/HTML/Content_categories#flow_content).
 */
export interface ViewSlotPopulator {
    /**
     * Inserts the content in the form of a
     * [flow HTML element](https://developer.mozilla.org/en-US/docs/Web/HTML/Content_categories#flow_content).
     *
     * @param html The element to insert.
     */
    insertFlowHtml(html: HTMLElement): void;
}

/**
 * Slot that never renders.
 */
export const NULL_SLOT: ElementViewSlot = {
    destroy() {
        // Nothing to clean up
    },
    populator: {
        insertFlowHtml() {
            // Nowhere to insert content to
        },
    },
};
