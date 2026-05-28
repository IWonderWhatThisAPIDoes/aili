<!--
 * Wrapper for a bare Aili-Vis viewport that can be bound from the outside.
-->

<script setup lang="ts">
    import { onMounted, onUnmounted, useTemplateRef } from 'vue';
    import { DEFAULT_MODEL_FACTORY, Viewport } from 'aili-vis';
    import { VisTree } from '../utils/vis-tree';

    const container = useTemplateRef('container');
    let viewport: Viewport | undefined;

    const visTree = new VisTree();
    visTree.onRootChanged.hook(root => {
        if (viewport != undefined) {
            viewport.root = root;
        }
    });

    onMounted(() => {
        if (!container.value) {
            console.warn('Element is not mounted in mount hook');
            return;
        }
        viewport = new Viewport(container.value, DEFAULT_MODEL_FACTORY);
        if (visTree.root != undefined) {
            viewport.root = visTree.root;
        }
    });
    onUnmounted(() => (viewport = undefined));

    defineExpose({
        /**
         * Vis tree bound to the viewport.
         */
        visTree,
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
