<!--
 * Wrapper for an Aili-Vis viewport with a stateful renderer.
-->

<script setup lang="ts">
    import { onMounted, onUnmounted, useTemplateRef } from 'vue';
    import { DEFAULT_MODEL_FACTORY, Viewport, VisConnector, VisElement } from 'aili-vis';
    import { GdbStateGraph, GdbVisTreeRenderer, Stylesheet } from 'aili-jsapi';
    import { prettyPrintVisTree } from '../utils/pretty-vis';

    const container = useTemplateRef('container');
    let viewport: Viewport | undefined;
    let renderer: GdbVisTreeRenderer | undefined;

    onMounted(() => {
        if (!container.value) {
            console.warn('Element is not mounted in mount hook');
            return;
        }
        const vp = viewport = new Viewport(container.value, DEFAULT_MODEL_FACTORY);
        renderer = new GdbVisTreeRenderer({
            createElement: tagName => new VisElement(tagName),
            createConnector: () => new VisConnector,
            set root(root: VisElement) { vp.root = root; }
        });
    });
    onUnmounted(() => viewport = undefined);

    defineExpose({
        /**
         * Renders a state graph, resolved with a specified stylesheet,
         * into the viewport.
         * 
         * Stateful renderer is used, so this method may be called
         * multiple times to update only the parts of the visualization
         * that have changed.
         * 
         * @param stateGraph State graph that should be rendered.
         * @param stylesheet Stylesheet that describes how the state graph
         *                   should be represented.
         */
        render(stateGraph: GdbStateGraph, stylesheet: Stylesheet): void {
            renderer?.applyStylesheet(stylesheet, stateGraph);
        },
        /**
         * Serializes the resolved style passed to the viewport
         * in a human-readable format.
         * 
         * @returns Human-readable representation of the resolved style.
         */
        prettyPrintResolvedStyle(): string {
            return renderer?.prettyPrint() ?? '[not available]';
        },
        /**
         * Serializes the structure of the Vis tree
         * in a human-readable format.
         * 
         * @returns Human.readablerepresentation of the Vis tree.
         */
        prettyPrintVisTree(): string {
            if (viewport?.root) {
                return prettyPrintVisTree(viewport.root);
            } else {
                return '[no root]';
            }
        },
    });
</script>

<template>
    <div class="viewport" ref="container"></div>
</template>

<style>
    .viewport {
        background-color: white;
        border: 1px solid;
    }
</style>
