import { Hook } from '../../src/hook';
import { describe, it, expect, jest, beforeEach } from '@jest/globals';

describe(Hook, () => {
    let hook: Hook;
    let mockObserver = jest.fn();

    beforeEach(() => {
        hook = new Hook();
        mockObserver.mockReset();
    });

    it('calls registered callback when triggered', () => {
        hook.hook(mockObserver);
        hook.trigger();
        expect(mockObserver).toBeCalledTimes(1);
    });

    it('calls multiple callbacks', () => {
        const otherMockObserver = jest.fn();
        hook.hook(mockObserver);
        hook.hook(otherMockObserver);
        hook.trigger();
        expect(mockObserver).toBeCalledTimes(1);
        expect(otherMockObserver).toBeCalledTimes(1);
    });

    it('does not call callback that has been removed', () => {
        const registration = hook.hook(mockObserver);
        registration.unhook();
        hook.trigger();
        expect(mockObserver).not.toBeCalled();
    });

    it('adds up registrations of the same callback', () => {
        hook.hook(mockObserver);
        hook.hook(mockObserver);
        hook.trigger();
        expect(mockObserver).toBeCalledTimes(2);
    });

    it('removes repeated registrations', () => {
        const registration1 = hook.hook(mockObserver);
        const registration2 = hook.hook(mockObserver);
        registration1.unhook();
        registration2.unhook();
        hook.trigger();
        expect(mockObserver).not.toBeCalled();
    });

    it('distinguishes different registrations when unhooking', () => {
        const registration = hook.hook(mockObserver);
        hook.hook(mockObserver);
        // After one call, the registration becomes invalid
        // and cannot be used to unhook another registration
        registration.unhook();
        // We try it anyway, it should do nothing
        registration.unhook();
        hook.trigger();
        expect(mockObserver).toBeCalledTimes(1);
    });

    it('forwards arguments to observers', () => {
        const ARGUMENT_1 = 42;
        const ARGUMENT_2 = 'lipsum';
        const parametrizedHook = new Hook<[number, string]>();
        parametrizedHook.hook(mockObserver);
        parametrizedHook.trigger(ARGUMENT_1, ARGUMENT_2);
        expect(mockObserver).toBeCalledWith(ARGUMENT_1, ARGUMENT_2);
    });
});
