/**
 * Basic hand-written unit tests for visualization tree structure.
 * 
 * @module
 */

import { VisElement, VisConnector, VisPin, VisStructuralException } from '../../../src/tree';
import { expect, it, describe, jest, beforeEach } from '@jest/globals';

const ELEMENT_TAG_NAME = 'some-tag-name';

describe(VisElement, () => {
    let element: VisElement;

    beforeEach(() => {
        element = new VisElement(ELEMENT_TAG_NAME);
    });

    describe('New element', () => {
        it('has the specified tag name', () => {
            expect(element.tagName).toBe(ELEMENT_TAG_NAME);
        });

        it('has no parent', () => {
            expect(element.parent).toBeUndefined();
        });

        it('has no children', () => {
            expect(element.children).not.toContainEqual(expect.anything());
        });

        it('has no pins', () => {
            expect(element.pins).not.toContainEqual(expect.anything());
        });

        it('has no projected pins', () => {
            expect(element.projectedPins).not.toContainEqual(expect.anything());
        });

        it('has no projected connectors', () => {
            expect(element.projectedConnectors).not.toContainEqual(expect.anything());
        });
    });

    describe('Element inserted into a parent', () => {
        let parent: VisElement;

        beforeEach(() => {
            parent = new VisElement(ELEMENT_TAG_NAME);
            element.parent = parent;
        });

        it('has the parent element as its parent', () => {
            expect(element.parent).toBe(parent);
        });

        it('is a child of the parent element', () => {
            expect(parent.children).toContain(element);
        });
    });

    describe('Element removed from its parent', () => {
        let parent: VisElement;

        beforeEach(() => {
            parent = new VisElement(ELEMENT_TAG_NAME);
            element.parent = parent;
            element.parent = undefined;
        });

        it('has no parent', () => {
            expect(element.parent).toBeUndefined();
        });

        it('is not a child of its former parent', () => {
            expect(parent.children).not.toContain(element);
        });
    });

    describe('Element moved from one parent to another', () => {
        let source: VisElement;
        let dest: VisElement;

        beforeEach(() => {
            source = new VisElement(ELEMENT_TAG_NAME);
            dest = new VisElement(ELEMENT_TAG_NAME);
            element.parent = source;
            element.parent = dest;
        });

        it('has the new parent element as its parent', () => {
            expect(element.parent).toBe(dest);
        });

        it('is a child of the new parent element', () => {
            expect(dest.children).toContain(element);
        });

        it('is not a child of its formet parent', () => {
            expect(source.children).not.toContain(element);
        });
    });

    describe('Parent change observer', () => {
        let mockObserver = jest.fn().mockName('parentChangedObserver');

        beforeEach(() => {
            mockObserver.mockClear();
        });

        it('triggers when an element is inserted into a parent', () => {
            let parent = new VisElement(ELEMENT_TAG_NAME);
            element.onParentChanged.hook(mockObserver);
            element.parent = parent;
            expect(mockObserver).toBeCalledWith(parent, undefined);
        });

        it('triggers when an element is removed from its parent', () => {
            let parent = new VisElement(ELEMENT_TAG_NAME);
            element.parent = parent;
            element.onParentChanged.hook(mockObserver);
            element.parent = undefined;
            expect(mockObserver).toBeCalledWith(undefined, parent);
        });

        it('triggers when an element is moved from one parent to another', () => {
            let source = new VisElement(ELEMENT_TAG_NAME);
            let dest = new VisElement(ELEMENT_TAG_NAME);
            element.parent = source;
            element.onParentChanged.hook(mockObserver);
            element.parent = dest;
            expect(mockObserver).toBeCalledWith(dest, source);
        });

        it('does not trigger when an element is moved to its current parent', () => {
            let parent = new VisElement(ELEMENT_TAG_NAME);
            element.parent = parent;
            element.onParentChanged.hook(mockObserver);
            element.parent = parent;
            expect(mockObserver).not.toBeCalled();
        });

        it('does not trigger when a detached element is removed again', () => {
            element.onParentChanged.hook(mockObserver);
            element.parent = undefined;
            expect(mockObserver).not.toBeCalled();
        });

        it('triggers after relevant properties have been updated', () => {
            mockObserver.mockImplementationOnce(() => {
                expect(element.parent).toBe(parent);
                expect(parent.children).toContain(element);
            });
            let parent = new VisElement(ELEMENT_TAG_NAME);
            element.onParentChanged.hook(mockObserver);
            element.parent = parent;
            expect(mockObserver).toBeCalled();
        });
    });

    describe('Child insertion observer', () => {
        let mockObserver = jest.fn().mockName('addChildObserver');
        let child: VisElement;

        beforeEach(() => {
            mockObserver.mockClear();
            child = new VisElement(ELEMENT_TAG_NAME);
        });

        it('triggers when a child is added', () => {
            element.onAddChild.hook(mockObserver);
            child.parent = element;
            expect(mockObserver).toBeCalledWith(child);
        });

        it('triggers when a child is moved from a different parent', () => {
            let source = new VisElement(ELEMENT_TAG_NAME);
            child.parent = source;
            element.onAddChild.hook(mockObserver);
            child.parent = element;
            expect(mockObserver).toBeCalledWith(child);
        });

        it('does not trigger when a child is moved to its current parent', () => {
            child.parent = element;
            element.onAddChild.hook(mockObserver);
            child.parent = element;
            expect(mockObserver).not.toBeCalled();
        });

        it('triggers after parent change observer', () => {
            let mockParentChangeObserver = jest.fn().mockName('parentChangedObserver');
            child.onParentChanged.hook(mockParentChangeObserver);
            element.onAddChild.hook(mockObserver);
            child.parent = element;
            expect(mockObserver.mock.invocationCallOrder[0])
                .toBeGreaterThan(mockParentChangeObserver.mock.invocationCallOrder[0]);
        });

        it('triggers after relevant properties have been updated', () => {
            mockObserver.mockImplementationOnce(() => {
                expect(element.children).toContain(child);
                expect(child.parent).toBe(element);
            });
            element.onAddChild.hook(mockObserver);
            child.parent = element;
            expect(mockObserver).toBeCalled();
        });
    });

    describe('Pin insertion observer', () => {
        let mockObserver = jest.fn().mockName('addPinObserver');
        let connector: VisConnector;

        beforeEach(() => {
            mockObserver.mockClear();
            connector = new VisConnector();
        });

        it('triggers when a pin is attached', () => {
            element.onAddPin.hook(mockObserver);
            connector.start.target = element;
            expect(mockObserver).toBeCalledWith(connector.start);
        });

        it('triggers when a pin is moved from another element', () => {
            let source = new VisElement(ELEMENT_TAG_NAME);
            connector.start.target = source;
            element.onAddPin.hook(mockObserver);
            connector.start.target = element;
            expect(mockObserver).toBeCalledWith(connector.start);
        });

        it('does not trigger when a pin is moved to its current target', () => {
            connector.start.target = element;
            element.onAddPin.hook(mockObserver);
            connector.start.target = element;
            expect(mockObserver).not.toBeCalled();
        });

        it('triggers after the pin target change observer', () => {
            let mockTargetChangeObserver = jest.fn().mockName('targetChangedObserver');
            element.onAddPin.hook(mockObserver);
            connector.start.onTargetChanged.hook(mockTargetChangeObserver);
            connector.start.target = element;
            expect(mockObserver.mock.invocationCallOrder[0])
                .toBeGreaterThan(mockTargetChangeObserver.mock.invocationCallOrder[0]);
        });

        it('triggers after relevant properties have been updated', () => {
            mockObserver.mockImplementationOnce(() => {
                expect(element.pins).toContain(connector.start);
                expect(connector.start.target).toBe(element);
            });
            element.onAddPin.hook(mockObserver);
            connector.start.target = element;
            expect(mockObserver).toBeCalled();
        });
    });

    describe('Projected pin insertion observer', () => {
        let mockObserver = jest.fn().mockName('addProjectedPinObserver');
        let connector: VisConnector;

        beforeEach(() => {
            mockObserver.mockClear();
            connector = new VisConnector();
        });

        it('triggers when sibling elements are connected', () => {
            let left = new VisElement(ELEMENT_TAG_NAME);
            let right = new VisElement(ELEMENT_TAG_NAME);
            left.parent = element;
            right.parent = element;
            left.onAddProjectedPin.hook(mockObserver);
            connector.start.target = left;
            connector.end.target = right;
            expect(mockObserver).toBeCalledWith(connector.start);
        });

        it('triggers twice when an element is connected to itself', () => {
            element.onAddProjectedPin.hook(mockObserver);
            connector.start.target = element;
            connector.end.target = element;
            expect(mockObserver).toBeCalledWith(connector.start);
            expect(mockObserver).toBeCalledWith(connector.end);
        });

        it('does not trigger when pin moves within the subtree', () => {
            let left = new VisElement(ELEMENT_TAG_NAME);
            let right = new VisElement(ELEMENT_TAG_NAME);
            let child = new VisElement(ELEMENT_TAG_NAME);
            left.parent = element;
            right.parent = element;
            child.parent = left;
            connector.start.target = left;
            connector.end.target = right;
            left.onAddProjectedPin.hook(mockObserver);
            connector.start.target = child;
            expect(mockObserver).not.toBeCalled();
        });

        it('triggers when pin moves to a different subtree', () => {
            let source = new VisElement(ELEMENT_TAG_NAME);
            let right = new VisElement(ELEMENT_TAG_NAME);
            let dest = new VisElement(ELEMENT_TAG_NAME);
            source.parent = element;
            right.parent = element;
            dest.parent = element;
            connector.start.target = source;
            connector.end.target = right;
            dest.onAddProjectedPin.hook(mockObserver);
            connector.start.target = dest;
            expect(mockObserver).toBeCalledWith(connector.start);
        });

        it('does not trigger when other pin moves to a different subtree', () => {
            let source = new VisElement(ELEMENT_TAG_NAME);
            let right = new VisElement(ELEMENT_TAG_NAME);
            let dest = new VisElement(ELEMENT_TAG_NAME);
            source.parent = element;
            right.parent = element;
            dest.parent = element;
            connector.start.target = source;
            connector.end.target = right;
            right.onAddProjectedPin.hook(mockObserver);
            connector.start.target = dest;
            expect(mockObserver).not.toBeCalled();
        });

        it('triggers after projected target change observer', () => {
            let mockProjectedTargetChangedObserver = jest.fn().mockName('projectedTargetChangedObserver');
            let left = new VisElement(ELEMENT_TAG_NAME);
            let right = new VisElement(ELEMENT_TAG_NAME);
            left.parent = element;
            right.parent = element;
            connector.start.onProjectedTargetChanged.hook(mockProjectedTargetChangedObserver);
            right.onAddProjectedPin.hook(mockObserver);
            connector.start.target = left;
            connector.end.target = right;
            expect(mockObserver.mock.invocationCallOrder[0])
                .toBeGreaterThan(mockProjectedTargetChangedObserver.mock.invocationCallOrder[0]);
        });

        it('triggers after relevant properties have been updated', () => {
            mockObserver.mockImplementationOnce(() => {
                expect(element.projectedConnectors).toContain(connector);
                expect(left.projectedPins).toContain(connector.start);
                expect(right.projectedPins).toContain(connector.end);
                expect(connector.projectedParent).toBe(element);
                expect(connector.start.projectedTarget).toBe(left);
                expect(connector.end.projectedTarget).toBe(right);
            });
            let left = new VisElement(ELEMENT_TAG_NAME);
            let right = new VisElement(ELEMENT_TAG_NAME);
            left.parent = element;
            right.parent = element;
            left.onAddProjectedPin.hook(mockObserver);
            connector.start.target = left;
            connector.end.target = right;
            expect(mockObserver).toBeCalled();
        });
    });

    describe('Projection insertion observer', () => {
        let mockObserver = jest.fn().mockName('addProjectedConnectorObserver');
        let connector: VisConnector;

        beforeEach(() => {
            mockObserver.mockClear();
            connector = new VisConnector();
        });

        it('triggers when sibling elements are connected', () => {
            let left = new VisElement(ELEMENT_TAG_NAME);
            let right = new VisElement(ELEMENT_TAG_NAME);
            left.parent = element;
            right.parent = element;
            element.onAddProjectedConnector.hook(mockObserver);
            connector.start.target = left;
            connector.end.target = right;
            expect(mockObserver).toBeCalledWith(connector);
        });

        it('does not trigger when pin moves to a different subtree', () => {
            let source = new VisElement(ELEMENT_TAG_NAME);
            let right = new VisElement(ELEMENT_TAG_NAME);
            let dest = new VisElement(ELEMENT_TAG_NAME);
            source.parent = element;
            right.parent = element;
            dest.parent = element;
            connector.start.target = source;
            connector.end.target = right;
            element.onAddProjectedConnector.hook(mockObserver);
            connector.start.target = dest;
            expect(mockObserver).not.toBeCalled();
        });

        it('triggers when a pin moves under a different ancestor', () => {
            let parent = new VisElement(ELEMENT_TAG_NAME);
            let dest = new VisElement(ELEMENT_TAG_NAME);
            let left = new VisElement(ELEMENT_TAG_NAME);
            let right = new VisElement(ELEMENT_TAG_NAME);
            element.parent = parent;
            dest.parent = parent;
            left.parent = element;
            right.parent = element;
            connector.start.target = left;
            connector.end.target = right;
            parent.onAddProjectedConnector.hook(mockObserver);
            connector.start.target = dest;
            expect(mockObserver).toBeCalledWith(connector);
        });

        it('triggers after projected parent change observer', () => {
            let mockProjectedParentChangeObserver = jest.fn().mockName('projectedParentChangedObserver');
            let left = new VisElement(ELEMENT_TAG_NAME);
            let right = new VisElement(ELEMENT_TAG_NAME);
            left.parent = element;
            right.parent = element;
            element.onAddProjectedConnector.hook(mockObserver);
            connector.onProjectedParentChanged.hook(mockProjectedParentChangeObserver);
            connector.start.target = left;
            connector.end.target = right;
            expect(mockObserver.mock.invocationCallOrder[0])
                .toBeGreaterThan(mockProjectedParentChangeObserver.mock.invocationCallOrder[0]);
        });

        it('triggers after projected target change observer', () => {
            let mockStartProjectedTargetChangedObserver = jest.fn().mockName('startProjectedTargetChangedObserver');
            let mockEndProjectedTargetChangedObserver = jest.fn().mockName('endProjectedTargetChangedObserver');
            let left = new VisElement(ELEMENT_TAG_NAME);
            let right = new VisElement(ELEMENT_TAG_NAME);
            left.parent = element;
            right.parent = element;
            element.onAddProjectedConnector.hook(mockObserver);
            connector.start.onProjectedTargetChanged.hook(mockStartProjectedTargetChangedObserver);
            connector.end.onProjectedTargetChanged.hook(mockEndProjectedTargetChangedObserver);
            connector.start.target = left;
            connector.end.target = right;
            expect(mockObserver.mock.invocationCallOrder[0])
                .toBeGreaterThan(mockStartProjectedTargetChangedObserver.mock.invocationCallOrder[0]);
            expect(mockObserver.mock.invocationCallOrder[0])
                .toBeGreaterThan(mockEndProjectedTargetChangedObserver.mock.invocationCallOrder[0]);
        });

        it('triggers after relevant properties have been updated', () => {
            mockObserver.mockImplementationOnce(() => {
                expect(element.projectedConnectors).toContain(connector);
                expect(left.projectedPins).toContain(connector.start);
                expect(right.projectedPins).toContain(connector.end);
                expect(connector.projectedParent).toBe(element);
                expect(connector.start.projectedTarget).toBe(left);
                expect(connector.end.projectedTarget).toBe(right);
            });
            let left = new VisElement(ELEMENT_TAG_NAME);
            let right = new VisElement(ELEMENT_TAG_NAME);
            left.parent = element;
            right.parent = element;
            element.onAddProjectedConnector.hook(mockObserver);
            connector.start.target = left;
            connector.end.target = right;
            expect(mockObserver).toBeCalled();
        });
    });
});

