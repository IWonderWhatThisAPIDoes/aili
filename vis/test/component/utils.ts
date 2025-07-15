/**
 * General utilities for in-browser testing.
 * 
 * @module
 */

import { BoundingBox, ElementHandle, EvaluateFunc, JSHandle } from 'puppeteer';
import * as vis from '../../src';
import * as path from 'path';
import 'jest-puppeteer';

/**
 * De-serialized representation of a color value.
 */
export interface ColorChannels {
    r: number;
    g: number;
    b: number;
    a: number;
}

/**
 * Opaque black.
 */
export const BLACK: ColorChannels = {
    r: 0,
    g: 0,
    b: 0,
    a: 255,
}

/**
 * Opaque white.
 */
export const WHITE: ColorChannels = {
    r: 255,
    g: 255,
    b: 255,
    a: 255,
}

/**
 * How many digits after the decimal point (in pixels) must match
 * to pass an equality assertion in em units while approximating
 * them with font size.
 */
export const EM_TOLERANCE: number = -1;

/**
 * Regex that matches a CSS RGBA color specification.
 */
const RGBA_PATTERN: RegExp = /^rgba\(((?:\s*\d+\s*,){3}\s*\d+\s*)\)$/;
/**
 * Regex that matches a CSS RGB color specification.
 */
const RGB_PATTERN: RegExp = /^rgb\(((?:\s*\d+\s*,){2}\s*\d+\s*)\)$/;
/**
 * Regex that matches a CSS pixel length.
 */
const PIXEL_PATTERN: RegExp = /^(\d+(?:\.\d+)?)px$/;

/**
 * Deserializes a CSS color in order to properly assert its value.
 * 
 * @param color CSS color specification, in `rgb` or `rgba` format.
 * @returns The same color, deserialized.
 * @throws Color does not have an expected format.
 */
export function parseResolvedColor(color: string): ColorChannels {
    color = color.trim();
    var match: string | undefined;
    if (match = RGBA_PATTERN.exec(color)?.[1]) {
        const channels = match.split(',').map(c => Number.parseInt(c.trim()));
        return {
            r: channels[0],
            g: channels[1],
            b: channels[2],
            a: channels[3],
        };
    } else if (match = RGB_PATTERN.exec(color)?.[1]) {
        const channels = match.split(',').map(c => Number.parseInt(c.trim()));
        return {
            r: channels[0],
            g: channels[1],
            b: channels[2],
            a: 255,
        };
    } else {
        throw new Error(`Could not parse computed color '${color}'.`);
    }
}

/**
 * Deserializes a CSS length in order to properly assert its value.
 * 
 * @param length CSS length specification, expected to be in pixels.
 * @returns The same length, as a number in pixels.
 * @throws {Error} `length` is not a pixel length specification.
 */
export function parsePixels(length: string): number {
    const match = PIXEL_PATTERN.exec(length)?.[1];
    if (!match) {
        throw new Error('Found value is not a pixel length');
    }
    return Number.parseFloat(match);
}

/**
 * Reusable shorthands for testing view models.
 */
export class Testbed {
    /**
     * General setup. Run this before each test case.
     */
    async beforeEach(): Promise<void> {
        // Load the page
        await page.goto('file://' + path.resolve(__dirname, 'testbed/out/index.html'));
        // Find the app container
        this.appContainer = await page.$('#app') as ElementHandle<HTMLElement>;
        // Create the root element
        this.rootElementHandle = await page.evaluateHandle(
            tagName => new vis.VisElement(tagName),
            this.rootElementTagName
        );
    }
    /**
     * Creates the viewport and starts rendering into the container.
     */
    async setupViewport(): Promise<void> {
        await page.evaluate(
            (container, root) => new vis.Viewport(container, vis.DEFAULT_MODEL_FACTORY).root = root,
            this.appContainer,
            this.rootElementHandle,
        );
    }
    /**
     * Performs arbitrary operations on the root element.
     * Because this function runs in browser context,
     * Puppeteer's execution constraints apply.
     * 
     * @param callback The callback to run in context of the testbed.
     * @param extra Additional arguments passed to the callback.
     */
    async rootElement<T extends any[], F extends EvaluateFunc<[JSHandle<vis.VisElement>, ...T]>>(callback: F, ...extra: T): Promise<void> {
        await page.evaluate(callback, this.rootElementHandle, ...extra);
    }
    /**
     * Gets the default element or any other element if a selector is provided.
     * 
     * @param selector Selector that identifies the target element.
     *                 If not provided, {@link theElementSelector} will be used.
     * @returns The element that represents the visual's rendering.
     * @throws {Error} The element does not exist.
     */
    async theElement(selector?: string): Promise<ElementHandle<Element>> {
        const elem = await this.appContainer.$(selector ?? this.theElementSelector);
        if (!elem) {
            throw new Error('Queried element does not exist');
        }
        return elem;
    }
    /**
     * Gets the bounding box of a DOM element.
     * 
     * @param selector Selector that identifies the target element.
     *                 If not provided, {@link theElementSelector} will be used.
     * @returns Bounding box of the target element.
     * @throws {Error} The element does not exist or it is not included in layout.
     */
    async boundingBox(selector?: string): Promise<BoundingBox> {
        const element = await this.theElement(selector);
        return await Testbed.boundingBoxOf(element);
    }
    /**
     * Gets the text content of a DOM element.
     * 
     * @param selector Selector that identifies the target element.
     *                 If not provided, {@link theElementSelector} will be used.
     * @returns Text content of the visual's rendering.
     */
    async textContent(selector?: string): Promise<string> {
        const element = await this.theElement(selector);
        const text = await element.getProperty('textContent');
        return await text.jsonValue() ?? '';
    }
    /**
     * Reads a CSS property of the visual's rendering element.
     * 
     * @param property Name of the property to retrieve.
     * @returns Value of the requested CSS property.
     */
    async getComputedStyle(property: string): Promise<string> {
        return await page.evaluate(
            (e, property) => getComputedStyle(e).getPropertyValue(property),
            await this.theElement(),
            property
        );
    }
    /**
     * Gets the bounding box of an element, and fails if it does not have one
     * 
     * @param handle Handle to the examined element.
     * @returns Bounding box of the element.
     * @throws {Error} The element is not included in layout.
     */
    static async boundingBoxOf(handle: ElementHandle): Promise<BoundingBox> {
        const bb = await handle.boundingBox();
        if (!bb) {
            throw new Error('Requested bounding box of an element that is not included in layout');
        }
        return bb;
    }
    /**
     * The element that is the designated container for the app.
     */
    appContainer: ElementHandle<HTMLElement>;
    /**
     * Root of the visualization tree.
     */
    rootElementHandle: JSHandle<vis.VisElement>;
    /**
     * Tag name for the root element. Set this before calling {@link setupViewport}.
     */
    rootElementTagName: string;
    /**
     * Selector that should match the element's rendering..
     * Set this before accessing the rendering.
     */
    theElementSelector: string;
}
