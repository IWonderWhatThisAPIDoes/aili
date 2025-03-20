import { HookableLogger, Severity } from '../../src/logger';
import { expect, describe, it, beforeEach, jest } from '@jest/globals';

const LOG_SEVERITY = Severity.DEBUG;
const LOG_MESSAGE = 'Hello World';
const LOG_DESC = 'Lorem Ipsum';
const TOPIC_NAME = 'foo';
const SUBTOPIC_NAME = 'bar';

describe(HookableLogger, () => {
    let logger: HookableLogger;
    const mockObserver = jest.fn().mockName('logObserver');

    beforeEach(() => {
        mockObserver.mockClear();
        logger = new HookableLogger();
        logger.onLog.hook(mockObserver);
    });

    it('forwards log messages to the hook', () => {
        logger.log(LOG_SEVERITY, LOG_MESSAGE, LOG_DESC);
        expect(mockObserver).toBeCalledWith([], LOG_SEVERITY, LOG_MESSAGE, LOG_DESC);
    });

    it('annotates logged messages with topics', () => {
        const topicLogger = logger.createTopic(TOPIC_NAME);
        topicLogger.log(LOG_SEVERITY, LOG_MESSAGE, LOG_DESC);
        expect(mockObserver).toBeCalledWith([TOPIC_NAME], LOG_SEVERITY, LOG_MESSAGE, LOG_DESC);
    });

    it('allows nested topics', () => {
        const topicLogger = logger.createTopic(TOPIC_NAME);
        const subtopicLogger = topicLogger.createTopic(SUBTOPIC_NAME);
        subtopicLogger.log(LOG_SEVERITY, LOG_MESSAGE, LOG_DESC);
        expect(mockObserver).toBeCalledWith([TOPIC_NAME, SUBTOPIC_NAME], LOG_SEVERITY, LOG_MESSAGE, LOG_DESC);
    });
});
