/**
 * Pre-written stylesheets for common ways of displaying data.
 * 
 * @module
 */

/**
 * Displays the stack trace as a graph.
 */
export const TRACE_STYLESHEET: string =
`:: {
\xa0\xa0display: graph;
\xa0\xa0--root: @;
}

* {
\xa0\xa0parent: --root;
}

.alt(%, []) {
\xa0\xa0parent: --parent;
\xa0\xa0key: --NAME;
\xa0\xa0order: --INDEX;
}

:val {
\xa0\xa0display: cell;
\xa0\xa0value: @;
\xa0\xa0fill: aliceblue;
}

.alt(:struct, :frame) {
\xa0\xa0display: kvt;
\xa0\xa0title: typename(@);
\xa0\xa0--parent: @;
}

:arr {
\xa0\xa0display: row;
\xa0\xa0--parent: @;
}

:ref {
\xa0\xa0display: cell;
\xa0\xa0size: "0.5";
\xa0\xa0shape: circle;
\xa0\xa0fill: maroon;
}

len {
\xa0\xa0display: label;
\xa0\xa0value: "length: " + @;
\xa0\xa0parent: --parent;
\xa0\xa0vertical-justify: start;
\xa0\xa0horizontal-justify: start;
\xa0\xa0vertical-align: outside;
}

ref::edge {
\xa0\xa0stroke-style: dashed;
\xa0\xa0shape: cubic;
\xa0\xa0end/decoration: arrow;
}

next::edge {
\xa0\xa0stroke-width: 2;
\xa0\xa0end/decoration: arrow;
}

`;

export const COLUMN_TRACE_STYLESHEET: string =
`:: {
\xa0\xa0display: graph;
\xa0\xa0--root: @;
\xa0\xa0--sp: 0;
\xa0\xa0--trace: @(::extra(stack));
}

:: ::extra(stack) {
\xa0\xa0display: row;
\xa0\xa0gap: 1;
\xa0\xa0padding: "0.5";
\xa0\xa0stroke-width: 1;
\xa0\xa0direction: column;
\xa0\xa0title: "Stack Trace";
\xa0\xa0align-items: start;
}

:: ::extra(stack-label) {
\xa0\xa0display: label;
\xa0\xa0parent: --trace;
\xa0\xa0value: "stack trace";
\xa0\xa0vertical-justify: start;
\xa0\xa0vertical-align: outside;
}

* {
\xa0\xa0parent: --root;
}

:: main .many(next) {
\xa0\xa0--sp: --sp + 1;
\xa0\xa0order: --sp;
\xa0\xa0parent: --trace;
\xa0\xa0key: --sp;
}

.alt(%, []) {
\xa0\xa0parent: --parent;
\xa0\xa0key: --NAME;
\xa0\xa0order: --INDEX;
}

:val {
\xa0\xa0display: cell;
\xa0\xa0value: @;
\xa0\xa0fill: aliceblue;
}

.alt(:struct, :frame) {
\xa0\xa0display: kvt;
\xa0\xa0title: typename(@);
\xa0\xa0--parent: @;
}

:arr {
\xa0\xa0display: row;
\xa0\xa0--parent: @;
}

:ref {
\xa0\xa0display: cell;
\xa0\xa0size: "0.5";
\xa0\xa0shape: circle;
\xa0\xa0fill: maroon;
}

len {
\xa0\xa0display: label;
\xa0\xa0value: "length: " + @;
\xa0\xa0parent: --parent;
\xa0\xa0vertical-justify: start;
\xa0\xa0horizontal-justify: start;
\xa0\xa0vertical-align: outside;
}

ref::edge {
\xa0\xa0shape: quadratic;
\xa0\xa0stroke: maroon;
\xa0\xa0end/decoration: arrow;
}

`;

/**
 * Displays the state graph as plainly as possible.
 */
