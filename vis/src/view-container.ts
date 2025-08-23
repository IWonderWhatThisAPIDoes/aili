/**
 * Generic view containers.
 *
 * @module
 */

/**
 * Generic container that manages views -
 * renderings associated with a given object.
 *
 * @typeParam T Type of the objects that are viewed by the container.
 * @typeParam R Type of the view.
 */
export abstract class ViewContainer<T extends object, R extends ViewBase> {
    /**
     * Constructs an empty view container.
     */
    constructor() {
        this.views = new WeakMap();
    }
    /**
     * Retrieves the view associated with a specified object
     * or constructs one if needed.
     *
     * @param tag The object whose view should be retrieved.
     * @returns View associated with `tag` and a flag indicating
     *          whether it was just constructed.
     */
    getOrCreate(tag: T): { view: R; created: boolean } {
        const existingView = this.get(tag);
        if (existingView) {
            return { view: existingView, created: false };
        } else {
            const view = this.createNew(tag);
            this.views.set(tag, view);
            return { view, created: true };
        }
    }
    /**
     * Retrieves the view associated with a specified object
     * if one is present.
     *
     * @param tag The object whose view should be retrieved.
     * @returns View associated with `tag`, if any.
     */
    get(tag: T): R | undefined {
        return this.views.get(tag);
    }
    /**
     * Completely destroys an objects's view.
     * The object will be forgotten by the container.
     *
     * @param tag The object to be forgotten.
     */
    remove(tag: T): void {
        const view = this.get(tag);
        if (view) {
            view._destroy();
            this.views.delete(tag);
        }
    }
    /**
     * Constructs a new view associated with a given object.
     *
     * @param tag The object for which a view should be instantiated.
     * @returns New view associated with `tag`.
     */
    protected abstract createNew(tag: T): R;
    private readonly views: WeakMap<T, R>;
}

/**
 * Common interface of all views that can be managed by a {@link ViewContainer}.
 */
export interface ViewBase {
    /**
     * Notifies the view that it is being discarded.
     * Any cleanup procedures go here.
     */
    _destroy(): void;
}