describe(VisConnector, () => {
    let connector: VisConnector;

    beforeEach(() => {
        connector = new VisConnector();
    });

    describe('New connector', () => {
        it('is the parent of its pins', () => {
            expect(connector.start.connector).toBe(connector);
            expect(connector.end.connector).toBe(connector);
        });

        it('has no connections', () => {
            expect(connector.start.target).toBeUndefined();
            expect(connector.end.target).toBeUndefined();
        });

        it('has no projection', () => {
            expect(connector.projectedParent).toBeUndefined();
            expect(connector.start.projectedTarget).toBeUndefined();
            expect(connector.end.projectedTarget).toBeUndefined();
        });
    });

    describe('Pin attached to an element', () => {
        let element: VisElement;

        beforeEach(() => {
            element = new VisElement(ELEMENT_TAG_NAME);
            connector.start.target = element;
        });

        it('has the element set as its target', () => {
            expect(connector.start.target).toBe(element);
        });

        it('is a pin of the element', () => {
            expect(element.pins).toContain(connector.start);
        });
    });

    describe('Pin detached from an element', () => {
        let element: VisElement;

        beforeEach(() => {
            element = new VisElement(ELEMENT_TAG_NAME);
            connector.start.target = element;
            connector.start.target = undefined;
        });

        it('has no target', () => {
            expect(connector.start.target).toBeUndefined();
        });

        it('is not a pin of its former target', () => {
            expect(element.pins).not.toContain(connector.start); 
        });
    });

    describe('Pin moved from one element to another', () => {
        let source: VisElement;
        let dest: VisElement;

        beforeEach(() => {
            source = new VisElement(ELEMENT_TAG_NAME);
            dest = new VisElement(ELEMENT_TAG_NAME);
            connector.start.target = source;
            connector.start.target = dest;
        });

        it('has the new element set as its target', () => {
            expect(connector.start.target).toBe(dest);
        });

        it('is a pin of its new target', () => {
            expect(dest.pins).toContain(connector.start);
        });

        it('is not a pin of its former target', () => {
            expect(source.pins).not.toContain(connector.start);
        });
    });

    describe('Connector attached to sibling elements', () => {
        let parent: VisElement;
        let left: VisElement;
        let right: VisElement;

        beforeEach(() => {
            parent = new VisElement(ELEMENT_TAG_NAME);
            left = new VisElement(ELEMENT_TAG_NAME);
            right = new VisElement(ELEMENT_TAG_NAME);
            left.parent = parent;
            right.parent = parent;
            connector.start.target = left;
            connector.end.target = right;
        });

        it('has a projection in the parent element', () => {
            expect(connector.projectedParent).toBe(parent);
        });

        it('has end points projected into the sibling elements', () => {
            expect(connector.start.projectedTarget).toBe(left);
            expect(connector.end.projectedTarget).toBe(right);
        });

        it('is projected in the parent element', () => {
            expect(parent.projectedConnectors).toContain(connector);
        });

        it('has projections of end points in the sibling elements', () => {
            expect(left.projectedPins).toContain(connector.start);
            expect(right.projectedPins).toContain(connector.end);
        });
    });

    describe('Connector detached from one sibling element', () => {
        let parent: VisElement;
        let left: VisElement;
        let right: VisElement;

        beforeEach(() => {
            parent = new VisElement(ELEMENT_TAG_NAME);
            left = new VisElement(ELEMENT_TAG_NAME);
            right = new VisElement(ELEMENT_TAG_NAME);
            left.parent = parent;
            right.parent = parent;
            connector.start.target = left;
            connector.end.target = right;
            connector.start.target = undefined;
        });

        it('has no projection', () => {
            expect(connector.projectedParent).toBeUndefined();
            expect(connector.start.projectedTarget).toBeUndefined();
            expect(connector.end.projectedTarget).toBeUndefined();
        });

        it('is not projected in the parent element', () => {
            expect(parent.projectedConnectors).not.toContain(connector);
        });
    });

    describe('Connector attached to cousin elements', () => {
        let grandparent: VisElement;
        let parent: VisElement;
        let left: VisElement;
        let right: VisElement;
        let lowerLeft: VisElement;
        let lowerRight: VisElement;

        beforeEach(() => {
            grandparent = new VisElement(ELEMENT_TAG_NAME);
            parent = new VisElement(ELEMENT_TAG_NAME);
            left = new VisElement(ELEMENT_TAG_NAME);
            right = new VisElement(ELEMENT_TAG_NAME);
            lowerLeft = new VisElement(ELEMENT_TAG_NAME);
            lowerRight = new VisElement(ELEMENT_TAG_NAME);
            parent.parent = grandparent;
            left.parent = parent;
            right.parent = parent;
            lowerLeft.parent = left;
            lowerRight.parent = right;
            connector.start.target = lowerLeft;
            connector.end.target = lowerRight;
        });

        it('has a projection in the parent element', () => {
            expect(connector.projectedParent).toBe(parent);
        });

        it('has end points projected into children of the parent element', () => {
            expect(connector.start.projectedTarget).toBe(left);
            expect(connector.end.projectedTarget).toBe(right);
        });

        it('is projected in the parent element', () => {
            expect(parent.projectedConnectors).toContain(connector);
        });

        it('has projections of end points in children of the parent element', () => {
            expect(left.projectedPins).toContain(connector.start);
            expect(right.projectedPins).toContain(connector.end);
        });
    });

    describe('Connector attached to an element and its descendant', () => {
        let parent: VisElement;
        let child: VisElement;
        let grandchild: VisElement;

        beforeEach(() => {
            parent = new VisElement(ELEMENT_TAG_NAME);
            child = new VisElement(ELEMENT_TAG_NAME);
            grandchild = new VisElement(ELEMENT_TAG_NAME);
            child.parent = parent;
            grandchild.parent = child;
            connector.start.target = parent;
            connector.end.target = grandchild;
        });

        it('has a projection in the parent element', () => {
            expect(connector.projectedParent).toBe(parent);
        });

        it('has start endpoint projected into the parent', () => {
            expect(connector.start.projectedTarget).toBe(parent);
        });

        it('has end endpoint projected into a child of the parent', () => {
            expect(connector.end.projectedTarget).toBe(child);
        });

        it('is projected in the parent element', () => {
            expect(parent.projectedConnectors).toContain(connector);
        });

        it('has a projection of its start endpoint in the parent', () => {
            expect(parent.projectedPins).toContain(connector.start);
        });

        it('has a projection of its end endpoint in the child of the parent', () => {
            expect(child.projectedPins).toContain(connector.end);
        });
    });

    describe('Connector attached to an element at both ends', () => {
        let element: VisElement;

        beforeEach(() => {
            element = new VisElement(ELEMENT_TAG_NAME);
            connector.start.target = element;
            connector.end.target = element;
        });

        it('has a projection in the element', () => {
            expect(connector.projectedParent).toBe(element);
        });

        it('has endpoints projected into the element', () => {
            expect(connector.start.projectedTarget).toBe(element);
            expect(connector.end.projectedTarget).toBe(element);
        })

        it('is projected into the element', () => {
            expect(element.projectedConnectors).toContain(connector);
        });

        it('has projections of end points in the element', () => {
            expect(element.projectedPins).toContain(connector.start);
            expect(element.projectedPins).toContain(connector.end);
        });
    });

    describe('Connector attached to unrelated elements', () => {
        let left: VisElement;
        let right: VisElement;

        beforeEach(() => {
            left = new VisElement(ELEMENT_TAG_NAME);
            right = new VisElement(ELEMENT_TAG_NAME);
            connector.start.target = left;
            connector.end.target = right;
        });

        it('has no projection', () => {
            expect(connector.projectedParent).toBeUndefined();
            expect(connector.start.projectedTarget).toBeUndefined();
            expect(connector.end.projectedTarget).toBeUndefined();
        });
    });

    describe('Projected parent change observer', () => {
        let mockObserver = jest.fn().mockName('projectedParentChangedObserver');

        beforeEach(() => {
            mockObserver.mockClear();
        });

        it('triggers when sibling elements are connected', () => {
            let parent = new VisElement(ELEMENT_TAG_NAME);
            let left = new VisElement(ELEMENT_TAG_NAME);
            let right = new VisElement(ELEMENT_TAG_NAME);
            left.parent = parent;
            right.parent = parent;
            connector.onProjectedParentChanged.hook(mockObserver);
            connector.start.target = left;
            connector.end.target = right;
            expect(mockObserver).toBeCalledWith(parent, undefined);
        });

        it('does not trigger when pin moves within the subtree', () => {
            let parent = new VisElement(ELEMENT_TAG_NAME);
            let left = new VisElement(ELEMENT_TAG_NAME);
            let right = new VisElement(ELEMENT_TAG_NAME);
            left.parent = parent;
            right.parent = parent;
            connector.start.target = left;
            connector.end.target = right;
            connector.onProjectedParentChanged.hook(mockObserver);
            connector.start.target = parent;
            expect(mockObserver).not.toBeCalled();
        });

        it('does not trigger when pin moves to a different subtree', () => {
            let parent = new VisElement(ELEMENT_TAG_NAME);
            let source = new VisElement(ELEMENT_TAG_NAME);
            let right = new VisElement(ELEMENT_TAG_NAME);
            let dest = new VisElement(ELEMENT_TAG_NAME);
            source.parent = parent;
            right.parent = parent;
            dest.parent = parent;
            connector.start.target = source;
            connector.end.target = right;
            connector.onProjectedParentChanged.hook(mockObserver);
            connector.start.target = dest;
            expect(mockObserver).not.toBeCalled();
        });

        it('triggers when a pin is detached', () => {
            let parent = new VisElement(ELEMENT_TAG_NAME);
            let left = new VisElement(ELEMENT_TAG_NAME);
            let right = new VisElement(ELEMENT_TAG_NAME);
            left.parent = parent;
            right.parent = parent;
            connector.start.target = left;
            connector.end.target = right;
            connector.onProjectedParentChanged.hook(mockObserver)
            connector.start.target = undefined;
            expect(mockObserver).toBeCalledWith(undefined, parent);
        });

        it('triggers when the subtree with a pin is detached', () => {
            let parent = new VisElement(ELEMENT_TAG_NAME);
            let left = new VisElement(ELEMENT_TAG_NAME);
            let right = new VisElement(ELEMENT_TAG_NAME);
            left.parent = parent;
            right.parent = parent;
            connector.start.target = left;
            connector.end.target = right;
            connector.onProjectedParentChanged.hook(mockObserver);
            right.parent = undefined;
            expect(mockObserver).toBeCalledWith(undefined, parent);
        });

        it('triggers after pin insertion observer', () => {
            let mockAddPinObserver = jest.fn().mockName('addPinObserver');
            let parent = new VisElement(ELEMENT_TAG_NAME);
            let left = new VisElement(ELEMENT_TAG_NAME);
            let right = new VisElement(ELEMENT_TAG_NAME);
            left.parent = parent;
            right.parent = parent;
            connector.onProjectedParentChanged.hook(mockObserver);
            right.onAddPin.hook(mockAddPinObserver);
            connector.start.target = left;
            connector.end.target = right;
            expect(mockObserver.mock.invocationCallOrder[0])
                .toBeGreaterThan(mockAddPinObserver.mock.invocationCallOrder[0]);
        });

        it('triggers after child insertion observer', () => {
            let mockAddChildObserver = jest.fn().mockName('addChildObserver');
            let parent = new VisElement(ELEMENT_TAG_NAME);
            let left = new VisElement(ELEMENT_TAG_NAME);
            let right = new VisElement(ELEMENT_TAG_NAME);
            left.parent = parent;
            connector.start.target = left;
            connector.end.target = right;
            connector.onProjectedParentChanged.hook(mockObserver);
            parent.onAddChild.hook(mockAddChildObserver);
            right.parent = parent;
            expect(mockObserver.mock.invocationCallOrder[0])
                .toBeGreaterThan(mockAddChildObserver.mock.invocationCallOrder[0]);
        });

        it('triggers after relevant properties have been updated', () => {
            mockObserver.mockImplementationOnce(() => {
                expect(parent.projectedConnectors).toContain(connector);
                expect(left.projectedPins).toContain(connector.start);
                expect(right.projectedPins).toContain(connector.end);
                expect(connector.projectedParent).toBe(parent);
                expect(connector.start.projectedTarget).toBe(left);
                expect(connector.end.projectedTarget).toBe(right);
            });
            let parent = new VisElement(ELEMENT_TAG_NAME);
            let left = new VisElement(ELEMENT_TAG_NAME);
            let right = new VisElement(ELEMENT_TAG_NAME);
            left.parent = parent;
            right.parent = parent;
            connector.onProjectedParentChanged.hook(mockObserver);
            connector.start.target = left;
            connector.end.target = right;
            expect(mockObserver).toBeCalled();
        });
    });
});