export const RAW_STYLESHEET: string =
`// Root node will become the root element
:: {
\xa0\xa0// Render the scene as a graph
\xa0\xa0display: graph;
\xa0\xa0// Keep reference to root node in a variable
\xa0\xa0// so it can be easily recalled
\xa0\xa0--root: @;
}

// Render the root node as a node as well
// The root node is already the physical root
// of visualization, so we must use an extra
:: ::extra {
\xa0\xa0display: cell;
\xa0\xa0value: "<root>";
}

// All nodes except root should be rendered as cells
* {
\xa0\xa0display: cell;
\xa0\xa0value: @ + ":" + typename(@);
\xa0\xa0// Hierarchically, the parent element
\xa0\xa0// is the root graph
\xa0\xa0parent: --root;
}

// Make specific cells look nice
:ref {
\xa0\xa0value: "&";
}
:arr {
\xa0\xa0value: "[]";
}

// Show all edges as actual connectors
// This is by default, so we just need
// to select them, no display: or parent:

// The root is actually an extra though,
// so we need to attach the edge there
:: {
\xa0\xa0--root-extra: @(::extra);
}
:: *::edge {
\xa0\xa0parent: --root-extra;
}

// Label all edges according to their meaning
main::edge {
\xa0\xa0label: "<main>";
}
next::edge {
\xa0\xa0label: "<next>";
}
len::edge {
\xa0\xa0label: "<len>";
}
%::edge {
\xa0\xa0label: --NAME;
}
[]::edge {
\xa0\xa0label: "[" + --INDEX + "]";
}
ref::edge {
\xa0\xa0label: "<ref>";
}
ret::edge {
\xa0\xa0label: "<ret>";
}

::edge {
\xa0\xa0end/decoration: arrow;
}

`;

/**
 * Displays the state graph with colors and styles to make it
 * easier to navigate.
 */
export const PRETTY_GRAPH_STYLESHEET: string =
`// Root node will become the root element
:: {
\xa0\xa0// Render the scene as a graph
\xa0\xa0display: graph;
\xa0\xa0gap: 80;
\xa0\xa0// Keep reference to root node in a variable
\xa0\xa0// so it can be easily recalled
\xa0\xa0--root: @;
}

// Render the root node as a node as well
// The root node is already the physical root
// of visualization, so we must use an extra
:: ::extra {
\xa0\xa0display: cell;
\xa0\xa0value: "<root>";
\xa0\xa0size: 4;
\xa0\xa0fill: lightgrey;
}

// All nodes except root should be rendered as cells
* {
\xa0\xa0display: cell;
\xa0\xa0value: @ + ":" + typename(@);
\xa0\xa0size: 1;
\xa0\xa0fill: aliceblue;
\xa0\xa0// Hierarchically, the parent element
\xa0\xa0// is the root graph
\xa0\xa0parent: --root;
}

// Make specific cells look nice
:frame {
\xa0\xa0size: 3;
\xa0\xa0fill: gold;
}
:ref {
\xa0\xa0value: "&";
\xa0\xa0fill: lightgrey;
}
:arr {
\xa0\xa0value: "[]";
\xa0\xa0fill: lightgrey;
}

// Show all edges as actual connectors
// This is by default, so we just need
// to select them, no display: or parent:

// The root is actually an extra though,
// so we need to attach the edge there
:: {
\xa0\xa0--root-extra: @(::extra);
}
:: *::edge {
\xa0\xa0parent: --root-extra;
}

// Label all edges according to their meaning
main::edge {
\xa0\xa0label: "<main>";
}
next::edge {
\xa0\xa0label: "<next>";
}
len::edge {
\xa0\xa0label: "<len>";
}
%::edge {
\xa0\xa0label: --NAME;
}
%.if(--DISCRIMINATOR)::edge {
\xa0\xa0label: --NAME + "#" + --DISCRIMINATOR;
}
[]::edge {
\xa0\xa0label: "[" + --INDEX + "]";
}

// Highlight the stack trace
.alt(main, next)::edge {
\xa0\xa0stroke: gold;
\xa0\xa0stroke-width: 3;
}

// Dereferences are dashed because
// they do not denote ownership
ref::edge {
\xa0\xa0stroke-style: dashed;
}

::edge {
\xa0\xa0end/decoration: arrow;
}

`;

