/**
 * Common interface of all {@link ReadonlyVisElement} visualizations.
 * 
 * @module
 */

import { ElementViewSlot, ViewSlotPopulator } from './slots';
import { ReadonlyVisElement } from './tree';

/**
 * Enumerates the ways in which a child element's rendering
 * can be laid out relative to its parent.
 */
export enum ViewLayoutMode {
    /**
     * The child element is a part of its parent's layout.
     */
    INLINE,
    /**
     * The child element is placed over or near its parent,
     * outside of the parent's layout.
     */
    COMPANION,
}

/**
 * Describes a visual represnetation of a {@link ReadonlyVisElement} -
 * i. e. a graph, table, plot, ...
 * 
 * An instance of this is assigned to each rendered element.
 */
export interface ViewModel {
    /**
     * Layout mode expected by the model.
     * 
     * This should never change.
     */
    readonly preferredLayoutMode: ViewLayoutMode;
    /**
     * A boundary element that connector pins should
     * attach to, if the model provides one.
     * 
     * If no container is provided, connectors that attach
     * to an element rendered with this model
     * will not be rendered.
     */
    readonly pinContainer?: Element | undefined;
    /**
     * Slot for children whose models request being
     * laid out in {@link ViewLayoutMode.COMPANION} mode.
     * 
     * This should always be available (the slot should always
     * be rendered when the parent element is).
     */
    readonly companionSlot: ElementViewSlot;
    /**
     * Creates a rendering slot for a child that requests to be
     * laid out in {@link ViewLayoutMode.INLINE} mode.
     * 
     * @param child The child element that requests the slot.
     * @param childModel The view model of the child requesting the slot.
     *        Implementors can use this to only permit certain view models
     *        in their inline slots.
     * @returns Slot for the child. If the child does not have
     *          valid properties, the slot may not be rendered.
     *          The slot is only intended to be for one use by the child,
     *          i. e. caller must not assume that multiple elements
     *          can be inserted or that the slot remains valid after calling
     *          {@link ElementViewSlot.destroy} (although the implementation
     *          is free to return such slots).
     */
    createInlineSlot(child: ReadonlyVisElement, childModel: ViewModel): ElementViewSlot;
    /**
     * Moves the model's rendering to a new slot.
     * 
     * @param populator Slot that the model should be rendered into.
     *        A value of `undefined` indicates that the model
     *        should unrender itself.
     */
    useSlot(populator: ViewSlotPopulator | undefined): void;
    /**
     * Performs cleanup before the view model expires.
     * 
     * Implementations do not need to call {@link useSlot}
     * with a value of `undefined` (or perform equivalent actions)
     * here, the caller will take care of that.
     * This is specifically intended for other cleanup
     * that normally never happens otherwise.
     */
    destroy(): void;
}
