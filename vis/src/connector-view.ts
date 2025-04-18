/**
 * Rendering of {@link ReadonlyVisConnector}s.
 * 
 * @module
 */

import { ObserverHandle } from 'aili-hooligan';
import { setAttributeBindings } from './attributes';
import { ElementView } from './element-view';
import { ReadonlyVisConnector, ReadonlyVisElement, ReadonlyVisPin } from './tree';
import { ViewBase, ViewContainer } from './view-container';
import { ViewportContext } from './viewport-dom';
import * as jsplumb from '@jsplumb/browser-ui';

/**
 * Container that manages the {@link ConnectorView}s of individual {@link ReadonlyVisConnector}s.
 */
export class ConnectorViewContainer extends ViewContainer<ReadonlyVisConnector, ConnectorView> {
    /**
     * Constructs an empty view container.
     * 
     * @param context Context for construction of visuals.
     */
    constructor(context: ViewportContext) {
        super();
        this.context = context;
    }
    protected createNew(connector: ReadonlyVisConnector): ConnectorView {
        return new ConnectorViewImpl(connector, this.context);
    }
    private readonly context: ViewportContext;
}

/**
 * Rendering of a single {@link ReadonlyVisConnector}.
 */
export interface ConnectorView extends ViewBase {
    /**
     * Changes the placement of the view's endpoints. Visuals
     * will be updated as needed.
     * 
     * @param start The element view that the start pin should be attached to.
     * @param end The element view that the end pin should be attached to.
     */
    useEndpoints(start: ElementView, end: ElementView): void;
    /**
     * The connector associated with the view.
     */
    readonly connector: ReadonlyVisConnector;
}

class ConnectorViewImpl implements ConnectorView {
    /**
     * Constructs a view for a given connector.
     * 
     * @param connector The connector to be viewed.
     * @param context Context for creation of visuals.
     */
    constructor(connector: ReadonlyVisConnector, context: ViewportContext) {
        this.connector = connector;
        this.jsplumb = context.jsplumb;
        this.start = new PinView(connector.start);
        this.end = new PinView(connector.end);
    }
    useEndpoints(start: ElementView, end: ElementView): void {
        // Drop the old visual if there is one, we will recreate it
        this.model?.destroy();
        // Update endpoints
        this.start.moveToElement(start);
        this.end.moveToElement(end);
        // Update visual if possible
        if (this.start.pinHtmlContainer && this.end.pinHtmlContainer) {
            this.model = new ConnectorViewModel(
                this.connector,
                this.jsplumb,
                this.start.pinHtmlContainer,
                this.end.pinHtmlContainer
            );
        } else {
            this.model = undefined;
        }
    }
    /**
     * Cleans up after the view has expired.
     * 
     * @internal
     */
    _destroy(): void {
        // Drop the connector visual if it exists
        this.model?.destroy();
        this.model = undefined;
        // Notify both endpoint visuals
        this.start._destroy();
        this.end._destroy();
    }
    readonly connector: ReadonlyVisConnector;
    private readonly start: PinView;
    private readonly end: PinView;
    private readonly jsplumb: jsplumb.BrowserJsPlumbInstance;
    private model: ConnectorViewModel | undefined;
}

/**
 * Rendering of a single {@link ReadonlyVisPin}.
 */
class PinView {
    constructor(pin: ReadonlyVisPin) {
        this.pin = pin;
    }
    moveToElement(element: ElementView | undefined) {
        if (element.element === this.currentParent) {
            return;
        }
        this.pinContainer = element?.model?.pinContainer;
    }
    _destroy() {
        // Nothing to clean up
    }
    get pinHtmlContainer(): Element | undefined {
        return this.pinContainer;
    }
    private currentParent: ReadonlyVisElement | undefined;
    private pinContainer: Element | undefined;
    readonly pin: ReadonlyVisPin;
}

const DECORATION_OVERLAY_ID = ['back', 'front'];
const LABEL_OVERLAY_ID = 'label';
const DEFAULT_STROKE_COLOR = 'black';

