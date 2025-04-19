<!--
 * Controlling of a debug session.
 *
 * Allows the common operations of a debug session -
 * starting, stopping, and single-stepping.
-->

<script setup lang="ts">
    import { computed, ref } from 'vue';
    import { DebugSessionStatus } from '../controllers/session';
    import { DebugSessionManager } from '../controllers/session-manager';

    const { session } = defineProps<{ session: DebugSessionManager }>();

    const pathToDebuggee = ref(session.pathToDebuggee);
    const status = ref(session.status);
    session.onStatusChanged.hook(s => status.value = s);

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

    function nextStep(): void {
        if (status.value === DebugSessionStatus.READY) {
            session.step();
        }
    }
</script>

<template>
    <div class="session-control">
        <label>
            Path to debuggee:
            <input v-model="pathToDebuggee" :disabled="status !== DebugSessionStatus.INACTIVE">
        </label>
        <button :disabled="status === DebugSessionStatus.BUSY || pathToDebuggee === ''" @click="startStop">
            {{ startStopButtonText }}
        </button>
        <button :disabled="status !== DebugSessionStatus.READY" @click="nextStep">
            Step
        </button>
    </div>
</template>

<style>
    .session-control {
        display: flex;
        gap: 0.5em;
    }
</style>
