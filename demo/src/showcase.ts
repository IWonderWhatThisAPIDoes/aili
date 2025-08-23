/**
 * The main logic of presenting renderings to the user.
 *
 * @module
 */

import { Logger, Severity } from 'aili-hooligan';
import { DEFAULT_MODEL_FACTORY, Viewport, VisConnector, VisElement } from 'aili-vis';
import { applyStylesheet, StateGraph, Stylesheet, VisTreeRenderer } from 'aili-jsapi';
import { createSampleGraph, SampleGraph } from './sample-graph';
import { prettyPrintVisTree } from './pretty-vis';

/**
 * Encapsulates the logic of showcasing visualizations.
 */
export class Showcase {
    constructor(viewportPanel: HTMLElement, logger: Logger) {
        const viewport = new Viewport(viewportPanel, DEFAULT_MODEL_FACTORY);
        this.renderer = new VisTreeRenderer({
            createElement: (tagName: string) => new VisElement(tagName),
            createConnector: () => new VisConnector(),
            set root(root: VisElement) {
                viewport.root = root;
            },
        });
        this.logger = logger;
        this.viewport = viewport;
        this.stateGraph = createSampleGraph(SampleGraph.DEFAULT);
        this.renderer.logger = {
            log(severity, message) {
                logger.log(severity as number, message);
            },
        };
    }
    stylesheetChanged(stylesheetSource: string): void {
        let stylesheet: Stylesheet;
        try {
            stylesheet = Stylesheet.parse(stylesheetSource, (e: Error) =>
                this.logger.log(Severity.WARNING, e.message),
            );
        } catch (e) {
            this.logger.log(Severity.ERROR, 'Stylesheet failed to parse: ' + e.message);
            return;
        }
        applyStylesheet(stylesheet, this.stateGraph, this.renderer);
    }
    useStateGraph(graph: StateGraph): void {
        this.stateGraph = graph;
    }
    prettyPrintTree(): string {
        if (this.viewport.root) {
            return prettyPrintVisTree(this.viewport.root);
        } else {
            return '[empty tree]';
        }
    }
    prettyPrintResolvedStyle(): string {
        return this.renderer.prettyPrint().replace(/ /g, '\xa0');
    }
    private logger: Logger;
    private viewport: Viewport;
    private renderer: VisTreeRenderer;
    private stateGraph: StateGraph;
}
