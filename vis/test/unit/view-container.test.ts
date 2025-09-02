import { beforeEach, describe, expect, it, jest } from '@jest/globals';
import { ViewBase, ViewContainer } from '../../src/view-container';

const mockDestructor = jest.fn().mockName('_destroy');
const mockConstructor = jest
    .fn(() => ({ _destroy: mockDestructor }) as ViewBase)
    .mockName('createNew');

class TestViewContainer extends ViewContainer<object, ViewBase> {
    createNew = mockConstructor;
}

describe(ViewContainer, () => {
    const tag = {};
    let container: ViewContainer<object, ViewBase>;

    beforeEach(() => {
        mockConstructor.mockClear();
        mockDestructor.mockClear();
        container = new TestViewContainer();
    });

    describe(ViewContainer.prototype.get, () => {
        it('returns undefined if no view is present', () => {
            expect(container.get(tag)).toBeUndefined();
        });

        it('returns existing value if one has been created', () => {
            const { view } = container.getOrCreate(tag);
            const retrievedView = container.get(tag);
            expect(retrievedView).toBeDefined();
            expect(retrievedView).toBe(view);
        });

        it('returns undefined if a view has been removed', () => {
            container.getOrCreate(tag);
            container.remove(tag);
            expect(container.get(tag)).toBeUndefined();
        });
    });

    describe(ViewContainer.prototype.getOrCreate, () => {
        it('delegates creation to the implementation', () => {
            const constructedView = { _destroy() {} };
            mockConstructor.mockImplementationOnce(() => constructedView);
            const { view } = container.getOrCreate(tag);
            expect(mockConstructor).toHaveBeenCalled();
            expect(view).toBe(constructedView);
        });

        it('does not recreate an existing view', () => {
            container.getOrCreate(tag);
            container.getOrCreate(tag);
            expect(mockConstructor).toHaveBeenCalledTimes(1);
        });

        it('returns the same view on subsequent calls', () => {
            const { view } = container.getOrCreate(tag);
            const { view: retrievedView } = container.getOrCreate(tag);
            expect(retrievedView).toBe(view);
        });

        it('recreates a deleted view', () => {
            container.getOrCreate(tag);
            container.remove(tag);
            container.getOrCreate(tag);
            expect(mockConstructor).toHaveBeenCalledTimes(2);
        });

        it('indicates whether the value was created', () => {
            const { created: createdFirst } = container.getOrCreate(tag);
            const { created: createdSecond } = container.getOrCreate(tag);
            container.remove(tag);
            const { created: createdThird } = container.getOrCreate(tag);
            expect(createdFirst).toBe(true);
            expect(createdSecond).toBe(false);
            expect(createdThird).toBe(true);
        });
    });

    describe(ViewContainer.prototype.remove, () => {
        it('calls the destructor', () => {
            container.getOrCreate(tag);
            container.remove(tag);
            expect(mockDestructor).toHaveBeenCalled();
        });
    });
});
