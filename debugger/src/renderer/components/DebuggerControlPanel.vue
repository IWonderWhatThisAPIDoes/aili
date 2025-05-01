<!--
 * Control panel of a debugger session.
 *
 * Displays debugger status and IO log and permits starting
 * and stopping the debugger, and sending arbitrary commands to it.
-->

<script setup lang="ts">
    import { computed, ref, useTemplateRef } from 'vue';
    import { Debugger, DebuggerInputSource, DebuggerStatus, isStatusRunning } from '../controllers/debugger';
    import DebuggerStatusIndicator from './DebuggerStatusIndicator.vue';
    import LogConsole from './LogConsole.vue';
    import ScrollBox from './ScrollBox.vue';
    import { HookableLogger } from 'aili-hooligan';

    const { debug } = defineProps<{ debug: Debugger }>();

    // Bridge between Vue reactivity API and Aili-Hooligan hooks
    const pid = ref(debug.pid);
    const status = ref(debug.status);
    debug.onStatusChanged.hook((s, p) => {
        status.value = s;
        pid.value = p;
    });

    const isAcceptingInput = computed(() => isStatusRunning(status.value));
    const isStatusChangeable = computed(() => status.value !== DebuggerStatus.STARTING && status.value !== DebuggerStatus.STOPPING);
    const changeStatusActionName = computed<string>(prev => {
        switch (status.value) {
            case DebuggerStatus.INACTIVE:
                return 'Start';
            case DebuggerStatus.IDLE:
            case DebuggerStatus.LAUNCHING:
            case DebuggerStatus.EXECUTING:
            case DebuggerStatus.PAUSED:
                return 'Stop';
            default:
                return prev ?? '';
        }
    });
    function changeStatus(): void {
        switch (status.value) {
            case DebuggerStatus.INACTIVE:
                debug.start();
                break;
            case DebuggerStatus.IDLE:
            case DebuggerStatus.LAUNCHING:
            case DebuggerStatus.EXECUTING:
            case DebuggerStatus.PAUSED:
                debug.stop();
                break;
        }
    }

    const logConsole = useTemplateRef('log-console');
    const debugLogger = new HookableLogger();
    debug.logger = debugLogger;
    debugLogger.onLog.hook((...log) => {
        logConsole.value?.addEntry(...log);
    });

    const inputToDebugger = ref('');
    function sendInputToDebugger(): void {
        const line = inputToDebugger.value;
        inputToDebugger.value = '';
        debug.sendInputLine(line, DebuggerInputSource.USER);
    }
</script>

<template>
    <div class="debugger-controls">
        <div class="debugger-control-row debugger-path">
            <label>
                Path to debugger:
                <input v-model="Debugger.pathToDebugger">
            </label>
        </div>
        <div class="debugger-control-row debugger-status">
            Status: <DebuggerStatusIndicator :status="status" />
            <button :disabled="!isStatusChangeable" @click="changeStatus">
                {{ changeStatusActionName }}
            </button>
        </div>
        <div class="debugger-control-row debugger-pid">PID: {{ String(pid ?? 'NA') }}</div>
        <ScrollBox class="debugger-log">
            <LogConsole ref="log-console" />
        </ScrollBox>
        <label class="debugger-control-row debugger-input">
            GDB>
            <input v-model="inputToDebugger" @keyup.enter.self="sendInputToDebugger" :disabled="!isAcceptingInput">
        </label>
    </div>
</template>

<style>
    .debugger-controls {
        display: flex;
        flex-direction: column;
        gap: 0.25em;
    }

    .debugger-control-row {
        display: flex;
        align-items: baseline;
        gap: 0.25em;
    }

    .debugger-log,
    .debugger-input > input {
        flex-grow: 1;
    }

    .log-line.topic-from-debugger {
        color: gray;
    }

    .log-line.topic-to-debugger {
        color: aquamarine;
    }

    .log-line.topic-to-debugger::after {
        display: inline-block;
        content: 'unknown source';
        border: 1px solid;
        font-size: smaller;
        margin-left: 0.5em;
    }

    .log-line.topic-to-debugger.topic-from-user::after {
        content: 'user';
    }

    .log-line.topic-to-debugger.topic-from-session {
        color: yellowgreen;
    }

    .log-line.topic-to-debugger.topic-from-session::after {
        content: 'session';
    }

    .log-line.topic-to-debugger.topic-from-state {
        color: yellow;
    }

    .log-line.topic-to-debugger.topic-from-state::after {
        content: 'state';
    }

    .log-line.topic-to-debugger::before, .log-line.topic-from-debugger::before {
        /* Drop these labels to make more space */
        content: none;
    }
</style>
