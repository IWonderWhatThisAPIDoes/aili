/**
 * Factory for {@link ViewModel}s.
 * 
 * @module
 */

import { ViewModel } from './model';
import { ReadonlyVisElement } from './tree';
import { ViewportContext } from './viewport-dom';

/**
 * Constructor for {@link ViewModel} that can be used with a {@link ViewModelFactory}.
 * 
 * @param element Element that should be modeled.
 * @param context Context for constructing visuals.
 */
export type ViewModelConstructor = new (element: ReadonlyVisElement, context: ViewportContext) => ViewModel;

/**
 * Factory that provides {@link ViewModel}s for {@link ReadonlyVisElement}s.
 */
export class ViewModelFactory {
    /**
     * Constructs a factory that constructs the provided model definitions.
     * 
     * @param models View model types, mapped by {@link ReadonlyVisElement.tagName}
     *        of the element that they should be used for.
     * @param fallback The view model type to be used when {@link ReadonlyVisElement.tagName}
     *        of the element does not have a model assiciated with it.
     * @param context Context for constructing visuals. Will be passed
     *        on to the models constructed by the factory.
     */
    constructor(models: ReadonlyMap<string, ViewModelConstructor>, fallback: ViewModelConstructor, context: ViewportContext) {
        this.models = models;
        this.fallback = fallback;
        this.context = context;
    }
    /**
     * Constructs a view model for a provided element.
     * 
     * The constructed view model is linked to the element.
     * 
     * The type of the view model is determined by the element's
     * {@link ReadonlyVisElement.tagName}, which is resolved using
     * the map that the factory was constructed with. If the name
     * does not match any specified tag name, the fallback model
     * provided on construction is used instead.
     * 
     * @param element Element for which a model should be created.
     * @returns New view model for `element`.
     */
    createViewModel(element: ReadonlyVisElement): ViewModel {
        const constructor = this.models.get(element.tagName) ?? this.fallback;
        return new constructor(element, this.context);
    }
    private readonly models: ReadonlyMap<string, ViewModelConstructor>;
    private readonly fallback: ViewModelConstructor;
    private readonly context: ViewportContext;
}
