/**
 * Verifies that {@link TestFrame} tests fail when they are supposed to.
 *
 * @module
 */

import { beforeEach, describe, it } from '@jest/globals';
import { TestFrame } from './testframe';
import { VisConnector, VisElement, VisPin } from '../../../src/tree';

/**
 * Incomplete and writable wrapper of an object type.
 *
 * Used to construct invalid scenarios for testing.
 */
type Raw<T extends object> = { -readonly [P in keyof T]?: T[P] };

const ELEMENT_COUNT = 5;
const CONNECTOR_COUNT = 5;

describe(TestFrame, () => {
    let frame: TestFrame;

    beforeEach(() => {
        frame = new TestFrame(ELEMENT_COUNT, CONNECTOR_COUNT);
    });

    describe(TestFrame.prototype.verifyTreeStructure, () => {
        it('passes with the default tree', () => {
            frame.verifyTreeStructure();
        });

        it.failing('fails when an element is its own parent', () => {
            const invalidElement: Raw<VisElement> = {};
            invalidElement.children = new Set([invalidElement as VisElement]);
            invalidElement.parent = invalidElement as VisElement;
            frame.elements[0] = invalidElement as VisElement;
            frame.verifyTreeStructure();
        });

        it.failing('fails when an element is its own ancestor', () => {
            const invalidParent: Raw<VisElement> = {};
            const invalidChild: Raw<VisElement> = {};
            const invalidGrandchild: Raw<VisElement> = {};
            invalidChild.parent = invalidParent as VisElement;
            invalidChild.children = new Set([invalidGrandchild as VisElement]);
            invalidParent.parent = invalidGrandchild as VisElement;
            invalidParent.children = new Set([invalidChild as VisElement]);
            invalidGrandchild.parent = invalidChild as VisElement;
            invalidGrandchild.children = new Set([invalidParent as VisElement]);
            frame.elements[0] = invalidParent as VisElement;
            frame.elements[1] = invalidChild as VisElement;
            frame.elements[2] = invalidGrandchild as VisElement;
            frame.verifyTreeStructure();
        });
    });

    describe(TestFrame.prototype.verifyParentsMirrorChildren, () => {
        it('passes with the default tree', () => {
            frame.verifyParentsMirrorChildren();
        });

        it.failing('fails when an element is not the parent of its child', () => {
            const invalidParent: Raw<VisElement> = {
                children: new Set([frame.elements[1]]),
            };
            frame.elements[0] = invalidParent as VisElement;
            frame.verifyParentsMirrorChildren();
        });
    });

    describe(TestFrame.prototype.verifyChildrenMirrorParents, () => {
        it('passes with the default tree', () => {
            frame.verifyChildrenMirrorParents();
        });

        it.failing('fails when an element is not a child of its parent', () => {
            const invalidChild: Raw<VisElement> = {
                parent: frame.elements[1],
            };
            frame.elements[0] = invalidChild as VisElement;
            frame.verifyChildrenMirrorParents();
        });
    });

    describe(TestFrame.prototype.verifyTargetsMirrorPins, () => {
        it('passes with the default tree', () => {
            frame.verifyTargetsMirrorPins();
        });

        it.failing('fails when an element is not the target of its pin (start pin)', () => {
            const invalidTarget: Raw<VisElement> = {
                pins: new Set([frame.connectors[0].start]),
            };
            frame.elements[0] = invalidTarget as VisElement;
            frame.verifyTargetsMirrorPins();
        });

        it.failing('fails when an element is not the target of its pin (end pin)', () => {
            const invalidTarget: Raw<VisElement> = {
                pins: new Set([frame.connectors[0].end]),
            };
            frame.elements[0] = invalidTarget as VisElement;
            frame.verifyTargetsMirrorPins();
        });
    });

    describe(TestFrame.prototype.verifyPinsMirrorTargets, () => {
        it('passes with the default tree', () => {
            frame.verifyPinsMirrorTargets();
        });

        it.failing('fails when a pin is not a pin of its target (start pin)', () => {
            const invalidConnector: Raw<VisConnector> = {
                start: { target: frame.elements[0] } as VisPin,
                end: {} as VisPin,
            };
            frame.connectors[0] = invalidConnector as VisConnector;
            frame.verifyPinsMirrorTargets();
        });

        it.failing('fails when a pin is not a pin of its target (end pin)', () => {
            const invalidConnector: Raw<VisConnector> = {
                start: {} as VisPin,
                end: { target: frame.elements[0] } as VisPin,
            };
            frame.connectors[0] = invalidConnector as VisConnector;
            frame.verifyPinsMirrorTargets();
        });
    });

    describe(TestFrame.prototype.verifyProjectedParentsMirrorProjections, () => {
        it('passes with the default tree', () => {
            frame.verifyProjectedParentsMirrorProjections();
        });

        it.failing('fails when an element is not the parent of its projection', () => {
            const invalidElement: Raw<VisElement> = {
                projectedConnectors: new Set([frame.connectors[0]]),
            };
            frame.elements[0] = invalidElement as VisElement;
            frame.verifyProjectedParentsMirrorProjections();
        });
    });

    describe(TestFrame.prototype.verifyProjectionsMirrorProjectedParents, () => {
        it('passes with the default tree', () => {
            frame.verifyProjectionsMirrorProjectedParents();
        });

        it.failing('fails when a connector is not a projection of its projected parent', () => {
            const invalidConnector: Raw<VisConnector> = {
                projectedParent: frame.elements[0],
            };
            frame.connectors[0] = invalidConnector as VisConnector;
            frame.verifyProjectionsMirrorProjectedParents();
        });
    });
});
