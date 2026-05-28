<!--
 * Top-level multi-page layout of the application.
-->

<script setup lang="ts">
    import { computed, reactive, useSlots } from 'vue';

    // App panels will be passed through the default slot
    const mainSlot = useSlots().default;
    // Initially show panels that have the special attribute
    const showPanel = reactive(
        (mainSlot && mainSlot().map(node => node.props?.initshow != null)) || [],
    );
    const allPanelsHidden = computed(() => showPanel.every(s => !s));
</script>

<template>
    <div class="app">
        <div class="main">
            <div class="placeholder" v-show="allPanelsHidden"></div>
            <component
                v-if="$slots.default"
                v-for="(node, i) in $slots.default()"
                :is="node"
                class="panel"
                v-show="showPanel[i]"
            />
        </div>
        <div class="footer">
            <div v-if="$slots.default" class="display-settings">
                <span class="display-settings-label">Show:</span>
                <label v-for="(node, i) in $slots.default()">
                    <input type="checkbox" v-model="showPanel[i]" />
                    {{ node.props?.title }}
                </label>
            </div>
            <slot name="extra-footer"></slot>
        </div>
    </div>
</template>

<style>
    .app > .main {
        /* Fill the application container */
        flex-grow: 1;
        /* Lay out panels side-by-side */
        display: flex;
        gap: 0.5em;
    }

    .app > .main > .placeholder {
        /* When the container is empty, show something
        so it does not look blank */
        background-image: url('../../../assets/empty-screen.svg');
        background-repeat: no-repeat;
        background-size: contain;
        background-position: bottom;
    }

    .app > .main > .panel,
    .app > .main > .placeholder {
        /* Fill the screen horizontally */
        flex-grow: 1;
    }

    .app > .footer {
        margin-top: 0.5em;
        display: flex;
        flex-wrap: wrap-reverse;
        gap: 1em;
    }

    .display-settings {
        display: flex;
        align-items: baseline;
        flex-wrap: wrap;
        gap: 1em;
    }

    .display-settings-label {
        font-size: larger;
        font-weight: bolder;
    }

    .display-settings > label {
        display: flex;
        align-items: baseline;
        gap: 0.25em;
    }

    .display-settings input[type='checkbox'] {
        margin: 0;
    }
</style>
