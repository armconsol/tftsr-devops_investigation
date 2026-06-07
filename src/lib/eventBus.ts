import { invoke } from "@tauri-apps/api/core";

export type EventCallback<T = any> = (data: T) => void;

export interface EventUnsubscribe {
  (): void;
}

export interface EventBus {
  on<T = any>(event: string, callback: EventCallback<T>): EventUnsubscribe;
  off(event: string, callback: EventCallback): void;
  emit<T = any>(event: string, data?: T): void;
  once<T = any>(event: string, callback: EventCallback<T>): EventUnsubscribe;
}

class SimpleEventBus implements EventBus {
  private events: Record<string, Set<EventCallback>> = {};
  private onceEvents: Record<string, Set<EventCallback>> = {};

  on<T = any>(event: string, callback: EventCallback<T>): EventUnsubscribe {
    if (!this.events[event]) {
      this.events[event] = new Set();
    }
    this.events[event].add(callback);

    return () => this.off(event, callback);
  }

  off(event: string, callback: EventCallback): void {
    if (this.events[event]) {
      this.events[event].delete(callback);
    }
  }

  emit<T = any>(event: string, data?: T): void {
    const callbacks = this.events[event];
    if (callbacks) {
      callbacks.forEach((callback) => callback(data as T));
    }

    const onceCallbacks = this.onceEvents[event];
    if (onceCallbacks) {
      onceCallbacks.forEach((callback) => callback(data as T));
      delete this.onceEvents[event];
    }
  }

  once<T = any>(event: string, callback: EventCallback<T>): EventUnsubscribe {
    if (!this.onceEvents[event]) {
      this.onceEvents[event] = new Set();
    }
    this.onceEvents[event].add(callback);

    return () => {
      if (this.onceEvents[event]) {
        this.onceEvents[event].delete(callback);
      }
    };
  }
}

export const eventBus: EventBus = new SimpleEventBus();

export async function subscribeToK8sEvents(
  clusterId: string,
  namespace: string,
  resourceType: string,
  callback: EventCallback<any>
): Promise<EventUnsubscribe> {
  try {
    const unsubscribeId = await invoke<string>("subscribe_to_k8s_events", {
      clusterId,
      namespace,
      resourceType,
    });

    const handler = (data: any) => {
      callback(data);
    };

    eventBus.on(`k8s:${clusterId}:${namespace}:${resourceType}`, handler);

    return () => {
      eventBus.off(`k8s:${clusterId}:${namespace}:${resourceType}`, handler);
      invoke<void>("unsubscribe_from_k8s_events", { unsubscribeId });
    };
  } catch (error) {
    console.error("Failed to subscribe to K8s events:", error);
    return () => {};
  }
}

export async function subscribeToAllEvents(
  clusterId: string,
  callback: EventCallback<any>
): Promise<EventUnsubscribe> {
  try {
    const unsubscribeId = await invoke<string>("subscribe_to_all_k8s_events", {
      clusterId,
    });

    const handler = (data: any) => {
      callback(data);
    };

    eventBus.on(`k8s:${clusterId}:all`, handler);

    return () => {
      eventBus.off(`k8s:${clusterId}:all`, handler);
      invoke<void>("unsubscribe_from_k8s_events", { unsubscribeId });
    };
  } catch (error) {
    console.error("Failed to subscribe to all K8s events:", error);
    return () => {};
  }
}