class ConnectorViewModel {
    constructor(
        connector: ReadonlyVisConnector,
        jsplumb: jsplumb.BrowserJsPlumbInstance,
        source: Element,
        target: Element,
    ) {
        const connection = this.connection = jsplumb.connect({
            source,
            target,
            // We do not want connectors to be interactive... yet
            detachable: false,
            // Specify default style because it is not the same default
            // as the one JSPlumb uses
            paintStyle: {
                strokeWidth: 1,
                stroke: DEFAULT_STROKE_COLOR,
            },
            // Use attributes et al to figure out connector appearence.
            // This cannot be changed later without recreating the whole connector,
            // so we do not update it with attribute observers.
            connector: ConnectorViewModel.getJsplumbConnectorDescription(connector),
            anchor: ConnectorViewModel.PIN_ANCHOR_MAP.auto,
            endpoint: 'Blank',
        });

        this.connectorObserver = setAttributeBindings(connector.attributes, {
            stroke(value) {
                value ??= DEFAULT_STROKE_COLOR;
                connection.paintStyle.stroke = value;
                connection.endpoints[0].paintStyle.fill = value;
                connection.endpoints[1].paintStyle.fill = value;
                jsplumb.repaint(connection.endpoints[0].element);
                jsplumb.repaint(connection.endpoints[1].element);
            },
            'stroke-width'(value) {
                let numValue: number;
                if (!value || !((numValue = Number.parseFloat(value)) > 0)) {
                    numValue = 1;
                }
                connection.paintStyle.strokeWidth = numValue;
                jsplumb.repaint(connection.endpoints[0].element);
            },
            'stroke-style'(value) {
                if (value === 'dotted') {
                    connection.paintStyle.dashstyle = '1 2';
                } else if (value === 'dashed') {
                    connection.paintStyle.dashstyle = '3 3';
                } else {
                    connection.paintStyle.dashstyle = '1 0';
                }
                jsplumb.repaint(connection.endpoints[0].element);
            },
            label(value) {
                // Fetch the overlay if it exists
                const label = connection.getOverlay<jsplumb.LabelOverlay>(LABEL_OVERLAY_ID);
                // If value is set and not empty, update the text
                if (value) {
                    // Create a new label if it is not there
                    if (!label) {
                        jsplumb.addOverlay(connection, {
                            type: 'Label',
                            options: {
                                label: value,
                                id: LABEL_OVERLAY_ID
                            }
                        });
                    } else {
                        label.setLabel(value);
                    }
                } else {
                    // Drop the label if it exists
                    if (label) {
                        jsplumb.removeOverlay(connection, LABEL_OVERLAY_ID);
                    }
                }
            }
        });
        this.pinObservers = [connector.start, connector.end].map((pin, i) => {
            const endpoint = connection.endpoints[i];
            return setAttributeBindings(pin.attributes, {
                anchor(value) {
                    const anchor = ConnectorViewModel.PIN_ANCHOR_MAP[value] ?? ConnectorViewModel.PIN_ANCHOR_MAP.auto;
                    endpoint.setAnchor(anchor);
                    jsplumb.repaint(endpoint.element);
                },
                decoration(value) {
                    // Drop old overlay, if any
                    jsplumb.removeOverlay(connection, DECORATION_OVERLAY_ID[i]);
                    // Create new decoration
                    // We use both overlays and endpoint appearences to render decorations
                    switch (value) {
                        case 'square':
                            endpoint.setEndpoint('Rectangle');
                            jsplumb.repaint(endpoint.element);
                            break;
                        case 'circle':
                            endpoint.setEndpoint('Dot');
                            jsplumb.repaint(endpoint.element);
                            break;
                        case 'arrow':
                            endpoint.setEndpoint('Blank');
                            jsplumb.addOverlay(connection, {
                                type: 'Arrow',
                                options: {
                                    id: DECORATION_OVERLAY_ID[i],
                                    width: 10,
                                    length: 10,
                                    location: i * 1.0,
                                    direction: i * 2 - 1, // 1 forward, -1 backwards
                                },
                            });
                            break;
                        default:
                            endpoint.setEndpoint('Blank');
                            break;
                    }
                },
                label(value) {
                    // Fetch the overlay if it exists
                    const label = endpoint.getOverlay<jsplumb.LabelOverlay>(LABEL_OVERLAY_ID);
                    // If value is set and not empty, update the text
                    if (value) {
                        // Create a new label if it is not there
                        if (!label) {
                            jsplumb.addOverlay(endpoint, {
                                type: 'Label',
                                options: {
                                    label: value,
                                    id: LABEL_OVERLAY_ID
                                }
                            })
                        } else {
                            label.setLabel(value);
                        }
                    } else {
                        // Drop the label if it exists
                        if (label) {
                            jsplumb.removeOverlay(endpoint, LABEL_OVERLAY_ID);
                        }
                    }
                }
            });
        }) as [ObserverHandle, ObserverHandle];
    }

    private static readonly PIN_ANCHOR_MAP: Record<string, jsplumb.AnchorSpec> = {
        north: 'Top',
        south: 'Bottom',
        east: 'Right',
        west: 'Left',
        northeast: 'TopRight',
        northwest: 'TopLeft',
        southeast: 'BottomRight',
        southwest: 'BottomLeft',
        auto: { type: 'Perimeter', options: { shape: "Rectangle" } },
    };

    destroy(): void {
        // Delete the connector visual
        this.connection.instance.deleteConnection(this.connection);
        // Unhook all attribute observers
        this.connectorObserver.unhook();
        this.pinObservers[0].unhook();
        this.pinObservers[1].unhook();
    }
    private readonly connection: jsplumb.Connection;
    private readonly connectorObserver: ObserverHandle;
    private readonly pinObservers: readonly [ObserverHandle, ObserverHandle];

    private static getJsplumbConnectorDescription(connector: ReadonlyVisConnector): jsplumb.ConnectorSpec {
        switch (connector.attributes.shape.value) {
            case 'square': return 'Flowchart';
            case 'quadratic': return { type: 'StateMachine', options: { margin: 1 /* 0 is not allowed */ }};
            case 'cubic': return 'Bezier';
            default:
                // Straight connectors cannot handle connecting an element to itself
                if (connector.start.target === connector.end.target) {
                    return 'Bezier';
                } else {
                    return 'Straight';
                }
        }
    }
}
