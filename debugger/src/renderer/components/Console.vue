<!--
 * Wrapper that renders its contents with terminal aesthetics.
-->

<template>
    <div class="console">
        <slot></slot>
    </div>
</template>

<style>
    .console {
        padding: 0.5em;
        font-family: Consolas, 'Courier New', Courier, monospace;
        white-space: pre;
        background-color: black;
        color: lightgrey;
    }
</style>
