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
    function scrollToBottom(): void {
        if (scrollBox.value) {
            scrollBox.value.scrollTo({ top: scrollBox.value.scrollHeight });
        }
    }
    provide(INJECT_SCROLL_TO_BOTTOM, scrollToBottom);
    defineExpose({ scrollToBottom });
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
