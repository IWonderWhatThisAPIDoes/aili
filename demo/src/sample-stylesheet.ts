/**
 * Pre-written stylesheets for common ways of displaying data.
 * 
 * @module
 */

/**
 * Identifiers of pre-written stylesheets that can be
 * used out of the box.
 */
export enum BuiltinStylesheet {
    /**
     * Raw view of the state graph itself.
     */
    STATE_GRAPH,
    /**
     * State graph with color-coded and style-coded
     * elements for ease of reading.
     */
    STATE_GRAPH_PRETTY,
    /**
     * Stack trace. Stack frames are nodes in a graph.
     */
    TRACE_GRAPH,
    /**
     * Stack trace. Stack frames are columns.
     */
    TRACE_COLUMN,
    /**
     * Stylesheet for a simple vector structure.
     */
    VECTOR,
    /**
     * Stylesheet for a simple linked list structure.
     */
    LINKED_LIST,
    /**
     * The default setting.
     */
    DEFAULT = STATE_GRAPH_PRETTY,
}

/**
 * Maps keys of built-in stylesheets to their display names.
 */
export const STYLESHEET_NAME: Record<BuiltinStylesheet, string> = {
    [BuiltinStylesheet.TRACE_GRAPH]: 'Stack trace (graph)',
    [BuiltinStylesheet.TRACE_COLUMN]: 'Stack trace (column)',
    [BuiltinStylesheet.STATE_GRAPH]: 'Raw view',
    [BuiltinStylesheet.STATE_GRAPH_PRETTY]: 'State graph with highlights',
    [BuiltinStylesheet.VECTOR]: 'Vector',
    [BuiltinStylesheet.LINKED_LIST]: 'Linked list',
}

/**
 * Gets a predefined stylesheet by its key.
 * 
 * @param key Identifier of the requested stylesheet.
 */
export function getSampleStylesheet(key: BuiltinStylesheet): string {
    return BUILTIN_STYLESHEETS[key];
}

/**
 * The text of the built-in stylesheets.
 */
