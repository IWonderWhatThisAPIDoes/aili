/**
 * Built-in {@link ViewModel} types.
 * 
 * @module
 */

import { ViewModel } from '../model';
import { ViewModelConstructor, ViewModelFactory } from '../model-factory';
import { CellViewModel } from './cell';
import { CheckboxViewModel } from './checkbox';
import { FallbackViewModel } from './fallback';
import { GraphViewModel } from './graph';
import { KeyValueTableViewModel } from './kvt';
import { LabelViewModel } from './label';
import { RowViewModel } from './row';
import { TextViewModel } from './text';

export * from './cell';
export * from './checkbox';
export * from './fallback';
export * from './flow-base';
export * from './graph';
export * from './kvt';
export * from './label';
export * from './row';
export * from './text';

/**
 * Canonical tag name for a {@link CellViewModel} element.
 */
export const TAG_CELL: string = 'cell';
/**
 * Canonical tag name for a {@link CheckboxViewModel} element.
 */
export const TAG_CHECKBOX: string = 'checkbox';
/**
 * Canonical tag name for a {@link GraphViewModel} element.
 */
export const TAG_GRAPH: string = 'graph';
/**
 * Canonical tag name for a {@link KeyValueTableViewModel} element.
 */
export const TAG_KVT: string = 'kvt';
/**
 * Canonical tag name for a {@link LabelViewModel} element.
 */
export const TAG_LABEL: string = 'label';
/**
 * Canonical tag name for a {@link RowViewModel} element.
 */
export const TAG_ROW: string = 'row';
/**
 * Canonical tag name for a {@link TextViewModel} element.
 */
export const TAG_TEXT: string = 'text';

/**
 * The default {@link ViewModel}s under their canonical names.
 */
export const DEFAULT_MODELS: Map<string, ViewModelConstructor> = new Map([
    [TAG_CELL, CellViewModel],
    [TAG_CHECKBOX, CheckboxViewModel],
    [TAG_GRAPH, GraphViewModel],
    [TAG_KVT, KeyValueTableViewModel],
    [TAG_LABEL, LabelViewModel],
    [TAG_ROW, RowViewModel],
    [TAG_TEXT, TextViewModel],
]);

/**
 * The default {@link ViewModelFactory} that provides
 * {@link DEFAULT_MODELS} and {@link FallbackViewModel} as fallback.
 */
export const DEFAULT_MODEL_FACTORY: ViewModelFactory =
    new ViewModelFactory(DEFAULT_MODELS, FallbackViewModel);
