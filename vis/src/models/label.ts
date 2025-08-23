/**
 * The companion label view model.
 *
 * @module
 */

import { setAttributeBindings } from '../attributes';
import { ViewModel, ViewLayoutMode } from '../model';
import { ReadonlyVisElement } from '../tree';
import { ViewportContext } from '../viewport-dom';
import * as bind from '../attribute-binds';
import { FlowViewModel } from './flow-base';
import './label.css';

/**
 * CSS class for a companion label element.
 */
export const CLASS_LABEL: string = 'aili-label';
/**
 * CSS class for an outer companion label wrapper.
 */
export const CLASS_LABEL_WRAPPER: string = 'aili-label-outer';
/**
 * CSS class for an intermediate companion label wrapper.
 */
export const CLASS_LABEL_SPACER: string = 'aili-label-mid';
/**
 * CSS class for positioning a label element at the top of or above its parent.
 */
export const CLASS_LABEL_NORTH: string = 'aili-north';
/**
 * CSS class for positioning a label element at the bottom of or below its parent.
 */
export const CLASS_LABEL_SOUTH: string = 'aili-south';
/**
 * CSS class for positioning a label element at or to the right of its parent.
 */
export const CLASS_LABEL_EAST: string = 'aili-east';
/**
 * CSS class for positioning a label element at or to the left of its parent.
 */
export const CLASS_LABEL_WEST: string = 'aili-west';
/**
 * CSS class for positioning a label element at the outer edge of its parent vertically.
 */
export const CLASS_LABEL_NS_OUTSIDE: string = 'aili-nsoutside';
/**
 * CSS class for positioning a label element at the outer edge of its parent horizontally.
 */
export const CLASS_LABEL_WE_OUTSIDE: string = 'aili-weoutside';
/**
 * CSS class for positioning a label element to intersect the border
 * of its parent vertically.
 */
export const CLASS_LABEL_NS_MIDDLE: string = 'aili-nsmiddle';
/**
 * CSS class for positioning a label element to intersect the border
 * of its parent horizontally.
 */
export const CLASS_LABEL_WE_MIDDLE: string = 'aili-wemiddle';
/**
 * CSS class of the SVG hat that can accompany the label.
 */
export const CLASS_LABEL_HAT: string = 'aili-label-hat';
/**
 * CSS class applied to the label wrapper to indicate that
 * the hat is above or below the label text, rather than to the side.
 */
export const CLASS_LABEL_HAT_NS: string = 'aili-label-hat-ns';

/**
 * {@link ViewModel} that represents an element as a simple
 * text label positioned near the parent element.
 * It can optionally be decorated with an arrow-like hat.
 *
 * ```text
 *   +----------+
 *   |          |
 * + - - +      |
 * -  42 - -----+
 * + - - +
 * ```
 *
 * Labels can be positioned at the center of their parent,
 * or at any of the 9 cardinal directions, inside, outside,
 * or on the edge of the parent container, for a total of 49 placements.
 *
 * Pictured are the 25 placements that do not overlap the edge.
 *
 * ```text
 * +---+----+   +---+   +----+---+
 * | A |  B |   | C |   | D  | E |
 * +---,+===================+,---+
 * | F || G |   | H |   | I || J |
 * +---||---+   +---+   +---||---+
 *     ||                   ||
 * +---||---+   +---+   +---||---+
 * | K || L |   | M |   | N || O |
 * +---||---+   +---+   +---||---+
 *     ||                   ||
 * +---||---+   +---+   +---||---+
 * | P || Q |   | R |   | S || T |
 * +---`+===================+`---+
 * | U |  V |   | W |   | X  | Y |
 * +---+----+   +---+   +----+---+
 * ```
 *
 * ## Permitted Parents
 * Any {@link ViewModel} that permits a {@link ViewLayoutMode.COMPANION} child.
 *
 * ## Permitted Children
 * Only {@link ViewLayoutMode.COMPANION} {@link ViewModel}s.
 *
 * ## Model Attributes
 * The following attributes of {@link ReadonlyVisElement.attributes}
 * affect the appearence of the visual.
 *
 * ### value
 * ```text
 * value: ''
 * ```
 * The text that will be displayed on the label.
 *
 * ### color
 * ```text
 * color: black
 * ```
 * Color of the label text.
 *
 * ### vertical-justify
 * ```text
 * vertical-justify: center
 * ```
 * Vertical position relative to parent. Permitted values are:
 * - `start`
 * - `center`
 * - `end`
 *
 * ### horizontal-justify
 * ```text
 * horizontal-justify: center
 * ```
 * Horizontal position relative to parent. Permitted values are:
 * - `start`
 * - `center`
 * - `end`
 *
 * ### vertical-align
 * ```text
 * vertical-align: inside
 * ```
 * Vertical alignment relative to parent's border. Permitted values are:
 * - `inside`
 * - `middle`
 * - `outside`
 *
 * ### horizontal-align
 * ```text
 * horizontal-align: inside
 * ```
 * Horizontal alignment relative to parent's border. Permitted values are:
 * - `inside`
 * - `middle`
 * - `outside`
 *
 * ### padding
 * ```text
 * padding: 0
 * ```
 * Padding between the label text and the edge of parent element, in em units.
 */