const BUILTIN_STYLESHEETS: Record<BuiltinStylesheet, string> = {
    [BuiltinStylesheet.STATE_GRAPH_PRETTY]:
`/**
 * This stylesheet is similar to the raw view,
 * but it includes color and patterns for ease
 * of reading.
 */

// Root node will become the root element
:: {
  // Render the scene as a graph
  display: graph;
  gap: 80;
  // Keep reference to root node in a variable
  // so it can be easily recalled
  --root: @;
}

// Render the root node as a node as well
// The root node is already the physical root
// of visualization, so we must use an extra
:: ::extra {
  display: cell;
  value: "<root>";
  size: 4;
  fill: lightgrey;
}

// All nodes except root should be rendered as cells
* {
  display: cell;
  value: @ + ":" + typename(@);
  size: 1;
  fill: aliceblue;
  // Hierarchically, the parent element
  // is the root graph
  parent: --root;
}

// Make specific cells look nice
:frame {
  size: 3;
  fill: gold;
}
:ref {
  value: "&";
  fill: lightgrey;
}
:arr {
  value: "[]";
  fill: lightgrey;
}

// Show all edges as actual connectors
// This is by default, so we just need
// to select them, no display: or parent:

// The root is actually an extra though,
// so we need to attach the edge there
:: {
  --root-extra: @(::extra);
}
:: *::edge {
  parent: --root-extra;
}

// Label all edges according to their meaning
main::edge {
  label: "<main>";
}
next::edge {
  label: "<next>";
}
len::edge {
  label: "<len>";
}
%::edge {
  label: --NAME;
}
%.if(--DISCRIMINATOR)::edge {
  label: --NAME + "#" + --DISCRIMINATOR;
}
[]::edge {
  label: "[" + --INDEX + "]";
}

// Highlight the stack trace
.alt(main, next)::edge {
  stroke: gold;
  stroke-width: 3;
}

// Dereferences are dashed because
// they do not denote ownership
ref::edge {
  stroke-style: dashed;
}

::edge {
  end/decoration: arrow;
}

`,
    [BuiltinStylesheet.TRACE_GRAPH]:
`/**
 * This stylesheet displays a program as a graph
 * where each node is a stack frame or a heap allocation
 */

:: {
  display: graph;
  layout: layered;
  --root: @;
}

* {
  parent: --root;
}

.alt(%, []) {
  parent: --pred;
  key: --NAME;
  order: --INDEX;
}

{
  --pred: @;
}

:val {
  display: cell;
  value: @;
  fill: aliceblue;
}

.alt(:struct, :frame) {
  display: kvt;
  title: typename(@);
}

:arr {
  display: row;
}

.alt(:ref, :nullptr) {
  display: cell;
  size: "0.5";
  fill: maroon;
}

:ref {
    shape: circle;
}

len {
  display: label;
  value: "length: " + @;
  parent: --parent;
  vertical-justify: start;
  horizontal-justify: start;
  vertical-align: outside;
}

ref::edge {
  shape: quadratic;
  stroke: maroon;
  end/decoration: arrow;
  end/anchor: northwest;
}

next::edge {
  stroke-width: 2;
  end/decoration: arrow;
}

`,
    [BuiltinStylesheet.TRACE_COLUMN]:
`/**
 * This stylesheet displays a program as a graph
 * where each node is a heap allocation around
 * the central stack
 */

:: {
  display: graph;
  layout: layered;
  direction: east;
  --root: @;
  --trace: @(::extra(stack));
  --sp: 0;
}

:: ::extra(stack) {
  display: row;
  gap: 1;
  padding: "0.5";
  stroke-width: 1;
  direction: column;
  title: "Stack Trace";
  align-items: start;
}

:: ::extra(stack-label) {
  display: label;
  parent: --trace;
  value: "stack trace";
  vertical-justify: start;
  vertical-align: outside;
}

* {
  parent: --root;
}

:: main .many(next) {
  --sp: --sp + 1;
  order: --sp;
  parent: --trace;
  key: --sp;
}

.alt(%, []) {
  parent: --parent;
  key: --NAME;
  order: --INDEX;
}

:val {
  display: cell;
  value: @;
  fill: aliceblue;
}

.alt(:struct, :frame) {
  display: kvt;
  title: typename(@);
  --parent: @;
}

:arr {
  display: row;
  --parent: @;
}

.alt(:ref, :nullptr) {
  display: cell;
  size: "0.5";
  shape: circle;
  fill: maroon;
}

len {
  display: label;
  value: "length: " + @;
  parent: --parent;
  vertical-justify: start;
  horizontal-justify: start;
  vertical-align: outside;
}

ref::edge {
  shape: quadratic;
  stroke: maroon;
  end/decoration: arrow;
  end/anchor: northwest;
}

`,
    [BuiltinStylesheet.STATE_GRAPH]:
`/**
 * This stylesheet displays the underlying state graph
 * as plainly as possible
 *
 * It is quite long because programs were not actually
 * intended to be visualized this way
 */

// Root node will become the root element
:: {
  // Render the scene as a graph
  display: graph;
  // Keep reference to root node in a variable
  // so it can be easily recalled
  --root: @;
}

// Render the root node as a node as well
// The root node is already the physical root
// of visualization, so we must use an extra
:: ::extra {
  display: cell;
  value: "<root>";
}

// All nodes except root should be rendered as cells
* {
  display: cell;
  value: @ + ":" + typename(@);
  // Hierarchically, the parent element
  // is the root graph
  parent: --root;
}

// Make specific cells look nice
:ref {
  value: "&";
}
:arr {
  value: "[]";
}

// Show all edges as actual connectors
// This is by default, so we just need
// to select them, no display: or parent:

// The root is actually an extra though,
// so we need to attach the edge there
:: {
  --root-extra: @(::extra);
}
:: *::edge {
  parent: --root-extra;
}

// Label all edges according to their meaning
main::edge {
  label: "<main>";
}
next::edge {
  label: "<next>";
}
len::edge {
  label: "<len>";
}
%::edge {
  label: --NAME;
}
[]::edge {
  label: "[" + --INDEX + "]";
}
ref::edge {
  label: "<ref>";
}
ret::edge {
  label: "<ret>";
}

::edge {
  end/decoration: arrow;
}

`,
    [BuiltinStylesheet.VECTOR]:
`/**
 * THIS STYLESHEET IS SPECIFICALLY INTENDED
 * TO BE USED WITH THE VECTOR EXAMPLE
 *
 * Displays the vector as a row of cells
 */

// Display the root, so we can see anything in the first place
:: { display: graph; }

// Write down length and capacity of the vector,
// we will be using them later
:vector {
  --vec-len: @("len");
  --vec-cap: @("cap");
}

// Display length and capacity somewhere
:vector "len" {
  display: text;
  value: "length:" + @;
  color: maroon;
}
:vector "cap" {
  display: text;
  value: "capacity:" + @;
  color: maroon;
}

// Display the vector's main body as a row
:vector "ptr" ref {
  display: row;
}

// Display the vector's items as cells in the row
:vector "ptr" ref [] {
  display: cell;
  // Sort by index
  order: --INDEX;
  value: @;
  fill: aliceblue;
}

// Draw unused cells differently
// to make the difference glaringly obvious
:vector "ptr" ref [].if(--INDEX >= --vec-len) {
    value: "X";
    fill: lightgrey;
}

// Add index indicators to taste
:vector "ptr" ref []::extra(index) {
  display: label;
  vertical-justify: start;
  vertical-align: outside;
  value: --INDEX;
}

// Make labels where the vector's parameters point to
:vector "ptr" ref [].if(--INDEX == --vec-cap - 1)::extra(len-cap) {
  display: label;
  value: "cap";
}
:vector "ptr" ref [].if(--INDEX == --vec-len - 1)::extra(len-cap) {
  display: label;
  value: "len";
}
:vector "ptr" ref []::extra(len-cap) {
  color: maroon;
  vertical-justify: end;
  vertical-align: outside;
  horizontal-justify: end;
  horizontal-align: middle;
  hat: north;
}

`,
    [BuiltinStylesheet.LINKED_LIST]:
`/**
 * THIS STYLESHEET IS SPECIFICALLY INTENDED
 * TO BE USED WITH THE LINKED LIST EXAMPLE
 *
 * Displays the list as a graph
 */

:: {
  display: graph;
  layout: layered;
  direction: east;
  --root: @;
}

:list {
  display: cell;
  value: head;
}

:node {
  display: cell;
  shape: circle;
  value: @("value");
  parent: --root;
}

.alt(:node "next", :list "head") {
  display: connector;
  target: @(ref);
  end/decoration: arrow;
}

:: main .many(next).if(!@(next)) "turtle" {
  display: label;
  value: "turtle";
  color: "#85d";
  parent: @(ref);
  vertical-justify: end;
  vertical-align: outside;
  hat: north;
}

:: main .many(next).if(!@(next)) "hare" {
  display: label;
  value: "hare";
  color: "#4a0";
  parent: @(ref);
  vertical-justify: start;
  vertical-align: outside;
  hat: south;
}

`,
}
