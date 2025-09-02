import { afterEach, beforeEach, describe, expect, it, jest } from '@jest/globals';
import { TreeView } from '../../src/tree-view';
import { ElementView, ViewEmbedding } from '../../src/element-view';
import { ConnectorView } from '../../src/connector-view';
import { ReadonlyVisConnector, ReadonlyVisElement, VisConnector, VisElement } from '../../src/tree';
import { ViewBase, ViewContainer } from '../../src/view-container';
import { ViewModel } from '../../src/model';

const ELEMENT_TAG_NAME = 'foo';

class TestViewContainer<T extends object, R extends ViewBase> extends ViewContainer<T, R> {
    createNew(tag: T): R {
        // Find the views associated with the tag
        const views = this.expectedCalls.get(tag);
        // Remove the view, only expect it to be called once
        const view = views?.shift();
        // The requested view must be present
        if (!view) {
            throw new Error('Container has requested construction of an unexpected view');
        }
        // Drop the entry if no more views are available for it
        if (views && views.length === 0) {
            this.expectedCalls.delete(tag);
        }
        // Return the view
        return view;
    }
    /**
     * Set expected calls for the mock constructor.
     */
    expectCalls(...calls: [T, R | R[]][]): void {
        calls.forEach(([tag, view]) => {
            if (!(view instanceof Array)) {
                view = [view];
            }
            if (view.length === 0) {
                return;
            }
            let views = this.expectedCalls.get(tag);
            if (!views) {
                this.expectedCalls.set(tag, (views = []));
            }
            views.push(...view);
        });
    }
    /**
     * Clears the expected call map.
     */
    mockClear(): void {
        this.expectedCalls = new Map();
    }
    /**
     * Whether all expected calls have been made.
     */
    get allExpectedCallsReceived(): boolean {
        return this.expectedCalls.size == 0;
    }
    /**
     * Maps tags that are expected to be passed to {@link createNew}
     * to the values associated with those tags.
     *
     * Each tag is expected exactly once.
     */
    private expectedCalls: Map<T, R[]>;
}

class MockElementView implements ElementView {
    constructor(element: ReadonlyVisElement, mockName: string) {
        this.element = element;
        this.useEmbedding = jest
            .fn((em: ViewEmbedding) => (this.hasExplicitEmbedding = !!em.slot))
            .mockName(`${mockName}.useEmbedding`);
        this._destroy = jest.fn().mockName(`${mockName}._destroy`);
    }
    element: ReadonlyVisElement;
    model: ViewModel;
    hasExplicitEmbedding: boolean;
    useEmbedding: jest.Mock<(_: ViewEmbedding) => void>;
    _destroy: jest.Mock;
}

class MockConnectorView implements ConnectorView {
    constructor(connector: ReadonlyVisConnector, mockName: string) {
        this.connector = connector;
        this.useEndpoints = jest.fn().mockName(`${mockName}.useEndpoints`);
        this._destroy = jest.fn().mockName(`${mockName}._destroy`);
    }
    connector: ReadonlyVisConnector;
    useEndpoints: jest.Mock<(...args: unknown[]) => void>;
    _destroy: jest.Mock;
}

