/**
 * Visualization tree invariant testing with large test cases
 * generated using fuzzer.
 * 
 * @module
 */

import { Hookable } from 'aili-hooligan';
import { create as createRng } from 'random-seed';
import { describe, expect, it, jest } from '@jest/globals';
import { executeInstructionFallible, InstructionGenerator } from './instruction';
import { TestFrame } from './testframe';

const rand = createRng('42');

const ELEMENT_COUNT = 30;
const CONNECTOR_COUNT = 30;
const TEST_CASE_COUNT = 20;
const OPERATION_COUNT = 500;
const instructionGenerator = new InstructionGenerator(ELEMENT_COUNT, CONNECTOR_COUNT, {
    insert: 4,
    remove: 1,
    connect: 4,
    detach: 1,
});
const uniformInstructionGenerator = new InstructionGenerator(ELEMENT_COUNT, CONNECTOR_COUNT, {
    insert: 1,
    remove: 1,
    connect: 1,
    detach: 1,
});

for (let i = 0; i < TEST_CASE_COUNT; ++i) {
    const frame = new TestFrame(ELEMENT_COUNT, CONNECTOR_COUNT);
    const instructions = Array.from({ length: OPERATION_COUNT }, () => instructionGenerator.generate(rand));

    describe(
        `Frame with ${ELEMENT_COUNT} elements, ${CONNECTOR_COUNT} connectors; `+
        `operations: {\n\t${instructions.map(i => i.describe()).join('\n\t')}\n}`,
        () => {
            instructions.forEach(i => executeInstructionFallible(i, frame));
            frame.mergeIntoSingleTree();
        
            it('has consistent parent-child relationships', () => {
                frame.verifyChildrenMirrorParents();
                frame.verifyParentsMirrorChildren();
            });

            it('has consistent pin-target relationships', () => {
                frame.verifyPinsMirrorTargets();
                frame.verifyTargetsMirrorPins();
            });

            it('has a tree structure', () => {
                frame.verifyTreeStructure();
            });

            it('has consistent projection-parent relationships', () => {
                frame.verifyProjectedParentsMirrorProjections();
                frame.verifyProjectionsMirrorProjectedParents();
            });

            it('has consistent projected pin-target relationships', () => {
                frame.verifyProjectedTargetsMirrorProjectedPins();
                frame.verifyProjectedPinsMirrorProjectedTargets();
            });

            it('has consistent projection-target relationships', () => {
                frame.verifyProjectedPinsConsistency();
            });

            it('has no fully attached connectors witout a projection', () => {
                frame.verifyAllAttachedConnectorsProjected();
            });
        }
    );
}

function mockObservers<T>(targets: T[], name: string, getHook: (target: T) => Hookable): (() => void)[] {
    return targets.map((target, i) => {
        const mockObserver = jest.fn().mockName(`${name}Observer${i}`);
        getHook(target).hook(mockObserver);
        return mockObserver
    });
}

const TEST_OPERATION_COUNT = 50;

