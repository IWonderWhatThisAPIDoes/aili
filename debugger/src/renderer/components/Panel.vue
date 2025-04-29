<!--
 * Wrapper that adds a title to its content and lays it out
 * in a unified way.
-->

<script setup lang="ts">
    defineProps<{ title: string }>();
</script>

<template>
    <div class="panel">
        <div class="head">
            <div class="title">
                {{ title }}
            </div>
        </div>
        <div class="body">
            <slot></slot>
        </div>
    </div>
</template>

<style>
    .panel {
        /* Allow the content to fill the panel */
        display: flex;
        flex-direction: column;
    }

    .panel > .head {
        border-bottom: 1px solid;
        margin-bottom: 0.5em;
    }

    .panel > .head > .title {
        font-size: larger;
        font-weight: bolder;
        /* Set width to zero to prevent it from affecting flex layout */
        width: 0;
        text-wrap-mode: nowrap;
    }

    .panel > .body {
        /* Fill the rest of the panel */
        flex-grow: 1;
        /* Relative positioning to fill this with the content */
        position: relative;
    }

    .panel > .body > * {
        /* Fill the container without altering its dimensions */
        position: absolute;
        width: 100%;
        height: 100%;
        box-sizing: border-box;
    }
</style>
