/**
 * Base {@link ViewModel} implementations reused by several models.
 * 
 * @module
 */

import { ObserverHandle } from 'aili-hooligan';
import { ViewLayoutMode, ViewModel } from '../model';
import { NULL_SLOT, ElementViewSlot, ViewSlotPopulator } from '../slots';
import { ReadonlyVisElement } from '../tree';

/**
 * Base class for common view models that consist of a single
 * [flow element](https://developer.mozilla.org/en-US/docs/Web/HTML/Content_categories#flow_content)
 * which also acts as the container for companion elements and connector pins.
 */
export abstract class FlowViewModel implements ViewModel {
    /**
     * Constructs a view model base that manages a provided HTML subtree.
     * 
     * @param wrapperHtml Root element of the view's DOM content.
     *                    This should be created by derived class constructors.
     * @param contentHtml Element that corresponds to the bounding box of the view's
     *                    actual content. Must be a positioned element. Must be a valid container for
     *                    [flow content](https://developer.mozilla.org/en-US/docs/Web/HTML/Content_categories#flow_content).
     *                    If not specified, `wrapperHtml` will be used in its place.
     */
    constructor(wrapperHtml: HTMLElement, contentHtml?: HTMLElement) {
        contentHtml ??= wrapperHtml;
        this.observers = [];
        this.html = wrapperHtml;
        this.pinContainer = contentHtml;
        this.companionSlot = {
            populator: {
                insertFlowHtml(child) {
                    contentHtml.append(child);
                }
            },
            destroy() {},
        };
    }
    useSlot(populator: ViewSlotPopulator | undefined): void {
        if (populator) {
            populator.insertFlowHtml(this.html);
        } else {
            this.html.remove();
        }
    }
    createInlineSlot(child: ReadonlyVisElement, childModel: ViewModel): ElementViewSlot {
        // The default view model does not allow inline children,
        // but implementations can override this
        return NULL_SLOT;
    }
    destroy(): void {
        // Unhook all observers on destruction
        this.observers.forEach(observer => observer.unhook());
    }
    /**
     * Adds an observer to be unhooked when the view is destroyed.
     * 
     * @param handle Handle to the observer that should be unhooked.
     */
    protected unhookOnDestroy(...handle: ObserverHandle[]): void {
        this.observers.push(...handle);
    }
    readonly preferredLayoutMode: ViewLayoutMode = ViewLayoutMode.INLINE;
    readonly companionSlot: ElementViewSlot;
    readonly pinContainer: Element;
    protected readonly html: HTMLElement;
    private readonly observers: ObserverHandle[];
}
