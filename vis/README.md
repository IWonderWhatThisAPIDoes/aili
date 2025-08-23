# Aili-Vis

Browser-based renderer for the Aili Visualization model,
written in Typescript.

## Using Aili-Vis

To use this module out of the box, create an instance of the `Viewport`
class which connects a logical visualization tree to an HTML container.

The default view model factory provides a number of common
visual representations.

```js
import { Viewport, VisElement, TAG_GRAPH, TAG_CELL, DEFAULT_MODEL_FACTORY } from 'aili-vis';

// Attach a viewport to the DOM
const container = document.getElementById('viewport');
const viewport = new Viewport(container, DEFAULT_MODEL_FACTORY);

// Render a vis tree in the viewport
viewport.root = new VisElement(TAG_GRAPH);

// The viewport will update whenever the vis tree changes
const child = new VisElement(TAG_CELL);
child.parent = viewport.root;
```

See the source code of examples included in the repository
for more complete examples of usage.

To define a new view model, implement the `ViewModel` interface.
This is typically done by extending the abstract `FlowViewModel` class,
which implements basic functionalities that are shared between
most view models.

```js
import { Viewport, VisElement, ViewModelFactory, FlowViewModel, FallbackViewModel } from 'aili-vis';

class MyViewModel extends FlowViewModel {
    constructor(visElement, context) {
        // This view model consists of a 'Hello World' label
        const html = context.ownerDocument.createElement('span');
        html.innerText = 'Hello World';

        // Forward the HTML content to FlowViewModel
        // and let it handle the rest
        super(html);
    }
}

// Pass the custom view model to the viewport
// using a view model factory
const factory = new ViewModelFactory(new Map([['my-model', MyViewModel]]), FallbackViewModel);
const container = document.getElementById('viewport');
const viewport = new Viewport(container, factory);

// Use the model by creating elements with the matching tag name
viewport.root = new VisElement('my-model');
```

## Documentation

The following command generates documentation and saves it to the `doc` directory.

```sh
npm run doc
```

## Examples

The following command builds all examples and saves them to `examples/*/out`.

```sh
npm run examples
```

Alternatively, examples can be built individually with

```sh
npm run examples:<name of example>
```
