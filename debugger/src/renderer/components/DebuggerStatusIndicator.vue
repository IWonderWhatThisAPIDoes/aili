<!--
 * Small colored rectangle that inidcates the status
 * of an external debugger process.
-->

<script setup lang="ts">
    import { DebuggerStatus } from '../controllers/debugger';

    defineProps<{ status: DebuggerStatus }>();

    const STATUS_DESCRIPTION: Record<DebuggerStatus, string> = {
        [DebuggerStatus.INACTIVE]: 'inactive',
        [DebuggerStatus.STARTING]: 'starting',
        [DebuggerStatus.STOPPING]: 'stopping',
        [DebuggerStatus.IDLE]: 'idle',
        [DebuggerStatus.LAUNCHING]: 'launching',
        [DebuggerStatus.EXECUTING]: 'running',
        [DebuggerStatus.PAUSED]: 'paused',
        [DebuggerStatus.FAILED_TO_STOP]: 'failed to stop',
    };

    const STATUS_CLASS: Record<DebuggerStatus, string> = {
        [DebuggerStatus.INACTIVE]: 'inactive',
        [DebuggerStatus.STARTING]: 'starting',
        [DebuggerStatus.STOPPING]: 'stopping',
        [DebuggerStatus.IDLE]: 'idle',
        [DebuggerStatus.LAUNCHING]: 'launching',
        [DebuggerStatus.EXECUTING]: 'running',
        [DebuggerStatus.PAUSED]: 'paused',
        [DebuggerStatus.FAILED_TO_STOP]: 'failed-to-stop',
    };
</script>

<template>
    <span :class="['status-indicator', STATUS_CLASS[status]]">
        {{ STATUS_DESCRIPTION[status] }}
    </span>
</template>

<style scoped>
    .status-indicator {
        padding: 0.25em;
        border: 1px solid;
        color: black;
    }

    .status-indicator.inactive {
        background-color: #aaa;
    }

    .status-indicator.starting,
    .status-indicator.stopping {
        background-color: #ed8;
    }

    .status-indicator.idle,
    .status-indicator.launching {
        background-color: #bda;
    }

    .status-indicator.running {
        background-color: #6d4;
    }

    .status-indicator.paused {
        background-color: #5a2;
    }

    .status-indicator.failed-to-stop {
        background-color: #f55;
    }
</style>
