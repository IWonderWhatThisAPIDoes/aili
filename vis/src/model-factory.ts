/**
 * Factory for {@link ViewModel}s.
 * 
 * @module
 */

import { ViewModel } from './model';
import { ReadonlyVisElement } from './tree';
import { ViewportContext } from './viewport-dom';

/**
 * Constructor for {@link ViewModel} that can be used with a {@link ContextFreeViewModelFactory}.
 * 
 * @param element Element that should be modeled.
 * @param context Context for constructing visuals.
 */
export type ViewModelConstructor = new (element: ReadonlyVisElement, context: ViewportContext) => ViewModel;

/**
 * Factory that provides {@link ViewModel}s for {@link ReadonlyVisElement}s
 * under a specified context.
 */
export class ViewModelFactory {
    /**
     * Constructs a factory that constructs the provided model definitions.
     * 
     * @param models View model types, mapped by {@link ReadonlyVisElement.tagName}
     *        of the element that they should be used for.
     * @param fallback The view model type to be used when {@link ReadonlyVisElement.tagName}
     *        of the element does not have a model assiciated with it.
     */
    constructor(models: ReadonlyMap<string, ViewModelConstructor>, fallback: ViewModelConstructor) {
        this.models = models;
        this.fallback = fallback;
    }
    /**
     * Constructs a view model for a provided element
     * under a provided context.
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
     * @param context Context that is passed to the view model.
     * @returns New view model for `element`.
     */
    createViewModel(element: ReadonlyVisElement, context: ViewportContext): ViewModel {
        const constructor = this.models.get(element.tagName) ?? this.fallback;
        return new constructor(element, context);
    }
    private readonly models: ReadonlyMap<string, ViewModelConstructor>;
    private readonly fallback: ViewModelConstructor;
}

/**
 * Factory that provides {@link ViewModel}s for {@link ReadonlyVisElement}s
 * without the need for a context.
 * 
 * Wraps a {@link ViewModelFactory}. All requests get forwarded to that factory
 * along with a static context.
 */
export class ContextFreeViewModelFactory {
    /**
     * Constructs a factory that wraps a given {@link ViewModelFactory}.
     * 
     * @param factory The inner factory that model construction is forwarded to.
     * @param context Context for constructing visuals. Will be passed
     *        on to the models constructed by the factory.
     */
    constructor(factory: ViewModelFactory, context: ViewportContext) {
        this.factory = factory;
        this.context = context;
    }
    /**
     * Constructs a view model for a provided element.
     * 
     * The constructed view model is linked to the element.
     * 
     * The model is created by the wrapped {@link ViewModelFactory},
     * using the context provided on construction.
     * 
     * @param element Element for which a model should be created.
     * @returns New view model for `element`.
     */
    createViewModel(element: ReadonlyVisElement): ViewModel {
        return this.factory.createViewModel(element, this.context);
    }
    private readonly factory: ViewModelFactory;
    private readonly context: ViewportContext;
}
