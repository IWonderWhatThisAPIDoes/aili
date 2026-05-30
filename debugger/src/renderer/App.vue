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
    import { onMounted, ref, useTemplateRef } from 'vue';
    import { HookableLogger } from 'aili-hooligan';
    import { GdbStateGraph, Stylesheet, PropertyMap } from 'aili-jsapi';
    import { Debugger } from './controllers/debugger';
    import { DEFAULT_STYLESHEET } from './utils/default-stylesheet';
    import { MetaVisTreeRenderer } from './utils/meta-vis-tree';
    import { DebugSessionStatus } from './controllers/session';
    import { DebugSessionManager } from './controllers/session-manager';
    import { SourceViewer } from './controllers/source-viewer';
    import DebuggerControlPanel from './components/DebuggerControlPanel.vue';
    import ScrollBox from './components/ScrollBox.vue';
    import LogConsole from './components/LogConsole.vue';
    import StyleEditor from './components/StyleEditor.vue';
    import DebugSessionControl from './components/DebugSessionControl.vue';
    import VisViewportWithRenderer from './components/VisViewportWithRenderer.vue';
    import SourceView from './components/SourceView.vue';
    import HelpPage from './components/HelpPage.vue';
    import DebuggeeControlPanel from './components/DebuggeeControlPanel.vue';
    import AppView from './components/AppView.vue';
    import Panel from './components/Panel.vue';
    import VisViewport from './components/VisViewport.vue';
    import PropertyTable from './components/PropertyTable.vue';

    const mainViewport = useTemplateRef('main-viewport');
    const rawViewport = useTemplateRef('raw-viewport');
    const treeViewport = useTemplateRef('tree-viewport');
    const logConsole = useTemplateRef('log-console');

    const resolvedStyle = ref([] as PropertyMap[]);
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
            resolvedStyle.value = mainViewport.value.resolvedStyleTable();
        }
    }

    function stylesheetChanged(_: string, stylesheet: Stylesheet): void {
        mainStylesheet = stylesheet;
        applyMainStylesheet();
    }

    onMounted(() => {
        if (!mainViewport.value?.visTree || !treeViewport.value) {
            console.warn('Element is not mounted in mount hook');
            return;
        }
        new MetaVisTreeRenderer(mainViewport.value.visTree, treeViewport.value.visTree);
    });
</script>

<template>
    <AppView>
        <Panel title="Debugger">
            <DebuggerControlPanel :debug="debuggerContainer" />
        </Panel>
        <Panel title="Debuggee">
            <DebuggeeControlPanel :session="debugSession" />
        </Panel>
        <Panel title="Source">
            <SourceView :debug="debuggerContainer" :sourceViewer="sourceViewer" />
        </Panel>
        <Panel title="Stylesheet">
            <StyleEditor
                :content="DEFAULT_STYLESHEET"
                :compile="Stylesheet.parse"
                @style-changed="stylesheetChanged"
            />
        </Panel>
        <Panel title="Resolved Style">
            <ScrollBox>
                <div>
                    <PropertyTable class="resolved-style-table" :mapping="resolvedStyle" />
                </div>
            </ScrollBox>
        </Panel>
        <Panel title="VisTree">
            <VisViewport ref="tree-viewport" />
        </Panel>
        <Panel title="Viewport">
            <VisViewportWithRenderer ref="main-viewport" />
        </Panel>
        <Panel title="Raw View">
            <VisViewportWithRenderer ref="raw-viewport" />
        </Panel>
        <Panel title="Log">
            <ScrollBox>
                <LogConsole ref="log-console" showTopic />
            </ScrollBox>
        </Panel>
        <Panel title="Help" initshow>
            <HelpPage />
        </Panel>
        <template #extra-footer>
            <DebugSessionControl class="session-control" :session="debugSession" />
        </template>
    </AppView>
</template>

<style>
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

    .session-control {
        margin-left: auto;
    }

    .resolved-style-table {
        min-width: 100%;
    }
</style>
