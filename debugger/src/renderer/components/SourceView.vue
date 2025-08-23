<!--
 * View of source code that updates whenever the debugger stops.
-->

<script lang="ts">
    /**
     * Enumerates the states in which a source view can be.
     */
    enum SourceViewStatus {
        /**
         * Source is not displayed because no debug session
         * has been started yet.
         */
        UNINITIALIZED,
        /**
         * Source is displayed.
         */
        VIEWING,
        /**
         * Source location information is not available.
         */
        UNKNOWN_LOCATION,
        /**
         * The source file could not be loaded.
         */
        LOAD_FAILED,
    }
</script>

<script setup lang="ts">
    import { nextTick, ref, useTemplateRef } from 'vue';
    import { Debugger, DebuggerStatus } from '../controllers/debugger';
    import { SourceViewer } from '../controllers/source-viewer';
    import Console from './Console.vue';
    import ScrollBox from './ScrollBox.vue';

    const { debug, sourceViewer } = defineProps<{ debug: Debugger; sourceViewer: SourceViewer }>();
    const source = ref<readonly string[]>([]);
    const lineNumber = ref<number | undefined>(undefined);
    const status = ref(SourceViewStatus.UNINITIALIZED);
    const scrollBox = useTemplateRef('scrollbox');

    function scrollToCurrentLine(): void {
        if (lineNumber.value && scrollBox.value) {
            scrollBox.value.scrollToY((lineNumber.value - 1) / (source.value.length - 1));
        }
    }

    // Listen for when the debugger's state changes
    debug.onStatusChanged.hook(async (debugStatus, _, sourceLocation) => {
        if (debugStatus !== DebuggerStatus.PAUSED) {
            // Leave the last source file open, we would probably load it again on next pause anyway
            lineNumber.value = undefined;
            return;
        }
        if (!sourceLocation?.filePath) {
            // Debugger paused, but no source location information is available,
            // so clear the source view because we are not going to see anything
            source.value = [];
            lineNumber.value = undefined;
            status.value = SourceViewStatus.UNKNOWN_LOCATION;
            return;
        }
        try {
            // Read the source and display it in the window
            source.value = await sourceViewer.loadFile(sourceLocation.filePath);
            lineNumber.value = sourceLocation.lineNumber;
            status.value = SourceViewStatus.VIEWING;
            nextTick(scrollToCurrentLine);
        } catch (e) {
            // The source file could not be read, so clear the data
            source.value = [];
            lineNumber.value = undefined;
            status.value = SourceViewStatus.LOAD_FAILED;
        }
    });
</script>

<template>
    <ScrollBox ref="scrollbox">
        <Console class="source-view">
            <template v-if="status === SourceViewStatus.VIEWING">
                <!-- Column with line numbers -->
                <div class="source-line-numbers">
                    <div
                        v-for="(_, i) in source"
                        class="source-line-number"
                        :class="{ 'current-line': i + 1 === lineNumber }"
                    >
                        {{ i + 1 }}
                    </div>
                </div>
                <!-- Column with the source code -->
                <div class="source-code">
                    <div
                        v-for="(line, i) in source"
                        class="source-line"
                        :class="{ 'current-line': i + 1 === lineNumber }"
                    >
                        {{ line }}
                    </div>
                </div>
            </template>
            <!-- If we are not seeing any code, show a message explaining why -->
            <div
                v-else
                class="source-view-hint"
                :class="{
                    'source-view-uninit': status === SourceViewStatus.UNINITIALIZED,
                    'source-view-error': status !== SourceViewStatus.UNINITIALIZED,
                }"
            >
                <template v-if="status === SourceViewStatus.UNINITIALIZED">
                    <span>Start a debug session to view the source code</span>
                </template>
                <template v-else-if="status === SourceViewStatus.UNKNOWN_LOCATION">
                    <span>Source location information is not available</span>
                </template>
                <template v-else-if="status === SourceViewStatus.LOAD_FAILED">
                    <span>Cannot preview source: file moved or missing</span>
                </template>
            </div>
        </Console>
    </ScrollBox>
</template>

<style>
    .source-view {
        display: flex;
        gap: 1em;
    }

    .source-line-numbers {
        color: grey;
        text-align: right;
        pointer-events: none;
        user-select: none;
    }

    .source-line-number {
        padding-left: 0.5em;
    }

    .source-code {
        flex-grow: 1;
        min-height: fit-content;
    }

    .source-line {
        /* Do not collapse empty lines, they should have the full height */
        min-height: 1lh;
    }

    .source-line.current-line {
        background-color: #9acd3240;
    }

    .source-line-number.current-line {
        color: black;
        background-color: yellowgreen;
        position: relative;
        /* Bring it one level forward, so that we may insert the tip behid it */
        z-index: 1;
    }

    .source-line-number.current-line::after {
        /* This will become the arrow tip */
        content: '';
        position: absolute;
        background-color: yellowgreen;
        transform: rotate(45deg);
        /* Place it behind the parent element */
        z-index: -1;
        width: 0.707lh; /* ~ sqrt(1/2) */
        height: 0.707lh;
        top: 0.1465lh; /* (1lh - height) / 2*/
        right: -0.3535lh; /* (height - 1lh */
    }

    .source-view-hint {
        flex-grow: 1;
        text-align: center;
        margin-top: 2lh;
    }

    .source-view-error {
        color: #f55;
    }
</style>
