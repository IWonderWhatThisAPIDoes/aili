<!--
 * Controlling of a debug session.
 *
 * Allows the common operations of a debug session -
 * starting, stopping, and single-stepping.
-->

<script lang="ts">
    import { SourceLocation } from '../controllers/debugger';

    function sourceLocationText(sourceLocation: SourceLocation): string {
        let output = '';
        if (sourceLocation.functionName) {
            output += sourceLocation.functionName + ', ';
        }
        if (sourceLocation.fileName && sourceLocation.lineNumber) {
            output += `${sourceLocation.fileName}:${sourceLocation.lineNumber}`;
        } else {
            output += '???';
        }
        return output;
    }
</script>

<script setup lang="ts">
    import { computed, ref } from 'vue';
    import { DebugSessionStatus, DebugStepRange } from '../controllers/session';
    import { DebugSessionManager } from '../controllers/session-manager';

    const { session } = defineProps<{ session: DebugSessionManager }>();

    const pathToDebuggee = ref(session.pathToDebuggee);
    const status = ref(session.status);
    const sourceLocation = ref(session.sourceLocation);
    session.onStatusChanged.hook(s => {
        sourceLocation.value = session.sourceLocation;
        status.value = s;
    });

    const startStopButtonText = computed(prev => {
        switch (status.value) {
            case DebugSessionStatus.INACTIVE:
                return 'Start';
            case DebugSessionStatus.READY:
                return 'Stop';
            default:
                return prev ?? '';
        }
    });

    function startStop(): void {
        switch (status.value) {
            case DebugSessionStatus.INACTIVE:
                session.pathToDebuggee = pathToDebuggee.value;
                session.start();
                break;
            case DebugSessionStatus.READY:
                session.stop();
                break;
        }
    }

    function nextStep(range: DebugStepRange): void {
        if (status.value === DebugSessionStatus.READY) {
            session.step(range);
        }
    }
</script>

<template>
    <div class="session-control">
        <label>
            Path to debuggee:
            <input v-model="pathToDebuggee" :disabled="status !== DebugSessionStatus.INACTIVE">
        </label>
        <span v-if="sourceLocation !== undefined">
            {{ sourceLocationText(sourceLocation) }}
        </span>
        <button :disabled="status === DebugSessionStatus.BUSY || pathToDebuggee === ''" @click="startStop">
            {{ startStopButtonText }}
        </button>
        <button :disabled="status !== DebugSessionStatus.READY" @click="nextStep(DebugStepRange.SINGLE)">
            Step
        </button>
        <button :disabled="status !== DebugSessionStatus.READY" @click="nextStep(DebugStepRange.NEXT)">
            Next
        </button>
        <button :disabled="status !== DebugSessionStatus.READY" @click="nextStep(DebugStepRange.FINISH)">
            Out
        </button>
    </div>
</template>

<style>
    .session-control {
        display: flex;
        gap: 0.5em;
        align-items: baseline;
    }
</style>