for (let i = 0; i < TEST_OPERATION_COUNT; ++i) {
    const frame = new TestFrame(ELEMENT_COUNT, CONNECTOR_COUNT);
    const instructions = Array.from({ length: OPERATION_COUNT }, () => instructionGenerator.generate(rand));
    instructions.forEach(i => executeInstructionFallible(i, frame));
    const snapshot = frame.getRelationsSnapshot();

    const mockAddChildObservers = mockObservers(frame.elements, 'addChild', e => e.onAddChild);
    const mockParentChangedObservers = mockObservers(frame.elements, 'parentChanged', e => e.onParentChanged);
    const mockStartTargetChangedObservers = mockObservers(frame.connectors, 'startTargetChanged', c => c.start.onTargetChanged);
    const mockEndTargetChangedObservers = mockObservers(frame.connectors, 'endTargetChanged', c => c.end.onTargetChanged);
    const mockAddPinObservers = mockObservers(frame.elements, 'addPin', e => e.onAddPin);
    const mockProjectedParentChangedObservers = mockObservers(frame.connectors, 'projectedParentChanged', c => c.onProjectedParentChanged);
    const mockAddProjectedConnectorObservers = mockObservers(frame.elements, 'addProjectedConnector', e => e.onAddProjectedConnector);
    const mockStartProjectedTargetChangedObservers = mockObservers(frame.connectors, 'startProjectedTargetChanged', c => c.start.onProjectedTargetChanged);
    const mockEndProjectedTargetChangedObservers = mockObservers(frame.connectors, 'endProjectedTargetChanged', c => c.end.onProjectedTargetChanged);
    const mockAddProjectedPinObservers = mockObservers(frame.elements, 'addProjectedPin', e => e.onAddProjectedPin);

    const newInstruction = uniformInstructionGenerator.generate(rand);
    executeInstructionFallible(newInstruction, frame);

    describe(
        `Frame with ${ELEMENT_COUNT} elements, ${CONNECTOR_COUNT} connectors; ` +
        `operations: {\n\t${instructions.map(i => i.describe()).join('\n\t')}\n} ` +
        `after additional operation: ${newInstruction.describe()}`,
        () => {
            it('triggers parent-child observers correctly', () => {
                for (let i = 0; i < ELEMENT_COUNT; ++i) {
                    const parent = frame.elements[i].parent;
                    const previousParent = snapshot.elementParents[i];
                    if (parent !== previousParent) {
                        expect(mockParentChangedObservers[i]).toBeCalledWith(parent, previousParent);
                        if (parent) {
                            expect(mockAddChildObservers[frame.elements.indexOf(parent)])
                                .toBeCalledWith(frame.elements[i]);
                        }
                    } else {
                        expect(mockParentChangedObservers[i]).not.toBeCalled();
                    }
                }
            });

            it('triggers starting pin-target observers correctly', () => {
                for (let i = 0; i < CONNECTOR_COUNT; ++i) {
                    const startTarget = frame.connectors[i].start.target;
                    const previousStartTarget = snapshot.startPinTargets[i];
                    if (startTarget !== previousStartTarget) {
                        expect(mockStartTargetChangedObservers[i]).toBeCalledWith(startTarget, previousStartTarget);
                        if (startTarget) {
                            expect(mockAddPinObservers[frame.elements.indexOf(startTarget)])
                                .toBeCalledWith(frame.connectors[i].start);
                        }
                    } else {
                        expect(mockStartTargetChangedObservers[i]).not.toBeCalled();
                    }
                }
            });

            it('triggers ending pin-target observers correctly', () => {
                for (let i = 0; i < CONNECTOR_COUNT; ++i) {
                    const endTarget = frame.connectors[i].end.target;
                    const previousEndTarget = snapshot.endPinTargets[i];
                    if (endTarget !== previousEndTarget) {
                        expect(mockEndTargetChangedObservers[i]).toBeCalledWith(endTarget, previousEndTarget);
                        if (endTarget) {
                            expect(mockAddPinObservers[frame.elements.indexOf(endTarget)])
                                .toBeCalledWith(frame.connectors[i].end);
                        }
                    } else {
                        expect(mockEndTargetChangedObservers[i]).not.toBeCalled();
                    }
                }
            });

            it('triggers projection-projected parent observers correctly', () => {
                for (let i = 0; i < CONNECTOR_COUNT; ++i) {
                    const projectedParent = frame.connectors[i].projectedParent;
                    const previousProjectedParent = snapshot.connectorProjectedParents[i];
                    if (projectedParent !== previousProjectedParent) {
                        expect(mockProjectedParentChangedObservers[i]).toBeCalledWith(projectedParent, previousProjectedParent);
                        if (projectedParent) {
                            expect(mockAddProjectedConnectorObservers[frame.elements.indexOf(projectedParent)])
                                .toBeCalledWith(frame.connectors[i]);
                        }
                    } else {
                        expect(mockProjectedParentChangedObservers[i]).not.toBeCalled();
                    }
                }
            });

            it('triggers starting pin-projection observers correctly', () => {
                for (let i = 0; i < CONNECTOR_COUNT; ++i) {
                    const startTarget = frame.connectors[i].start.projectedTarget;
                    const previousStartTarget = snapshot.startPinProjectedTargets[i];
                    if (startTarget !== previousStartTarget) {
                        expect(mockStartProjectedTargetChangedObservers[i]).toBeCalledWith(startTarget, previousStartTarget);
                        if (startTarget) {
                            expect(mockAddProjectedPinObservers[frame.elements.indexOf(startTarget)])
                                .toBeCalledWith(frame.connectors[i].start)
                        }
                    } else {
                        expect(mockStartProjectedTargetChangedObservers[i]).not.toBeCalled();
                    }
                }
            });

            it('triggers ending pin-projection observers correctly', () => {
                for (let i = 0; i < CONNECTOR_COUNT; ++i) {
                    const endTarget = frame.connectors[i].end.projectedTarget;
                    const previousEndTarget = snapshot.endPinProjectedTargets[i];
                    if (endTarget !== previousEndTarget) {
                        expect(mockEndProjectedTargetChangedObservers[i]).toBeCalledWith(endTarget, previousEndTarget);
                        if (endTarget) {
                            expect(mockAddProjectedPinObservers[frame.elements.indexOf(endTarget)])
                                .toBeCalledWith(frame.connectors[i].end)
                        }
                    } else {
                        expect(mockEndProjectedTargetChangedObservers[i]).not.toBeCalled();
                    }
                }
            });
        }
    );
}