export const VECTOR_STYLESHEET: string =
`// Display the root, so we can see anything in the first place
:: { display: graph; }

// Write down length and capacity of the vector,
// we will be using them later
:vector {
\xa0\xa0--vec-len: @("len");
\xa0\xa0--vec-cap: @("cap");
}

// Display length and capacity somewhere
:vector "len" {
\xa0\xa0display: text;
\xa0\xa0value: "length:" + @;
\xa0\xa0color: maroon;
}
:vector "cap" {
\xa0\xa0display: text;
\xa0\xa0value: "capacity:" + @;
\xa0\xa0color: maroon;
}

// Display the vector's main body as a row
:vector "ptr" ref {
\xa0\xa0display: row;
}

// Display the vector's items as cells in the row
:vector "ptr" ref [] {
\xa0\xa0display: cell;
\xa0\xa0// Sort by index
\xa0\xa0order: --INDEX;
\xa0\xa0// Is the memory cell actually out of range of the vector?
\xa0\xa0--out-of-range: --INDEX >= --vec-len;
\xa0\xa0// Draw unused cells differently
\xa0\xa0// to make the difference glaringly obvious
\xa0\xa0value: --out-of-range ? "X" : @;
\xa0\xa0fill: --out-of-range ? "lightgrey" : "aliceblue";
}

// Add index indicators to taste
:vector "ptr" ref []::extra(index) {
\xa0\xa0display: label;
\xa0\xa0vertical-justify: start;
\xa0\xa0vertical-align: outside;
\xa0\xa0value: --INDEX;
}

// Make labels where the vector's parameters point to
:vector "ptr" ref [].if(--INDEX == --vec-cap - 1)::extra(len-cap) {
\xa0\xa0display: label;
\xa0\xa0value: "cap^";
}
:vector "ptr" ref [].if(--INDEX == --vec-len - 1)::extra(len-cap) {
\xa0\xa0display: label;
\xa0\xa0value: "len^";
}
:vector "ptr" ref []::extra(len-cap) {
\xa0\xa0color: maroon;
\xa0\xa0vertical-justify: end;
\xa0\xa0vertical-align: outside;
\xa0\xa0horizontal-justify: start;
}
 
`;

/**
 * Stylesheet for the linked list example.
 */
export const LIST_STYLESHEET: string =
`:: {
\xa0\xa0display: graph;
\xa0\xa0layout: layered;
\xa0\xa0direction: east;
\xa0\xa0--root: @;
}

:list {
\xa0\xa0display: cell;
\xa0\xa0value: head;
}

:node {
\xa0\xa0display: cell;
\xa0\xa0shape: circle;
\xa0\xa0value: @("value");
\xa0\xa0parent: --root;
}

.alt(:node "next", :list "head") {
\xa0\xa0display: connector;
\xa0\xa0target: @(ref);
\xa0\xa0end/decoration: arrow;
}

:: main .many(next).if(!@(next)) "turtle" {
\xa0\xa0display: label;
\xa0\xa0value: "^ turtle";
\xa0\xa0color: "#85d";
\xa0\xa0parent: @(ref);
\xa0\xa0vertical-justify: end;
\xa0\xa0vertical-align: outside;
}

:: main .many(next).if(!@(next)) "hare" {
\xa0\xa0display: label;
\xa0\xa0value: "hare v";
\xa0\xa0color: "#4a0";
\xa0\xa0parent: @(ref);
\xa0\xa0vertical-justify: start;
\xa0\xa0vertical-align: outside;
}

`;
