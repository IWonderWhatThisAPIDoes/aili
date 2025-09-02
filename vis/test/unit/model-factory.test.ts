import { beforeEach, describe, expect, it, jest } from '@jest/globals';
import { ContextFreeViewModelFactory, ViewModelFactory } from '../../src/model-factory';
import { ViewportContext } from '../../src/viewport-dom';
import { VisElement } from '../../src/tree';
import { ViewModel } from '../../src/model';

const TAG_NAME = 'abc';
const NOT_TAG_NAME = 'xyz';

describe(ContextFreeViewModelFactory, () => {
    const model = {} as ViewModel;
    const mockConstructorOne = jest
        .fn<(...args: unknown[]) => ViewModel>(() => model)
        .mockName('ConstructorOne');
    const mockFallbackConstructor = jest
        .fn<(...args: unknown[]) => ViewModel>(() => model)
        .mockName('FallbackConstructor');
    const context = {} as ViewportContext;
    const innerFactory = new ViewModelFactory(
        new Map([[TAG_NAME, mockConstructorOne]]),
        mockFallbackConstructor,
    );
    const factory = new ContextFreeViewModelFactory(innerFactory, context);

    beforeEach(() => {
        mockConstructorOne.mockClear();
        mockFallbackConstructor.mockClear();
    });

    it('provides the correct model for a known tag name', () => {
        const element = new VisElement(TAG_NAME);
        const theModel = factory.createViewModel(element);
        expect(theModel).toBe(model);
        expect(mockConstructorOne).toHaveBeenCalledWith(element, context);
    });

    it('provides the fallback model for an unknown tag name', () => {
        const element = new VisElement(NOT_TAG_NAME);
        const theModel = factory.createViewModel(element);
        expect(theModel).toBe(model);
        expect(mockFallbackConstructor).toHaveBeenCalledWith(element, context);
    });
});
