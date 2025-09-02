import { AttributeMap, AttributeEntry, setAttributeBindings } from '../../src/attributes';
import { expect, describe, it, beforeEach, jest } from '@jest/globals';

const ATTRIBUTE_VALUE = '42';
const OTHER_ATTRIBUTE_VALUE = 'hello';

describe(AttributeEntry, () => {
    let entry: AttributeEntry;
    const mockObserver = jest.fn().mockName('changeObserver');

    beforeEach(() => {
        mockObserver.mockClear();
        entry = new AttributeEntry();
    });

    it('has no initial value', () => {
        expect(entry.value).toBeUndefined();
    });

    it('can be set to a value', () => {
        entry.value = ATTRIBUTE_VALUE;
        expect(entry.value).toBe(ATTRIBUTE_VALUE);
    });

    it('can be reset', () => {
        entry.value = ATTRIBUTE_VALUE;
        entry.value = undefined;
        expect(entry.value).toBe(undefined);
    });

    it('triggers the observer when value changes', () => {
        entry.onChange.hook(mockObserver);
        entry.value = ATTRIBUTE_VALUE;
        entry.value = OTHER_ATTRIBUTE_VALUE;
        entry.value = undefined;
        expect(mockObserver).toHaveBeenCalledWith(ATTRIBUTE_VALUE, undefined);
        expect(mockObserver).toHaveBeenCalledWith(OTHER_ATTRIBUTE_VALUE, ATTRIBUTE_VALUE);
        expect(mockObserver).toHaveBeenCalledWith(undefined, OTHER_ATTRIBUTE_VALUE);
    });

    it('does not trigger the observer when re-emptied', () => {
        entry.onChange.hook(mockObserver);
        entry.value = undefined;
        expect(mockObserver).not.toHaveBeenCalled();
    });

    it('does not trigger the observer when same value is re-assigned', () => {
        entry.value = ATTRIBUTE_VALUE;
        entry.onChange.hook(mockObserver);
        entry.value = ATTRIBUTE_VALUE;
        expect(mockObserver).not.toHaveBeenCalled();
    });
});

describe(AttributeMap, () => {
    let map: AttributeMap;

    beforeEach(() => {
        map = new AttributeMap();
    });

    it('creates entries as requested', () => {
        expect(map.someAttribute).toBeDefined();
    });

    it('initializes entries as empty', () => {
        expect(map.someAttribute.value).toBeUndefined();
    });

    it('keeps entries that have been created', () => {
        map.someAttribute.value = ATTRIBUTE_VALUE;
        expect(map.someAttribute.value).toBe(ATTRIBUTE_VALUE);
    });
});

describe(setAttributeBindings, () => {
    const mockFooChangedObserver = jest.fn().mockName('fooChangedObserver');
    let map: AttributeMap;

    beforeEach(() => {
        mockFooChangedObserver.mockClear();
        map = new AttributeMap();
    });

    it('triggers observers when value is changed later', () => {
        setAttributeBindings(map, { foo: mockFooChangedObserver });
        map.foo.value = ATTRIBUTE_VALUE;
        map.foo.value = OTHER_ATTRIBUTE_VALUE;
        map.foo.value = undefined;
        expect(mockFooChangedObserver).toHaveBeenCalledWith(ATTRIBUTE_VALUE, undefined);
        expect(mockFooChangedObserver).toHaveBeenCalledWith(OTHER_ATTRIBUTE_VALUE, ATTRIBUTE_VALUE);
        expect(mockFooChangedObserver).toHaveBeenCalledWith(undefined, OTHER_ATTRIBUTE_VALUE);
    });

    it('triggers observers when value is already present', () => {
        map.foo.value = ATTRIBUTE_VALUE;
        setAttributeBindings(map, { foo: mockFooChangedObserver });
        expect(mockFooChangedObserver).toHaveBeenCalledWith(ATTRIBUTE_VALUE, undefined);
    });

    it('does not trigger observers when no value is present', () => {
        setAttributeBindings(map, { foo: mockFooChangedObserver });
        expect(mockFooChangedObserver).not.toHaveBeenCalled();
    });

    it('does not trigger observers after unhooking', () => {
        const observer = setAttributeBindings(map, { foo: mockFooChangedObserver });
        observer.unhook();
        map.foo.value = ATTRIBUTE_VALUE;
        expect(mockFooChangedObserver).not.toHaveBeenCalled();
    });
});
