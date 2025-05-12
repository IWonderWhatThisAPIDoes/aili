/**
 * Aili-Translate Stylesheet that is used to render the Raw View panel,
 * and the default appearence of the Viewoprt panel.
 * 
 * @module
 */

/**
 * Default stylesheet for Raw View and for initial style of Viewport panel.
 */
export const DEFAULT_STYLESHEET: string =
`/**
 * Default stylesheet that renders
 * program state exactly the way it is
 * represented by the state graph.
 */

:: {
  display: graph;
  --root: @;
}

:: ::extra {
  display: cell;
  value: "<root>";
  size: 4;
  fill: "#d0cdcd";
  stroke-width: 2;
}

* {
  display: cell;
  value: @ + ":" + typename(@);
  size: 1;
  fill: "#b5d2ee";
  parent: --root;
}

:frame {
  value: typename(@) + "()";
  size: 3;
  fill: "#ca5";
  stroke-width: 2;
}

:ref {
  value: "&";
  fill: "#d0cdcd";
}

:arr {
  value: "[]";
  fill: "#d0cdcd";
}

:: {
  --root-extra: @(::extra);
}

:: *::edge {
  parent: --root-extra;
}

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

.alt(main, next)::edge {
  stroke-width: 2;
}

ref::edge {
  stroke-style: dashed;
}

::edge {
  end/decoration: arrow;
}

`;

/**
 * Default pointer hint sheet that does not provide any hints,
 * but explains the usage to the user.
 */
export const DEFAULT_HINT_SHEET: string =
`/**
 * This is the hint sheet -
 * hints for the backend to determine
 * the length of dynamically allocated
 * arrays go here
 * 
 * An example hint sheet might look like this:
 * 
 * :vector {
 *   // Get the vector's capacity
 *   --cap: @("cap");
 * }
 * 
 * :vector "ptr" {
 *   // Tell the backend to treat
 *   // the vector's buffer
 *   // as a pointer to array,
 *   // not to a single element,
 *   // and that its length can be
 *   // found in this variable
 *   length: --cap;
 * }
 */

`;
