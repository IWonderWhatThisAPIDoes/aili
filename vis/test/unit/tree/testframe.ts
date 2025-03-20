import { VisConnector, VisElement } from '../../../src/tree';
import { expect } from '@jest/globals';

export interface TestFrameRelationsSnapshot {
    readonly elementParents: (VisElement | undefined)[];
    readonly connectorProjectedParents: (VisElement | undefined)[];
    readonly startPinTargets: (VisElement | undefined)[];
    readonly endPinTargets: (VisElement | undefined)[];
    readonly startPinProjectedTargets: (VisElement | undefined)[];
    readonly endPinProjectedTargets: (VisElement | undefined)[];
}

export class TestFrame {
    constructor(elementCount: number, connectorCount: number) {
        this.elements = Array.from({ length: elementCount }, (_, i) => new VisElement(String(i)));
        this.connectors = Array.from({ length: connectorCount }, () => new VisConnector());
    }
    verifyParentsMirrorChildren(): void {
        for (const element of this.elements) {
            for (const child of element.children) {
                expect(child.parent).toBe(element);
            }
        }
    }
    verifyChildrenMirrorParents(): void {
        for (const element of this.elements) {
            if (element.parent) {
                expect(element.parent.children).toContain(element);
            }
        }
    }
    verifyTreeStructure(): void {
        const traversedElements = new Set();
        function traverseSubtree(element: VisElement) {
            traversedElements.add(element);
            for (const child of element.children) {
                traverseSubtree(child);
            }
        }
        this.elements.filter(e => !e.parent).forEach(traverseSubtree);
        const unvisitedElements = this.elements.filter(e => !traversedElements.has(e));
        expect(unvisitedElements).not.toContainEqual(expect.anything());
    }
    verifyPinsMirrorTargets(): void {
        for (const connector of this.connectors) {
            for (const pin of [connector.start, connector.end]) {
                if (pin.target) {
                    expect(pin.target.pins).toContain(pin);
                }
            }
        }
    }
    verifyTargetsMirrorPins(): void {
        for (const element of this.elements) {
            for (const pin of element.pins) {
                expect(pin.target).toBe(element);
            }
        }
    }
    verifyProjectedParentsMirrorProjections(): void {
        for (const element of this.elements) {
            for (const connector of element.projectedConnectors) {
                expect(connector.projectedParent).toBe(element);
            }
        }
    }
    verifyProjectionsMirrorProjectedParents(): void {
        for (const connector of this.connectors) {
            if (connector.projectedParent) {
                expect(connector.projectedParent.projectedConnectors).toContain(connector);
            }
        }
    }
    verifyProjectedTargetsMirrorProjectedPins(): void {
        for (const element of this.elements) {
            for (const pin of element.projectedPins) {
                expect(pin.projectedTarget).toBe(element);
            }
        }
    }
    verifyProjectedPinsMirrorProjectedTargets(): void {
        for (const connector of this.connectors) {
            for (const pin of [connector.start, connector.end]) {
                if (pin.projectedTarget) {
                    expect(pin.projectedTarget.projectedPins).toContain(pin);
                }
            }
        }
    }
    verifyProjectedPinsConsistency(): void {
        for (const connector of this.connectors) {
            if (connector.projectedParent) {
                expect(connector.start.target).toBeDefined();
                expect(connector.start.projectedTarget).toBeDefined();
                expect(connector.end.target).toBeDefined();
                expect(connector.end.projectedTarget).toBeDefined();
                if (connector.start.target === connector.projectedParent) {
                    expect(connector.start.projectedTarget).toBe(connector.projectedParent);
                } else {
                    expect(connector.start.projectedTarget?.parent).toBe(connector.projectedParent);
                }
                if (connector.end.target === connector.projectedParent) {
                    expect(connector.end.projectedTarget).toBe(connector.projectedParent);
                } else {
                    expect(connector.end.projectedTarget?.parent).toBe(connector.projectedParent);
                }
            } else {
                expect(connector.start.projectedTarget).toBeUndefined();
                expect(connector.end.projectedTarget).toBeUndefined();
            }
        }
    }
    verifyAllAttachedConnectorsProjected(): void {
        for (const connector of this.connectors) {
            if (connector.start.target && connector.end.target) {
                expect(connector.projectedParent).toBeDefined();
            }
        }
    }
    mergeIntoSingleTree(): void {
        // Get all root elements
        const roots = this.elements.filter(e => !e.parent);
        // Insert each one into the previous, so only one root remains
        // This is guaranteed to succeed because merging two separate
        // trees this way cannot create a circular link
        for (let i = 1; i < roots.length; ++i) {
            roots[i].parent = roots[i - 1];
        }
    }
    getRelationsSnapshot(): TestFrameRelationsSnapshot {
        return {
            elementParents: this.elements.map(e => e.parent),
            connectorProjectedParents: this.connectors.map(c => c.projectedParent),
            startPinTargets: this.connectors.map(c => c.start.target),
            endPinTargets: this.connectors.map(c => c.end.target),
            startPinProjectedTargets: this.connectors.map(c => c.start.projectedTarget),
            endPinProjectedTargets: this.connectors.map(c => c.end.projectedTarget),
        }
    }
    readonly elements: VisElement[];
    readonly connectors: VisConnector[];
}
