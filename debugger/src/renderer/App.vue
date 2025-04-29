<script lang="ts">
    /**
     * Key of the log topic that identifies all log messages
     * from {@link DebugSessionManager}.
     */
    export const LOG_TOPIC_SESSION: string = 'session';
    /**
     * Key of the log topic that identifies all log messages
     * from {@link SourceViewer}.
     */
    export const LOG_TOPIC_SOURCE_VIEWER: string = 'source-viewer';
</script>

<script setup lang="ts">
    import { computed, ref, useTemplateRef } from 'vue';
    import { HookableLogger } from 'aili-hooligan';
    import { GdbStateGraph, Stylesheet } from 'aili-jsapi';
    import { Debugger } from './controllers/debugger';
    import { DEFAULT_STYLESHEET } from './utils/default-stylesheet';
    import { DebugSessionStatus } from './controllers/session';
    import { DebugSessionManager } from './controllers/session-manager';
    import { SourceViewer } from './controllers/source-viewer';
    import Panel from './components/Panel.vue';
    import DebuggerControlPanel from './components/DebuggerControlPanel.vue';
    import ScrollBox from './components/ScrollBox.vue';
    import Console from './components/Console.vue';
    import LogConsole from './components/LogConsole.vue';
    import StyleEditor from './components/StyleEditor.vue';
    import DebugSessionControl from './components/DebugSessionControl.vue';
    import VisViewport from './components/VisViewport.vue';
    import SourceView from './components/SourceView.vue';

    const showDebugger = ref(false);
    const showLog = ref(false);
    const showStylesheet = ref(true);
    const showStyle = ref(false);
    const showVis = ref(false);
    const showViewport = ref(true);
    const showRaw = ref(false);
    const showSource = ref(false);
    const allPanelsHidden = computed(() => {
        return !showDebugger.value && !showLog.value && !showStylesheet.value &&
            !showStyle.value && !showVis.value && !showViewport.value &&
            !showRaw.value && !showSource.value;
    });

    const mainViewport = useTemplateRef('main-viewport');
    const rawViewport = useTemplateRef('raw-viewport');
    const logConsole = useTemplateRef('log-console');

    const resolvedStyle = ref('');
    const resolvedVisTree = ref('');
    const rawStylesheet = Stylesheet.parse(DEFAULT_STYLESHEET);
    let mainStylesheet: Stylesheet;
    let stateGraph: GdbStateGraph | undefined;

    const debuggerContainer = new Debugger();
    const debugSession = new DebugSessionManager(debuggerContainer);
    debugSession.onStateGraphUpdate.hook(state => {
        stateGraph = state;
        rawViewport.value?.render(stateGraph, rawStylesheet);
        applyMainStylesheet();
    });

    const sourceViewer = new SourceViewer();
    // Delete all cached sources when a debug session ends
    // so that we may modify the sources in between sessions
    debugSession.onStatusChanged.hook(status => {
        if (status === DebugSessionStatus.INACTIVE) {
            sourceViewer.clearCache();
        }
    });

    const mainLogger = new HookableLogger();
    debugSession.logger = mainLogger.createTopic(LOG_TOPIC_SESSION);
    sourceViewer.logger = mainLogger.createTopic(LOG_TOPIC_SOURCE_VIEWER);
    mainLogger.onLog.hook((...log) => {
        logConsole.value?.addEntry(...log);
    });

    function applyMainStylesheet(): void {
        if (stateGraph && mainStylesheet && mainViewport.value) {
            mainViewport.value.render(stateGraph, mainStylesheet);
            resolvedStyle.value = mainViewport.value.prettyPrintResolvedStyle();
            resolvedVisTree.value = mainViewport.value.prettyPrintVisTree();
        }
    }

    function stylesheetChanged(_: string, stylesheet: Stylesheet): void {
        mainStylesheet = stylesheet;
        applyMainStylesheet();
    }
</script>

<template>
    <div class="app">
        <div class="main">
            <div class="placeholder" v-show="allPanelsHidden"></div>
            <Panel class="panel" title="Debugger" v-show="showDebugger">
                <DebuggerControlPanel :debug="debuggerContainer" />
            </Panel>
            <Panel class="panel" title="Source" v-show="showSource">
                <SourceView :debug="debuggerContainer" :sourceViewer="sourceViewer" />
            </Panel>
            <Panel class="panel" title="Stylesheet" v-show="showStylesheet">
                <StyleEditor
                    :content="DEFAULT_STYLESHEET"
                    @style-changed="stylesheetChanged"/>
            </Panel>
            <Panel class="panel" title="Resolved Style" v-show="showStyle">
                <ScrollBox>
                    <Console>
                        {{ resolvedStyle }}
                    </Console>
                </ScrollBox>
            </Panel>
            <Panel class="panel" title="Vis Tree" v-show="showVis">
                <ScrollBox>
                    <Console>
                        {{ resolvedVisTree }}
                    </Console>
                </ScrollBox>
            </Panel>
            <Panel class="panel" title="Viewport" v-show="showViewport">
                <VisViewport ref="main-viewport" />
            </Panel>
            <Panel class="panel" title="Raw View" v-show="showRaw">
                <VisViewport ref="raw-viewport" />
            </Panel>
            <Panel class="panel" title="Log" v-show="showLog">
                <ScrollBox>
                    <LogConsole ref="log-console" />
                </ScrollBox>
            </Panel>
        </div>
        <div class="footer">
            <div class="display-settings">
                <span class="display-settings-label">Show:</span>
                <label>
                    <input type="checkbox" v-model="showDebugger">
                    Debugger
                </label>
                <label>
                    <input type="checkbox" v-model="showSource">
                    Source
                </label>
                <label>
                    <input type="checkbox" v-model="showStylesheet">
                    Stylesheet
                </label>
                <label>
                    <input type="checkbox" v-model="showStyle">
                    Resolved Style
                </label>
                <label>
                    <input type="checkbox" v-model="showVis">
                    Vis Tree
                </label>
                <label>
                    <input type="checkbox" v-model="showViewport">
                    Viewport
                </label>
                <label>
                    <input type="checkbox" v-model="showRaw">
                    Raw View
                </label>
                <label>
                    <input type="checkbox" v-model="showLog">
                    Log
                </label>
            </div>
            <DebugSessionControl class="session-control" :session="debugSession" />
        </div>
    </div>
</template>

<style scoped>
    .app {
        width: 100%;
        height: 100%;
        /* Pad the edges of the screen */
        padding: 0.5rem;
        box-sizing: border-box;
        /* Lay out main content above the display settings */
        display: flex;
        flex-direction: column;
    }

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
        background-image: url('../../assets/empty-screen.svg');
        background-repeat: no-repeat;
        background-size: contain;
        background-position: bottom;
    }

    .app > .main > .panel, .app > .main > .placeholder {
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

    .display-settings input[type="checkbox"] {
        margin: 0;
    }

    .session-control {
        margin-left: auto;
    }
</style>
