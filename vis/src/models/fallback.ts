/**
 * The fallback (sentinel) view model.
 * 
 * @module
 */

import { ViewLayoutMode, ViewModel } from '../model';
import { NULL_SLOT, ElementViewSlot } from '../slots';

/**
 * {@link ViewModel} that does not render anything.
 * 
 * Intended to be used as a sentinel model for invalid elements.
 */
export class FallbackViewModel implements ViewModel {
    readonly preferredLayoutMode = ViewLayoutMode.COMPANION;
    readonly pinContainer = undefined;
    readonly companionSlot = NULL_SLOT;
    createInlineSlot(): ElementViewSlot {
        return NULL_SLOT;
    }
    useSlot(): void {
        // This model does not render, so do nothing
    }
    destroy(): void {
        // Nothing to clean up
    }
}
