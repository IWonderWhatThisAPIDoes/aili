/**
 * Verifies that {@link TestFrame} tests fail when they are supposed to.
 * 
 * @module
 */

import { beforeEach, describe, it } from "@jest/globals";
import { TestFrame } from "./testframe";

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
            const invalidElement: any = {};
            invalidElement.children = [invalidElement];
            invalidElement.parent = invalidElement;
            frame.elements[0] = invalidElement;
            frame.verifyTreeStructure();
        });

        it.failing('fails when an element is its own ancestor', () => {
            const invalidParent: any = {};
            const invalidChild: any = {};
            const invalidGrandchild: any = {};
            invalidChild.parent = invalidParent;
            invalidChild.children = [invalidGrandchild];
            invalidParent.parent = invalidGrandchild;
            invalidParent.children = [invalidChild];
            invalidGrandchild.parent = invalidChild;
            invalidGrandchild.children = [invalidParent];
            frame.elements[0] = invalidParent;
            frame.elements[1] = invalidChild;
            frame.elements[2] = invalidGrandchild;
            frame.verifyTreeStructure();
        });
    });

    describe(TestFrame.prototype.verifyParentsMirrorChildren, () => {
        it('passes with the default tree', () => {
            frame.verifyParentsMirrorChildren();
        });

        it.failing('fails when an element is not the parent of its child', () => {
            const invalidParent: any = {
                children: [frame.elements[1]]
            };
            frame.elements[0] = invalidParent;
            frame.verifyParentsMirrorChildren();
        });
    });

    describe(TestFrame.prototype.verifyChildrenMirrorParents, () => {
        it('passes with the default tree', () => {
            frame.verifyChildrenMirrorParents();
        });

        it.failing('fails when an element is not a child of its parent', () => {
            const invalidChild: any = {
                parent: frame.elements[1]
            };
            frame.elements[0] = invalidChild;
            frame.verifyChildrenMirrorParents();
        });
    });

    describe(TestFrame.prototype.verifyTargetsMirrorPins, () => {
        it('passes with the default tree', () => {
            frame.verifyTargetsMirrorPins();
        });

        it.failing('fails when an element is not the target of its pin (start pin)', () => {
            const invalidTarget: any = {
                pins: [frame.connectors[0].start]
            };
            frame.elements[0] = invalidTarget;
            frame.verifyTargetsMirrorPins();
        });

        it.failing('fails when an element is not the target of its pin (end pin)', () => {
            const invalidTarget: any = {
                pins: [frame.connectors[0].end]
            };
            frame.elements[0] = invalidTarget;
            frame.verifyTargetsMirrorPins();
        });
    });

    describe(TestFrame.prototype.verifyPinsMirrorTargets, () => {
        it('passes with the default tree', () => {
            frame.verifyPinsMirrorTargets();
        });

        it.failing('fails when a pin is not a pin of its target (start pin)', () => {
            const invalidConnector: any = {
                start: { target: frame.elements[0] },
                end: {}
            };
            frame.connectors[0] = invalidConnector;
            frame.verifyPinsMirrorTargets();
        });

        it.failing('fails when a pin is not a pin of its target (end pin)', () => {
            const invalidConnector: any = {
                start: {},
                end: { target: frame.elements[0] }
            };
            frame.connectors[0] = invalidConnector;
            frame.verifyPinsMirrorTargets();
        });
    });

    describe(TestFrame.prototype.verifyProjectedParentsMirrorProjections, () => {
        it('passes with the default tree', () => {
            frame.verifyProjectedParentsMirrorProjections();
        });

        it.failing('fails when an element is not the parent of its projection', () => {
            const invalidElement: any = {
                projectedConnectors: [frame.connectors[0]]
            };
            frame.elements[0] = invalidElement;
            frame.verifyProjectedParentsMirrorProjections();
        });
    });

    describe(TestFrame.prototype.verifyProjectionsMirrorProjectedParents, () => {
        it('passes with the default tree', () => {
            frame.verifyProjectionsMirrorProjectedParents();
        });

        it.failing('fails when a connector is not a projection of its projected parent', () => {
            const invalidConnector: any = {
                projectedParent: frame.elements[0],
            };
            frame.connectors[0] = invalidConnector;
            frame.verifyProjectionsMirrorProjectedParents();
        });
    });
});
