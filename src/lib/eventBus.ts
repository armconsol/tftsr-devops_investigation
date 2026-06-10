import { invoke } from "@tauri-apps/api/core";

export type EventCallback<T = unknown> = (data: T) => void;

export interface EventUnsubscribe {
  (): void;
}

export interface EventBus {
  on<T = unknown>(event: string, callback: EventCallback<T>): EventUnsubscribe;
  off<T = unknown>(event: string, callback: EventCallback<T>): void;
  emit<T = unknown>(event: string, data?: T): void;
  once<T = unknown>(event: string, callback: EventCallback<T>): EventUnsubscribe;
}

class SimpleEventBus implements EventBus {
  private events: Record<string, Set<EventCallback<unknown>>> = {};
  private onceEvents: Record<string, Set<EventCallback<unknown>>> = {};

  on<T = unknown>(event: string, callback: EventCallback<T>): EventUnsubscribe {
    if (!this.events[event]) {
      this.events[event] = new Set();
    }
    this.events[event].add(callback as EventCallback<unknown>);
    return () => this.off(event, callback);
  }

  off<T = unknown>(event: string, callback: EventCallback<T>): void {
    if (this.events[event]) {
      this.events[event].delete(callback as EventCallback<unknown>);
    }
  }

  emit<T = unknown>(event: string, data?: T): void {
    const callbacks = this.events[event];
    if (callbacks) {
      callbacks.forEach((callback) => callback(data as unknown));
    }

    const onceCallbacks = this.onceEvents[event];
    if (onceCallbacks) {
      onceCallbacks.forEach((callback) => callback(data as unknown));
      delete this.onceEvents[event];
    }
  }

  once<T = unknown>(event: string, callback: EventCallback<T>): EventUnsubscribe {
    if (!this.onceEvents[event]) {
      this.onceEvents[event] = new Set();
    }
    this.onceEvents[event].add(callback as EventCallback<unknown>);

    return () => {
      if (this.onceEvents[event]) {
        this.onceEvents[event].delete(callback as EventCallback<unknown>);
      }
    };
  }
}

export const eventBus: EventBus = new SimpleEventBus();

export async function subscribeToK8sEvents(
  clusterId: string,
  namespace: string,
  resourceType: string,
  callback: EventCallback<unknown>
): Promise<EventUnsubscribe> {
  try {
    const unsubscribeId = await invoke<string>("subscribe_to_k8s_events", {
      clusterId,
      namespace,
      resourceType,
    });

    const handler = (data: unknown) => {
      callback(data);
    };

    eventBus.on(`k8s:${clusterId}:${namespace}:${resourceType}`, handler);

    return () => {
      // Synchronously remove from eventBus to prevent further callbacks
      eventBus.off(`k8s:${clusterId}:${namespace}:${resourceType}`, handler);
      // Fire-and-forget backend unsubscribe with error handling
      invoke<void>("unsubscribe_from_k8s_events", { unsubscribeId }).catch((err) => {
        console.error("Failed to unsubscribe from backend:", err);
      });
    };
  } catch (error) {
    console.error("Failed to subscribe to K8s events:", error);
    return () => {};
  }
}

export async function subscribeToAllEvents(
  clusterId: string,
  callback: EventCallback<unknown>
): Promise<EventUnsubscribe> {
  try {
    const unsubscribeId = await invoke<string>("subscribe_to_all_k8s_events", {
      clusterId,
    });

    const handler = (data: unknown) => {
      callback(data);
    };

    eventBus.on(`k8s:${clusterId}:all`, handler);

    return () => {
      // Synchronously remove from eventBus to prevent further callbacks
      eventBus.off(`k8s:${clusterId}:all`, handler);
      // Fire-and-forget backend unsubscribe with error handling
      invoke<void>("unsubscribe_from_k8s_events", { unsubscribeId }).catch((err) => {
        console.error("Failed to unsubscribe from backend:", err);
      });
    };
  } catch (error) {
    console.error("Failed to subscribe to all K8s events:", error);
    return () => {};
  }
}
