import { RandomSeed } from 'random-seed';
import { TestFrame } from './testframe';
import { VisStructuralException } from '../../../src/tree';

export interface Instruction {
    execute(frame: TestFrame): void;
    describe(): string;
}

export function executeInstructionFallible(instruction: Instruction, frame: TestFrame): void {
    try {
        instruction.execute(frame);
    } catch (e) {
        if (!(e instanceof VisStructuralException)) {
            // Rethrow, only structural exceptions can be ignored
            throw e;
        }
    }
}

export class InsertInstruction implements Instruction {
    constructor(parentIndex: number, childIndex: number) {
        this.parentIndex = parentIndex;
        this.childIndex = childIndex;
    }
    execute(frame: TestFrame): void {
        frame.elements[this.childIndex].parent = frame.elements[this.parentIndex];
    }
    describe(): string {
        return `insert element ${this.childIndex} into ${this.parentIndex}`;
    }
    readonly parentIndex: number;
    readonly childIndex: number;
}

export class RemoveInstruction implements Instruction {
    constructor(elementIndex: number) {
        this.elementIndex = elementIndex;
    }
    execute(frame: TestFrame): void {
        frame.elements[this.elementIndex].parent = undefined;
    }
    describe(): string {
        return `remove element ${this.elementIndex} from its parent`;
    }
    readonly elementIndex: number;
}

export class ConnectInstruction implements Instruction {
    constructor(connectorIndex: number, pinSide: 'start' | 'end', targetIndex: number) {
        this.connectorIndex = connectorIndex;
        this.pinSide = pinSide;
        this.targetIndex = targetIndex;
    }
    execute(frame: TestFrame): void {
        frame.connectors[this.connectorIndex][this.pinSide].target =
            frame.elements[this.targetIndex];
    }
    describe(): string {
        return `connect ${this.pinSide} of connector ${this.connectorIndex} to element ${this.targetIndex}`;
    }
    readonly connectorIndex: number;
    readonly pinSide: 'start' | 'end';
    readonly targetIndex: number;
}

export class DetachInstruction implements Instruction {
    constructor(connectorIndex: number, pinSide: 'start' | 'end') {
        this.connectorIndex = connectorIndex;
        this.pinSide = pinSide;
    }
    execute(frame: TestFrame): void {
        frame.connectors[this.connectorIndex][this.pinSide].target = undefined;
    }
    describe(): string {
        return `detach ${this.pinSide} of connector ${this.connectorIndex}`;
    }
    readonly connectorIndex: number;
    readonly pinSide: 'start' | 'end';
}

export interface InstructionWeights {
    insert: number;
    remove: number;
    connect: number;
    detach: number;
}

export class InstructionGenerator {
    constructor(elementCount: number, connectorCount: number, weights: InstructionWeights) {
        this.instructionTypePicker = new WeightedRandomSet<(rand: RandomSeed) => Instruction>([
            [
                rand =>
                    new InsertInstruction(
                        rand.intBetween(0, elementCount - 1),
                        rand.intBetween(0, elementCount - 1),
                    ),
                weights.insert,
            ],
            [rand => new RemoveInstruction(rand.intBetween(0, elementCount - 1)), weights.remove],
            [
                rand =>
                    new ConnectInstruction(
                        rand.intBetween(0, connectorCount - 1),
                        rand.random() > 0.5 ? 'start' : 'end',
                        rand.intBetween(0, elementCount - 1),
                    ),
                weights.connect,
            ],
            [
                rand =>
                    new DetachInstruction(
                        rand.intBetween(0, connectorCount - 1),
                        rand.random() > 0.5 ? 'start' : 'end',
                    ),
                weights.detach,
            ],
        ]);
    }
    generate(rand: RandomSeed): Instruction {
        return this.instructionTypePicker.generate(rand)(rand);
    }
    private readonly instructionTypePicker: WeightedRandomSet<(rand: RandomSeed) => Instruction>;
}

class WeightedRandomSet<T> {
    constructor(options: [T, number][]) {
        const weightSum = options.map(([_, w]) => w).reduce((a, b) => a + b, 0);
        let cummulative = 0;
        this.distribution = options.map(([value, weight]) => ({
            value,
            cummulative: (cummulative += weight / weightSum),
        }));
    }
    generate(rand: RandomSeed): T {
        const selector = rand.random();
        return this.distribution.find(({ cummulative }) => cummulative > selector)?.value as T;
    }
    private readonly distribution: { value: T; cummulative: number }[];
}
