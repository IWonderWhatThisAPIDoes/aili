<script setup lang="ts">
    import { Severity } from 'aili-hooligan';
    import { DebuggerStatus, LOG_TOPIC_DEBUGGER_META, LOG_TOPIC_FROM_DEBUGGER, LOG_TOPIC_FROM_SESSION, LOG_TOPIC_FROM_STATE, LOG_TOPIC_FROM_USER, LOG_TOPIC_TO_DEBUGGER } from '../controllers/debugger';
    import Console from './Console.vue';
    import DebuggerStatusIndicator from './DebuggerStatusIndicator.vue';
    import LogLine from './LogLine.vue';
</script>

<!--
 * Embedded user guide panel
-->

<template>
    <div class="help-wrapper">
        <div class="help">
            <h1>Aili</h1>
            <p class="centered">
                <a href="https://github.com/IWonderWhatThisAPIDoes/aili">
                    GitHub: IWonderWhatThisAPIDoes/aili
                </a>
            </p>
            <p class="centered">
                Prototype semantic visual debugger developed
                at the <a href="https://fit.cvut.cz/en/">
                Faculty of Information Technology, Czech Technical University in Prague</a>.
            </p>

            <h2>Why is this a thing?</h2>
            <p>
                Aili provides semantic visualization of C programs.
                This means the memory of the debuggee is represented with graphics
                that reflect how the programmer actually thinks about the data
                structures in their programs, rather than raw memory dumps.
                Imagine debugging an implementation of a data structure with
                lots of hard-to-keep-track-of invariants &ndash; say, a self-balancing
                search tree &ndash; and being able to see the tree on the screen
                and review operations made by the program in real time.
            </p>
            <p>
                It is best suited for visualizing small, self-contained programs.
                Think of it as a semantics-focused counterpart to
                <a href="https://pythontutor.com/">Online Python Tutor</a>.
                As such, its use cases are primarily academical.
            </p>
            <ul>
                <li>
                    Students may use it to debug their implementations
                    of algorithms that visualize nicely, but are hard
                    to debug when all you have is a raw memory view.
                </li>
                <li>
                    Teachers may prepare animated demonstrations of algorithms
                    &ndash; and quickly change their underlying implementations
                    if a student asks, simply by recompiling the program.
                </li>
            </ul>

            <h2>The Basics</h2>
            <p>
                For now, Aili creates visualizations based on a description provided
                by the user. The description format is heavily inspired by
                <a href="https://developer.mozilla.org/en-US/docs/Web/CSS">Cascading Style Sheets</a>,
                so if you are familiar with that, you may be able to draw some parallels.
            </p>
            <div class="centered">
                <img src="/assets/pipeline.png" width="100%" max-width="800" />
            </div>
            <p>
                Also, Aili relies on <a href="https://sourceware.org/gdb/">GDB</a>
                to actually look inside the debuggee. Make sure you have that installed
                on your system.
            </p>

            <h2>Carrying a Debug session</h2>
            <p>
                The primary controls of a debug session are in the lower right
                corner of your window. After you have given a path to the program
                you want to debug, you will be able to start a debug session.
                You can manually step through the debuggee at three different paces.
                These will be familiar to you if you have ever used a reasonably modern
                GUI debugger:
            </p>
            <ul>
                <li>Step - advance execution by one line of code.</li>
                <li>Next - advance execution by one line of code, skipping over function calls.</li>
                <li>Out - run the debuggee to the end of the current function.</li>
            </ul>
            <p>
                If the debug session does not start, it may be because
                GDB could not be started. Refer to the <a href="#panel-debugger">Debugger</a>
                panel in that case.
            </p>
            <p>
                This alone, however, does not show off the feature that makes Aili stand out:
                semantic visualization.
                For that, you will need to bring up some of the tool panels
                with the toggles in the lower left corner of your window.
            </p>

            <h2>Tool Panels</h2>
            <p>
                As a user, you should ideally only need a few of these panels:
            </p>
            <ul>
                <li><a href="#panel-debuggee">Debuggee</a></li>
                <li><a href="#panel-source">Source</a></li>
                <li><a href="#panel-stylesheet">Stylesheet</a></li>
                <li><a href="#panel-viewport">Viewport</a></li>
            </ul>
            <p>
                If you intend to write your own stylesheets, you may also find
                some of the diagnostic tool panels useful:
            </p>
            <ul>
                <li><a href="#panel-resolved">Resolved style</a></li>
                <li><a href="#panel-tree">Vis tree</a></li>
                <li><a href="#panel-raw">Raw view</a></li>
            </ul>
            <p>
                The other tool panels may offer an explanation in case the
                application starts behaving unexpectedly.
                This is a prototype that has not received much attention
                in the terms of user experience, so you will not be alerted
                about most errors. Check the log windows if something seems off.
            </p>
            <p>Nonetheless, here they in the order they appear on the toolbar.</p>

            <h3 id="panel-debugger">Debugger</h3>
            <p>This panel provides a view into the underlying instance of GDB.</p>
            <p>
                GDB is expected to be present on your PATH. If it is not,
                you can set the path to its executable on this panel.
            </p>
            <p>
                GDB can also be manually started or stopped with this panel.
                This is not something you would do in a normal debug session,
                but the option is there.
            </p>
            <p>The status indicator shows what GDB is currently doing.</p>
            <ul class="spacious">
                <li>
                    <DebuggerStatusIndicator :status="DebuggerStatus.INACTIVE" />
                    GDB is not running.
                </li>
                <li>
                    <DebuggerStatusIndicator :status="DebuggerStatus.STARTING" />
                    You have manually started GDB.
                </li>
                <li>
                    <DebuggerStatusIndicator :status="DebuggerStatus.STOPPING" />
                    You have manually stopped GDB.
                </li>
                <li>
                    <DebuggerStatusIndicator :status="DebuggerStatus.IDLE" />
                    GDB is active and no debug session is in progress.
                </li>
                <li>
                    <DebuggerStatusIndicator :status="DebuggerStatus.LAUNCHING" />
                    Debuggee is being launched and a debug session is about to start.
                </li>
                <li>
                    <DebuggerStatusIndicator :status="DebuggerStatus.EXECUTING" />
                    Debuggee is actively executing.
                </li>
                <li>
                    <DebuggerStatusIndicator :status="DebuggerStatus.PAUSED" />
                    Debuggee is running and paused on a breakpoint.
                </li>
                <li>
                    <DebuggerStatusIndicator :status="DebuggerStatus.FAILED_TO_STOP" />
                    Error handling status for when you
                    have attempted and failed to manually stop GDB.
                    If you ever see this status, it is probably my fault.
                    You will be given an option to detach from the running instance
                    of GDB, if it is even running at that point.
                </li>
            </ul>
            <p>
                The rest of the panel keeps a short history of communication with GDB.
                Some log messages will appear, but it is mostly messages
                to and from GDB.
                The origin of messages is indicated by their style.
            </p>
            <Console>
                <LogLine :severity="Severity.INFO" message="Log message" :topic="[LOG_TOPIC_DEBUGGER_META]" />
                <LogLine :severity="Severity.DEBUG" message="Debug session controller sent this message" :topic="[LOG_TOPIC_TO_DEBUGGER, LOG_TOPIC_FROM_SESSION]" />
                <LogLine :severity="Severity.DEBUG" message="State analyzer sent this message" :topic="[LOG_TOPIC_TO_DEBUGGER, LOG_TOPIC_FROM_STATE]" />
                <LogLine :severity="Severity.DEBUG" message="You sent this message manually" :topic="[LOG_TOPIC_TO_DEBUGGER, LOG_TOPIC_FROM_USER]" />
                <LogLine :severity="Severity.DEBUG" message="This is a response from GDB" :topic="[LOG_TOPIC_FROM_DEBUGGER]" />
            </Console>
            <p>
                Communication with GDB runs in
                <a href="https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI.html">GDB/MI</a>
                mode, which is a machine-readable communication protocol for integrating
                GDB into a GUI debugger frontend like this one.
            </p>

            <h3 id="panel-debuggee">Debuggee</h3>
            <p>
                This panel allows the user to specify a stylesheet that provides
                hints about sizes of memory buffers to the state analyzer.
                For more information, see the
                <a href="https://github.com/IWonderWhatThisAPIDoes/aili/blob/main/doc/stylesheets.md">documentation on stylesheets</a>.
            </p>
            <p>
                For the stylesheet that describes the graphical representation,
                see the <a href="#panel-stylesheet">Stylesheet</a> panel.
            </p>
            <p>
                An ongoing debug session needs to be restarted before the changes
                made here take effect.
            </p>

            <h3 id="panel-source">Source</h3>
            <p>
                Shows the source file where the debuggee has been paused,
                with an indication of the current line,
                like the average GUI debugger.
            </p>

            <h3 id="panel-stylesheet">Stylesheet</h3>
            <p>
                This panel allows the user to specify a stylesheet
                that describes the desired graphical representation
                of the debuggee. For more information, see the
                <a href="https://github.com/IWonderWhatThisAPIDoes/aili/blob/main/doc/stylesheets.md">documentation on stylesheets</a>.
            </p>
            <p>
                For the stylesheet that provides hints about memory layout,
                see the <a href="#panel-debuggee">Debuggee</a> panel.
            </p>
            <p>
                Changes made here are immediately reflected by the
                <a href="#panel-viewport">Viewport</a> and diagnostic panels.
                In case of a syntax error, only the affected rule is invalidated.
                The rest of the stylesheet is parsed and used in rendering normally.
            </p>

            <h3 id="panel-resolved">Resolved style</h3>
            <p>
                This panel shows the result of resolving the
                <a href="#panel-stylesheet">stylesheet</a>
                over the debuggee's current state.
            </p>
            <p>
                It is primarily a diagnostic tool for developers of Aili (that's me)
                and for stylesheet authors.
            </p>
            <p>
                The output is a table that shows the display properties assigned
                to individual elements.
                The first row lists the reference to the root element.
                If there is no root element, make sure you have set the <code>display</code>
                property of the root node (see the
                <a href="https://github.com/IWonderWhatThisAPIDoes/aili/blob/main/doc/stylesheets.md">documentation on stylesheets</a> for
                more information).
            </p>

            <h3 id="panel-tree">Vis tree</h3>
            <p>
                This is another diagnostic panel that shows an XML-like
                structural serialization of the visual scene that should show up in the
                <a href="#panel-viewport">viewport</a>.
            </p>
            <p>
                If the structure does not match
                the contents of the <a href="#panel-resolved">Resolved style</a> panel,
                there is an error in the translation from the resolved stylesheet
                to the visual scene (my fault).
                If it does not match the contents of the
                <a href="#panel-viewport">viewport</a> (usually some elements will be missing
                in the viewport), verify that you have spelled the <code>display</code>
                properties correctly, that the elements are not missing
                a required property, and that elements with children
                have <code>display</code> set to a model that can have child elements.
            </p>

            <h3 id="panel-viewport">Viewport</h3>
            <p>
                This is the panel that actually contains Aili's interesting output
                &ndash; the semantic visualization of the debuggee's state,
                presented as described by the stylesheet.
            </p>

            <h3 id="panel-raw">Raw view</h3>
            <p>
                Shows the debuggee's state in the way that Aili sees it.
                It is a graph where nodes represent objects and scopes.
                This is the graph against which stylesheets are resolved.
            </p>
            <p>
                This panel can be useful to stylesheet authors
                as a reference against which they write their stylesheet selectors.
            </p>

            <h3 id="panel-log">Log</h3>
            <p>
                This is where all components except <a href="#panel-debugger">the debugger controller</a>
                send their log messages. Messages are identified by the name
                of the component that sent them.
            </p>
            <p>
                Modal notifications are not, as of yet, employed to alert
                the user that an error has occurred,
                so any potential error reports can only be found here.
            </p>
        </div>
    </div>
</template>

<style>
    .help-wrapper {
        overflow: auto;
        scroll-behavior: smooth;
        border: 1px solid;
        background-color: white;
    }

    .help {
        padding: 1em;
    }

    .help > h1, .centered {
        text-align: center;
    }

    .spacious > li {
        margin-top: 1em;
        margin-bottom: 1em;
    }
</style>