export class LabelViewModel extends FlowViewModel {
    constructor(element: ReadonlyVisElement, context: ViewportContext) {
        const html = context.ownerDocument.createElement('div');
        const htmlMid = context.ownerDocument.createElement('div');
        const htmlInner = context.ownerDocument.createElement('span');
        const svgMain = context.ownerDocument.createElementNS('http://www.w3.org/2000/svg', 'svg');
        const svgPath = context.ownerDocument.createElementNS('http://www.w3.org/2000/svg', 'path');
        html.className = CLASS_LABEL_WRAPPER;
        htmlMid.className = CLASS_LABEL_SPACER;
        htmlInner.className = CLASS_LABEL;
        svgMain.setAttribute('class', CLASS_LABEL_HAT);
        svgMain.setAttribute('height', '1em');
        svgMain.setAttribute('width', '0.75em');
        svgMain.setAttribute('viewBox', '0 0 3 4');
        svgPath.setAttribute('d', 'M0 0V4L3 2Z');
        svgMain.style.display = 'none';
        svgMain.append(svgPath);
        htmlMid.append(svgMain);
        htmlMid.append(htmlInner);
        html.append(htmlMid);
        super(html, htmlInner);

        this.unhookOnDestroy(
            setAttributeBindings(element.attributes, {
                value: bind.textContent(htmlInner),
                color(value) {
                    const color = value && bind.color(value);
                    if (color) {
                        svgMain.style.setProperty('fill', color);
                        htmlInner.style.setProperty('color', color);
                    } else {
                        svgMain.style.removeProperty('fill');
                        htmlInner.style.removeProperty('color');
                    }
                },
                padding: bind.css(htmlMid, 'padding', bind.numeric(bind.positiveOrZero, 'em')),
                'vertical-align'(value) {
                    switch (value) {
                        case 'outside':
                            html.classList.add(CLASS_LABEL_NS_OUTSIDE);
                            html.classList.remove(CLASS_LABEL_NS_MIDDLE);
                            break;
                        case 'middle':
                            html.classList.remove(CLASS_LABEL_NS_OUTSIDE);
                            html.classList.add(CLASS_LABEL_NS_MIDDLE);
                            break;
                        default:
                            html.classList.remove(CLASS_LABEL_NS_OUTSIDE);
                            html.classList.remove(CLASS_LABEL_NS_MIDDLE);
                            break;
                    }
                },
                'horizontal-align'(value) {
                    switch (value) {
                        case 'outside':
                            html.classList.add(CLASS_LABEL_WE_OUTSIDE);
                            html.classList.remove(CLASS_LABEL_WE_MIDDLE);
                            break;
                        case 'middle':
                            html.classList.remove(CLASS_LABEL_WE_OUTSIDE);
                            html.classList.add(CLASS_LABEL_WE_MIDDLE);
                            break;
                        default:
                            html.classList.remove(CLASS_LABEL_WE_OUTSIDE);
                            html.classList.remove(CLASS_LABEL_WE_MIDDLE);
                            break;
                    }
                },
                'vertical-justify'(value) {
                    switch (value) {
                        case 'start':
                            html.classList.add(CLASS_LABEL_NORTH);
                            html.classList.remove(CLASS_LABEL_SOUTH);
                            break;
                        case 'end':
                            html.classList.remove(CLASS_LABEL_NORTH);
                            html.classList.add(CLASS_LABEL_SOUTH);
                            break;
                        default:
                            html.classList.remove(CLASS_LABEL_NORTH);
                            html.classList.remove(CLASS_LABEL_SOUTH);
                    }
                },
                'horizontal-justify'(value) {
                    switch (value) {
                        case 'start':
                            html.classList.add(CLASS_LABEL_WEST);
                            html.classList.remove(CLASS_LABEL_EAST);
                            break;
                        case 'end':
                            html.classList.remove(CLASS_LABEL_WEST);
                            html.classList.add(CLASS_LABEL_EAST);
                            break;
                        default:
                            html.classList.remove(CLASS_LABEL_WEST);
                            html.classList.remove(CLASS_LABEL_EAST);
                    }
                },
                hat(value) {
                    switch (value) {
                        case 'north':
                            svgMain.style.removeProperty('display');
                            svgMain.style.transform = 'rotate(270deg)';
                            html.classList.add(CLASS_LABEL_HAT_NS);
                            htmlMid.insertBefore(svgMain, htmlInner);
                            break;
                        case 'south':
                            svgMain.style.removeProperty('display');
                            svgMain.style.transform = 'rotate(90deg)';
                            html.classList.add(CLASS_LABEL_HAT_NS);
                            htmlMid.insertBefore(htmlInner, svgMain);
                            break;
                        case 'east':
                            svgMain.style.removeProperty('display');
                            svgMain.style.removeProperty('transform');
                            html.classList.remove(CLASS_LABEL_HAT_NS);
                            htmlMid.insertBefore(htmlInner, svgMain);
                            break;
                        case 'west':
                            svgMain.style.removeProperty('display');
                            svgMain.style.transform = 'rotate(180deg)';
                            html.classList.remove(CLASS_LABEL_HAT_NS);
                            htmlMid.insertBefore(svgMain, htmlInner);
                            break;
                        default:
                            svgMain.style.display = 'none';
                            break;
                    }
                },
            }),
        );
    }
    preferredLayoutMode: ViewLayoutMode = ViewLayoutMode.COMPANION;
}
