import { EventProcessor, Integration } from "@sentry/types";
export declare class TauriIntegration implements Integration {
    static id: string;
    name: string;
    setupOnce(addGlobalEventProcessor: (callback: EventProcessor) => void): void;
}
