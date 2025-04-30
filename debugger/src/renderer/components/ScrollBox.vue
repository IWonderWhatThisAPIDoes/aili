<!--
 * Wrapper that renders a scroll container over its content.
-->

<script lang="ts">
    import { InjectionKey } from 'vue';

    /**
     * Injection key that allows children to scroll to the bottom of the scroll box.
     */
    export const INJECT_SCROLL_TO_BOTTOM: InjectionKey<() => void> = Symbol('scroll-to-bottom');
</script>

<script setup lang="ts">
    import { provide, useTemplateRef } from 'vue';

    const scrollBox = useTemplateRef('scroll-box');
    /**
     * Scrolls the scroll box to a specified position vertically.
     * 
     * @param pos Relative scroll position, [0, 1].
     */
    function scrollToY(pos: number): void {
        if (scrollBox.value) {
            var adjpos = (pos - 0.5) / (1 - scrollBox.value.clientHeight / scrollBox.value.scrollHeight) + 0.5;
            adjpos = Math.max(Math.min(adjpos, 1), 0);
            scrollBox.value.scrollTo({ top: (scrollBox.value.scrollHeight - scrollBox.value.clientHeight) * adjpos });
        }
    }
    /**
     * Scrolls the scroll box to the bottom.
     */
    function scrollToBottom(): void {
        if (scrollBox.value) {
            scrollBox.value.scrollTo({ top: scrollBox.value.scrollHeight });
        }
    }

    provide(INJECT_SCROLL_TO_BOTTOM, scrollToBottom);
    defineExpose({
        scrollToY,
        scrollToBottom,
    });
</script>

<template>
    <div class="scroll-box" ref="scroll-box">
        <slot></slot>
    </div>
</template>

<style>
    .scroll-box {
        display: flex;
        overflow: auto;
    }

    .scroll-box > * {
        flex-grow: 1;
        min-width: fit-content;
        min-height: fit-content;
    }
</style>