describe(VisPin, () => {
    let connector: VisConnector;

    beforeEach(() => {
        connector = new VisConnector();
    });

    describe('Target change observer', () => {
        let mockObserver = jest.fn().mockName('targetChangedObserver');
        let element: VisElement;

        beforeEach(() => {
            mockObserver.mockClear();
            element = new VisElement(ELEMENT_TAG_NAME);
        });

        it('triggers when a pin is attached to an element', () => {
            connector.start.onTargetChanged.hook(mockObserver);
            connector.start.target = element;
            expect(mockObserver).toBeCalledWith(element, undefined);
        });

        it('triggers when a pin is detached from an element', () => {
            connector.start.target = element;
            connector.start.onTargetChanged.hook(mockObserver);
            connector.start.target = undefined;
            expect(mockObserver).toBeCalledWith(undefined, element);
        });

        it('triggers when a pin is moved from one element to another', () => {
            let source = new VisElement(ELEMENT_TAG_NAME);
            connector.start.target = source;
            connector.start.onTargetChanged.hook(mockObserver);
            connector.start.target = element;
            expect(mockObserver).toBeCalledWith(element, source);
        });

        it('does not trigger when a pin is moved to its current target', () => {
            connector.start.target = element;
            connector.start.onTargetChanged.hook(mockObserver);
            connector.start.target = element;
            expect(mockObserver).not.toBeCalled();
        });

        it('does not trigger when a detached pin is removed again', () => {
            connector.start.onTargetChanged.hook(mockObserver);
            connector.start.target = undefined;
            expect(mockObserver).not.toBeCalled();
        });

        it('triggers after relevant properties have been updated', () => {
            mockObserver.mockImplementationOnce(() => {
                expect(element.pins).toContain(connector.start);
                expect(connector.start.target).toBe(element);
            });
            connector.start.onTargetChanged.hook(mockObserver);
            connector.start.target = element;
            expect(mockObserver).toBeCalled();
        });
    });

    describe('Projected target change observer', () => {
        let mockStartObserver = jest.fn().mockName('projectedTargetChangedStartObserver');
        let mockEndObserver = jest.fn().mockName('projectedTargetChangedEndObserver');

        beforeEach(() => {
            mockStartObserver.mockClear();
            mockEndObserver.mockClear();
        });

        it('triggers when sibling elements are linked', () => {
            let parent = new VisElement(ELEMENT_TAG_NAME);
            let left = new VisElement(ELEMENT_TAG_NAME);
            let right = new VisElement(ELEMENT_TAG_NAME);
            left.parent = parent;
            right.parent = parent;
            connector.start.onProjectedTargetChanged.hook(mockStartObserver);
            connector.end.onProjectedTargetChanged.hook(mockEndObserver);
            connector.start.target = left;
            connector.end.target = right;
            expect(mockStartObserver).toBeCalledWith(left, undefined);
            expect(mockEndObserver).toBeCalledWith(right, undefined);
        });

        it('does not trigger when pin moves within the subtree', () => {
            let parent = new VisElement(ELEMENT_TAG_NAME);
            let left = new VisElement(ELEMENT_TAG_NAME);
            let right = new VisElement(ELEMENT_TAG_NAME);
            let child = new VisElement(ELEMENT_TAG_NAME);
            left.parent = parent;
            right.parent = parent;
            child.parent = left;
            connector.start.target = left;
            connector.end.target = right;
            connector.start.onProjectedTargetChanged.hook(mockStartObserver);
            connector.end.onProjectedTargetChanged.hook(mockEndObserver);
            connector.start.target = child;
            expect(mockStartObserver).not.toBeCalled();
            expect(mockEndObserver).not.toBeCalled();
        });

        it('does not trigger when unrelated nodes are linked', () => {
            let left = new VisElement(ELEMENT_TAG_NAME);
            let right = new VisElement(ELEMENT_TAG_NAME);
            connector.start.onProjectedTargetChanged.hook(mockStartObserver);
            connector.end.onProjectedTargetChanged.hook(mockEndObserver);
            connector.start.target = left;
            connector.end.target = right;
            expect(mockStartObserver).not.toBeCalled();
            expect(mockEndObserver).not.toBeCalled();
        });

        it('triggers when pin moves to a different subtree', () => {
            let parent = new VisElement(ELEMENT_TAG_NAME);
            let source = new VisElement(ELEMENT_TAG_NAME);
            let right = new VisElement(ELEMENT_TAG_NAME);
            let dest = new VisElement(ELEMENT_TAG_NAME);
            source.parent = parent;
            right.parent = parent;
            dest.parent = parent;
            connector.start.target = source;
            connector.end.target = right;
            connector.start.onProjectedTargetChanged.hook(mockStartObserver);
            connector.end.onProjectedTargetChanged.hook(mockEndObserver);
            connector.start.target = dest;
            expect(mockStartObserver).toBeCalledWith(dest, source);
            expect(mockEndObserver).not.toBeCalled();
        });

        it('triggers when the subtree with the pin moves', () => {
            let parent = new VisElement(ELEMENT_TAG_NAME);
            let left = new VisElement(ELEMENT_TAG_NAME);
            let right = new VisElement(ELEMENT_TAG_NAME);
            let dest = new VisElement(ELEMENT_TAG_NAME);
            left.parent = parent;
            right.parent = parent;
            dest.parent = parent;
            connector.start.target = left;
            connector.end.target = right;
            connector.start.onProjectedTargetChanged.hook(mockStartObserver);
            connector.end.onProjectedTargetChanged.hook(mockEndObserver);
            right.parent = dest;
            expect(mockStartObserver).not.toBeCalled();
            expect(mockEndObserver).toBeCalledWith(dest, right);
        });

        it('triggers when a pin is detached', () => {
            let parent = new VisElement(ELEMENT_TAG_NAME);
            let left = new VisElement(ELEMENT_TAG_NAME);
            let right = new VisElement(ELEMENT_TAG_NAME);
            left.parent = parent;
            right.parent = parent;
            connector.start.target = left;
            connector.end.target = right;
            connector.start.onProjectedTargetChanged.hook(mockStartObserver);
            connector.end.onProjectedTargetChanged.hook(mockEndObserver);
            connector.start.target = undefined;
            expect(mockStartObserver).toBeCalledWith(undefined, left);
            expect(mockEndObserver).toBeCalledWith(undefined, right);
        });

        it('triggers when the subtree with a pin is detached', () => {
            let parent = new VisElement(ELEMENT_TAG_NAME);
            let left = new VisElement(ELEMENT_TAG_NAME);
            let right = new VisElement(ELEMENT_TAG_NAME);
            left.parent = parent;
            right.parent = parent;
            connector.start.target = left;
            connector.end.target = right;
            connector.start.onProjectedTargetChanged.hook(mockStartObserver);
            connector.end.onProjectedTargetChanged.hook(mockEndObserver);
            right.parent = undefined;
            expect(mockStartObserver).toBeCalledWith(undefined, left);
            expect(mockEndObserver).toBeCalledWith(undefined, right);
        });

        it('triggers after projected parent change observer', () => {
            let mockProjectedParentChangeObserver = jest.fn().mockName('projectedParentChangedObserver');
            let parent = new VisElement(ELEMENT_TAG_NAME);
            let left = new VisElement(ELEMENT_TAG_NAME);
            let right = new VisElement(ELEMENT_TAG_NAME);
            left.parent = parent;
            right.parent = parent;
            connector.start.onProjectedTargetChanged.hook(mockStartObserver);
            connector.start.onProjectedTargetChanged.hook(mockEndObserver);
            connector.onProjectedParentChanged.hook(mockProjectedParentChangeObserver);
            connector.start.target = left;
            connector.end.target = right;
            expect(mockStartObserver.mock.invocationCallOrder[0])
                .toBeGreaterThan(mockProjectedParentChangeObserver.mock.invocationCallOrder[0]);
            expect(mockEndObserver.mock.invocationCallOrder[0])
                .toBeGreaterThan(mockProjectedParentChangeObserver.mock.invocationCallOrder[0]);
        });

        it('triggers after relevant properties have been updated', () => {
            let expectPropertiesToBeUpdated = () => {
                expect(parent.projectedConnectors).toContain(connector);
                expect(left.projectedPins).toContain(connector.start);
                expect(right.projectedPins).toContain(connector.end);
                expect(connector.projectedParent).toBe(parent);
                expect(connector.start.projectedTarget).toBe(left);
                expect(connector.end.projectedTarget).toBe(right);
            };
            mockStartObserver.mockImplementationOnce(expectPropertiesToBeUpdated);
            mockEndObserver.mockImplementationOnce(expectPropertiesToBeUpdated);
            connector.start.onProjectedTargetChanged.hook(mockStartObserver);
            connector.end.onProjectedTargetChanged.hook(mockEndObserver);
            let parent = new VisElement(ELEMENT_TAG_NAME);
            let left = new VisElement(ELEMENT_TAG_NAME);
            let right = new VisElement(ELEMENT_TAG_NAME);
            left.parent = parent;
            right.parent = parent;
            connector.start.target = left;
            connector.end.target = right;
            expect(mockStartObserver).toBeCalled();
            expect(mockEndObserver).toBeCalled();
        });
    });
});

describe(VisStructuralException, () => {
    it('is thrown when an element is inserted into itself', () => {
        let element = new VisElement(ELEMENT_TAG_NAME);
        expect(() => element.parent = element).toThrow(VisStructuralException);
    });

    it('is thrown when an element is inserted into its descendant', () => {
        let element = new VisElement(ELEMENT_TAG_NAME);
        let child = new VisElement(ELEMENT_TAG_NAME);
        let grandchild = new VisElement(ELEMENT_TAG_NAME);
        child.parent = element;
        grandchild.parent = child;
        expect(() => element.parent = grandchild).toThrow(VisStructuralException);
    });
});