describe(TreeView, () => {
    const elementViews = new TestViewContainer<ReadonlyVisElement, ElementView>();
    const connectorViews = new TestViewContainer<ReadonlyVisConnector, ConnectorView>();
    const treeView = new TreeView(elementViews, connectorViews);
    const rootSlot = {
        populator: {
            insertFlowHtml: jest.fn().mockName('rootSlot.insertFlowHtml'),
        },
        destroy: jest.fn().mockName('rootSlot.destroy'),
    };
    let root: VisElement;
    let rootView: MockElementView;
    let conn: VisConnector;
    let connView: MockConnectorView;

    beforeEach(() => {
        elementViews.mockClear();
        connectorViews.mockClear();
        rootSlot.destroy.mockClear();
        rootSlot.populator.insertFlowHtml.mockClear();
        root = new VisElement(ELEMENT_TAG_NAME);
        rootView = new MockElementView(root, 'rootView');
        conn = new VisConnector();
        connView = new MockConnectorView(conn, 'connectorView');
    });

    afterEach(() => {
        // Root slot cannot be vacated, ever
        expect(rootSlot.destroy).not.toHaveBeenCalled();
        // All expected calls to view constructors should be made
        expect(elementViews.allExpectedCallsReceived).toBeTruthy();
        expect(connectorViews.allExpectedCallsReceived).toBeTruthy();
    });

    it('embeds a root element in the provided slot', () => {
        elementViews.expectCalls([root, rootView]);
        treeView.addRootElement(root, rootSlot);
        expect(rootView.useEmbedding).toHaveBeenCalledWith(
            expect.objectContaining({ slot: rootSlot }),
        );
    });

    it('embeds child element in its parent', () => {
        const child = new VisElement(ELEMENT_TAG_NAME);
        const childView = new MockElementView(child, 'childView');
        child.parent = root;
        elementViews.expectCalls([root, rootView], [child, childView]);
        treeView.addRootElement(root, rootSlot);
        expect(childView.useEmbedding).toHaveBeenCalledWith(
            expect.objectContaining({ parent: rootView }),
        );
    });

    it('embeds grandchild after it is added', () => {
        const child = new VisElement(ELEMENT_TAG_NAME);
        const grandchild = new VisElement(ELEMENT_TAG_NAME);
        const childView = new MockElementView(child, 'childView');
        const grandchildView = new MockElementView(grandchild, 'grandchildView');
        child.parent = root;
        elementViews.expectCalls(
            [root, rootView],
            [child, childView],
            [grandchild, grandchildView],
        );
        treeView.addRootElement(root, rootSlot);
        // Now add the grandchild (after child was given a view)
        grandchild.parent = child;
        expect(grandchildView.useEmbedding).toHaveBeenCalledWith(
            expect.objectContaining({ parent: childView }),
        );
    });

    it('embeds grandchild if it is present at initialization', () => {
        const child = new VisElement(ELEMENT_TAG_NAME);
        const grandchild = new VisElement(ELEMENT_TAG_NAME);
        const childView = new MockElementView(child, 'childView');
        const grandchildView = new MockElementView(grandchild, 'grandchildView');
        child.parent = root;
        grandchild.parent = child;
        elementViews.expectCalls(
            [root, rootView],
            [child, childView],
            [grandchild, grandchildView],
        );
        // Initialize tree view, the grandchild is already there
        treeView.addRootElement(root, rootSlot);
        expect(grandchildView.useEmbedding).toHaveBeenCalledWith(
            expect.objectContaining({ parent: childView }),
        );
    });

    it('removes whole subtree when a child is detached', () => {
        const child = new VisElement(ELEMENT_TAG_NAME);
        const grandchild = new VisElement(ELEMENT_TAG_NAME);
        const childView = new MockElementView(child, 'childView');
        const grandchildView = new MockElementView(grandchild, 'grandchildView');
        child.parent = root;
        grandchild.parent = child;
        elementViews.expectCalls(
            [root, rootView],
            [child, childView],
            [grandchild, grandchildView],
        );
        treeView.addRootElement(root, rootSlot);
        child.parent = undefined;
        expect(childView._destroy).toHaveBeenCalled();
        expect(grandchildView._destroy).toHaveBeenCalled();
    });

    it('re-attaches a removed element', () => {
        const child = new VisElement(ELEMENT_TAG_NAME);
        const childViewOne = new MockElementView(child, 'childViewOne');
        const childViewTwo = new MockElementView(child, 'childViewTwo');
        child.parent = root;
        elementViews.expectCalls([root, rootView], [child, [childViewOne, childViewTwo]]);
        treeView.addRootElement(root, rootSlot);
        child.parent = undefined;
        child.parent = root;
        expect(childViewOne._destroy).toHaveBeenCalled();
        expect(childViewTwo.useEmbedding).toHaveBeenCalledWith(
            expect.objectContaining({ parent: rootView }),
        );
    });

    it('does nothing when root element is moved to a parent', () => {
        const parent = new VisElement(ELEMENT_TAG_NAME);
        elementViews.expectCalls([root, rootView]);
        treeView.addRootElement(root, rootSlot);
        root.parent = parent;
        // Specifically, the afterEach assertions are of interest here
    });

    it('does nothing when root element is moved to its former child', () => {
        const child = new VisElement(ELEMENT_TAG_NAME);
        const childView = new MockElementView(child, 'childView');
        elementViews.expectCalls([root, rootView], [child, childView]);
        treeView.addRootElement(root, rootSlot);
        child.parent = root;
        child.parent = undefined;
        // Now move the root into the child, nothing should happen
        root.parent = child;
        // Specifically, the afterEach assertions are of interest here
    });

    it('does nothing when root element is removed from parent', () => {
        const parent = new VisElement(ELEMENT_TAG_NAME);
        elementViews.expectCalls([root, rootView]);
        root.parent = parent;
        treeView.addRootElement(root, rootSlot);
        root.parent = undefined;
        // Specifically, the afterEach assertions are of interest here
    });

    it('embeds a connector if it is present at initialization', () => {
        elementViews.expectCalls([root, rootView]);
        connectorViews.expectCalls([conn, connView]);
        conn.start.target = root;
        conn.end.target = root;
        treeView.addRootElement(root, rootSlot);
        expect(connView.useEndpoints).toHaveBeenCalledWith(rootView, rootView);
    });

    it('embeds a connector if its end is attached later', () => {
        elementViews.expectCalls([root, rootView]);
        connectorViews.expectCalls([conn, connView]);
        conn.start.target = root;
        treeView.addRootElement(root, rootSlot);
        conn.end.target = root;
        expect(connView.useEndpoints).toHaveBeenCalledWith(rootView, rootView);
    });

    it('embeds a connector if its start is attached later', () => {
        elementViews.expectCalls([root, rootView]);
        connectorViews.expectCalls([conn, connView]);
        conn.end.target = root;
        treeView.addRootElement(root, rootSlot);
        conn.start.target = root;
        expect(connView.useEndpoints).toHaveBeenCalledWith(rootView, rootView);
    });

    it('embeds a connector if it is present in inserted subtree', () => {
        const child = new VisElement(ELEMENT_TAG_NAME);
        const childView = new MockElementView(child, 'childView');
        elementViews.expectCalls([root, rootView], [child, childView]);
        connectorViews.expectCalls([conn, connView]);
        conn.start.target = child;
        conn.end.target = child;
        treeView.addRootElement(root, rootSlot);
        child.parent = root;
        expect(connView.useEndpoints).toHaveBeenCalledWith(childView, childView);
    });

    it('embeds a connector when its target is inserted', () => {
        const left = new VisElement(ELEMENT_TAG_NAME);
        const right = new VisElement(ELEMENT_TAG_NAME);
        const leftView = new MockElementView(left, 'leftView');
        const rightView = new MockElementView(right, 'rightView');
        elementViews.expectCalls([root, rootView], [left, leftView], [right, rightView]);
        left.parent = root;
        conn.start.target = left;
        conn.end.target = right;
        treeView.addRootElement(root, rootSlot);
        connectorViews.expectCalls([conn, connView]); // Only expect it now
        right.parent = root;
        expect(connView.useEndpoints).toHaveBeenCalledWith(leftView, rightView);
    });

    it('embeds a connector when its target is attached indirectly', () => {
        const child = new VisElement(ELEMENT_TAG_NAME);
        const grandchild = new VisElement(ELEMENT_TAG_NAME);
        const childView = new MockElementView(child, 'childView');
        const grandchildView = new MockElementView(grandchild, 'grandchildView');
        elementViews.expectCalls(
            [root, rootView],
            [child, childView],
            [grandchild, grandchildView],
        );
        grandchild.parent = child;
        conn.start.target = root;
        conn.end.target = grandchild;
        treeView.addRootElement(root, rootSlot);
        connectorViews.expectCalls([conn, connView]); // Only expect it now
        child.parent = root;
        expect(connView.useEndpoints).toHaveBeenCalledWith(rootView, grandchildView);
    });

    it('removes a connector when its start is detached', () => {
        conn.start.target = root;
        conn.end.target = root;
        elementViews.expectCalls([root, rootView]);
        connectorViews.expectCalls([conn, connView]);
        treeView.addRootElement(root, rootSlot);
        conn.start.target = undefined;
        expect(connView._destroy).toHaveBeenCalled();
    });

    it('removes a connector when its end is detached', () => {
        conn.start.target = root;
        conn.end.target = root;
        elementViews.expectCalls([root, rootView]);
        connectorViews.expectCalls([conn, connView]);
        treeView.addRootElement(root, rootSlot);
        conn.end.target = undefined;
        expect(connView._destroy).toHaveBeenCalled();
    });

    it('removes a connector when its target is detached indirectly', () => {
        const child = new VisElement(ELEMENT_TAG_NAME);
        const grandchild = new VisElement(ELEMENT_TAG_NAME);
        const childView = new MockElementView(child, 'childView');
        const grandchildView = new MockElementView(grandchild, 'grandchildView');
        elementViews.expectCalls(
            [root, rootView],
            [child, childView],
            [grandchild, grandchildView],
        );
        connectorViews.expectCalls([conn, connView]);
        child.parent = root;
        grandchild.parent = child;
        conn.start.target = root;
        conn.end.target = grandchild;
        treeView.addRootElement(root, rootSlot);
        child.parent = undefined;
        expect(connView._destroy).toHaveBeenCalled();
    });

    it('removes whole tree when root is detached', () => {
        const child = new VisElement(ELEMENT_TAG_NAME);
        const childView = new MockElementView(child, 'childView');
        child.parent = root;
        elementViews.expectCalls([root, rootView], [child, childView]);
        treeView.addRootElement(root, rootSlot);
        treeView.removeRootElement(root);
        expect(childView._destroy).toHaveBeenCalled();
        expect(rootView._destroy).toHaveBeenCalled();
    });

    it('embeds a connector that connects different trees', () => {
        const otherRoot = new VisElement(ELEMENT_TAG_NAME);
        const otherRootView = new MockElementView(otherRoot, 'childView');
        elementViews.expectCalls([root, rootView]);
        elementViews.expectCalls([otherRoot, otherRootView]);
        connectorViews.expectCalls([conn, connView]);
        conn.start.target = root;
        conn.end.target = otherRoot;
        treeView.addRootElement(root, rootSlot);
        treeView.addRootElement(otherRoot, rootSlot);
        expect(connView.useEndpoints).toHaveBeenCalledWith(rootView, otherRootView);
    });
});
