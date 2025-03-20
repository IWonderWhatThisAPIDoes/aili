/**
 * Rendering of {@link ReadonlyVisElement}s.
 * 
 * @module
 */

import { ReadonlyVisElement, VisElement } from './tree';
import { ViewModel, ViewLayoutMode } from './model';
import { ElementViewSlot } from './slots';
import { ViewBase, ViewContainer } from './view-container';
import { ViewModelFactory } from './model-factory';

/**
 * Container that manages the {@link ElementView}s of individual {@link ReadonlyVisElement}s.
 */
export class ElementViewContainer extends ViewContainer<ReadonlyVisElement, ElementView> {
    /**
     * Constructs an empty view container.
     * 
     * @param modelFactory Factory for constructing view models for elements.
     */
    constructor(modelFactory: ViewModelFactory) {
        super();
        this.modelFactory = modelFactory;
    }
    protected createNew(element: VisElement): ElementView {
        const model = this.modelFactory.createViewModel(element);
        return new ElementViewImpl(element, model);
    }
    private readonly modelFactory: ViewModelFactory;
}

/**
 * Describes the embedding of an {@link ElementView},
 * i.e. the place where its respective {@link ReadonlyVisElement}
 * should be rendered.
 * 
 * Possible variants are:
 * - Root (explicit) embedding - {@link slot} is specified explicitly.
 * - Inherited embedding - {@link parent} is specified and
 *   the {@link ElementView} should be embedded in the parent view.
 * - No embedding - both {@link slot} and {@link parent} are `undefined`.
 *   The {@link ElementView} should not be embedded. This is used
 *   to fully deinitialize the view before dropping it.
 */
export interface ViewEmbedding {
    /**
     * Parent view. A {@link ElementViewSlot} will be requested
     * from it to place the view into.
     */
    parent?: ElementView | undefined,
    /**
     * Explicit slot. The {@link ElementView} will be embedded
     * in that slot. This will be indicated by the
     * {@link ElementView.hasExplicitEmbedding} property.
     */
    slot?: ElementViewSlot | undefined,
}

/**
 * Rendering of a single {@link ReadonlyVisElement}.
 */
export interface ElementView extends ViewBase {
    /**
     * Embeds the view in a new placement. Visuals will be updated
     * as needed.
     * 
     * @param embedding New embedding for the view.
     */
    useEmbedding(embedding: ViewEmbedding): void;
    /**
     * The element associated with the view.
     */
    readonly element: ReadonlyVisElement;
    /**
     * View model that renders the element.
     */
    readonly model: ViewModel;
    /**
     * Whether the last call to {@link useEmbedding} has embedded
     * the view in an explicit slot.
     */
    readonly hasExplicitEmbedding: boolean;
}

class ElementViewImpl implements ElementView {
    /**
     * Constructs a view for a given element.
     * 
     * @param element The element to be viewed.
     * @param model View model that determines the rendering of the element.
     *              The model will be owned by the view and will be destroyed
     *              with it.
     */
    constructor(element: ReadonlyVisElement, model: ViewModel) {
        this.element = element;
        this.model = model;
    }
    useEmbedding(embedding: ViewEmbedding): void {
        const slotIsUpToDate = !!embedding.slot && embedding.slot === this.slot;
        if (slotIsUpToDate) {
            // Skip this if everything is already set up
            return;
        }
        this._hasExplicitEmbedding = !!embedding.slot;
        this.moveToSlot(embedding.slot ?? (embedding.parent ? this.getSlot(embedding.parent) : undefined));
    }
    /**
     * Embeds the view in a specified slot.
     * 
     * @param slot New slot for the view.
     */
    private moveToSlot(slot: ElementViewSlot | undefined): void {
        if (slot === this.slot) {
            return;
        }
        this.slot?.destroy();
        this.slot = slot;
        this.model.useSlot(slot?.populator);
    }
    /**
     * Cleans up after the view has expired.
     * 
     * @internal
     */
    _destroy(): void {
        // Switch the view to unembedded
        // Slot switch handler should perform the necessary cleanup
        this.useEmbedding({});
        // Drop the view model, nobody else should own it
        this.model.destroy();
    }
    /**
     * Requests an inline or companion slot for an element
     * from its parent. 
     * 
     * @param parentView View for the parent element.
     * @returns Slot provided by the parent element's model.
     */
    private getSlot(parentView: ElementView): ElementViewSlot {
        const renderMode = this.model.preferredLayoutMode;
        switch (renderMode) {
            case ViewLayoutMode.INLINE:
                return parentView.model.createInlineSlot(this.element, this.model);
            case ViewLayoutMode.COMPANION:
                return parentView.model.companionSlot;
        }
    }
    get hasExplicitEmbedding(): boolean {
        return this._hasExplicitEmbedding;
    }
    readonly element: ReadonlyVisElement;
    readonly model: ViewModel;
    private slot?: ElementViewSlot;
    private _hasExplicitEmbedding: boolean = false;
}
