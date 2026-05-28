<!--
 * Wrapper for an Aili-Vis viewport with a stateful renderer.
-->

<script setup lang="ts">
    import { onMounted, useTemplateRef } from 'vue';
    import { GdbStateGraph, GdbVisTreeRenderer, Stylesheet } from 'aili-jsapi';
    import { prettyPrintVisTree } from '../utils/pretty-vis';
    import VisViewport from './VisViewport.vue';

    const inner = useTemplateRef('inner');
    let renderer: GdbVisTreeRenderer | undefined;

    onMounted(() => {
        if (!inner.value) {
            console.warn('Element is not mounted in mount hook');
            return;
        }
        renderer = new GdbVisTreeRenderer(inner.value.visTree);
    });

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
            if (inner.value?.visTree?.root) {
                return prettyPrintVisTree(inner.value.visTree.root);
            } else {
                return '[no root]';
            }
        },
    });
</script>

<template>
    <VisViewport ref="inner" />
</template>
