import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useFavorites } from "./useFavorites";

describe("useFavorites", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  afterEach(() => {
    localStorage.clear();
  });

  it("initializes with empty favorites", () => {
    const { result } = renderHook(() => useFavorites());
    expect(result.current.favorites).toEqual([]);
  });

  it("toggles favorite on/off", () => {
    const { result } = renderHook(() => useFavorites());
    const resource = {
      id: "pod-1",
      type: "pod",
      name: "test-pod",
      namespace: "default",
      clusterId: "cluster-1",
    };

    act(() => {
      result.current.toggleFavorite(resource);
    });
    expect(result.current.isFavorite("pod-1")).toBe(true);
    expect(result.current.favorites).toHaveLength(1);

    act(() => {
      result.current.toggleFavorite(resource);
    });
    expect(result.current.isFavorite("pod-1")).toBe(false);
    expect(result.current.favorites).toHaveLength(0);
  });

  it("persists favorites to localStorage", () => {
    const { result } = renderHook(() => useFavorites());
    const resource = {
      id: "pod-1",
      type: "pod",
      name: "test-pod",
      namespace: "default",
      clusterId: "cluster-1",
    };

    act(() => {
      result.current.toggleFavorite(resource);
    });

    const stored = localStorage.getItem("tftsr-favorites");
    expect(stored).toBeTruthy();
    const parsed = JSON.parse(stored!);
    expect(parsed).toHaveLength(1);
    expect(parsed[0].id).toBe("pod-1");
  });

  it("loads favorites from localStorage on init", () => {
    const favorites = [
      {
        id: "pod-1",
        type: "pod",
        name: "test-pod",
        namespace: "default",
        clusterId: "cluster-1",
        timestamp: Date.now(),
      },
    ];
    localStorage.setItem("tftsr-favorites", JSON.stringify(favorites));

    const { result } = renderHook(() => useFavorites());
    expect(result.current.favorites).toHaveLength(1);
    expect(result.current.isFavorite("pod-1")).toBe(true);
  });

  it("filters favorites by type", () => {
    const { result } = renderHook(() => useFavorites());
    act(() => {
      result.current.toggleFavorite({
        id: "pod-1",
        type: "pod",
        name: "test-pod",
        namespace: "default",
        clusterId: "cluster-1",
      });
      result.current.toggleFavorite({
        id: "svc-1",
        type: "service",
        name: "test-service",
        namespace: "default",
        clusterId: "cluster-1",
      });
    });

    const pods = result.current.getFavoritesByType("pod");
    expect(pods).toHaveLength(1);
    expect(pods[0].id).toBe("pod-1");
  });

  it("filters favorites by cluster", () => {
    const { result } = renderHook(() => useFavorites());
    act(() => {
      result.current.toggleFavorite({
        id: "pod-1",
        type: "pod",
        name: "test-pod",
        namespace: "default",
        clusterId: "cluster-1",
      });
      result.current.toggleFavorite({
        id: "pod-2",
        type: "pod",
        name: "test-pod-2",
        namespace: "default",
        clusterId: "cluster-2",
      });
    });

    const cluster1Favs = result.current.getFavoritesByCluster("cluster-1");
    expect(cluster1Favs).toHaveLength(1);
    expect(cluster1Favs[0].id).toBe("pod-1");
  });

  it("removes favorite by id", () => {
    const { result } = renderHook(() => useFavorites());
    act(() => {
      result.current.toggleFavorite({
        id: "pod-1",
        type: "pod",
        name: "test-pod",
        namespace: "default",
        clusterId: "cluster-1",
      });
    });
    expect(result.current.isFavorite("pod-1")).toBe(true);

    act(() => {
      result.current.removeFavorite("pod-1");
    });
    expect(result.current.isFavorite("pod-1")).toBe(false);
  });

  it("clears all favorites", () => {
    const { result } = renderHook(() => useFavorites());
    act(() => {
      result.current.toggleFavorite({
        id: "pod-1",
        type: "pod",
        name: "test-pod",
        namespace: "default",
        clusterId: "cluster-1",
      });
      result.current.toggleFavorite({
        id: "pod-2",
        type: "pod",
        name: "test-pod-2",
        namespace: "default",
        clusterId: "cluster-1",
      });
    });
    expect(result.current.favorites).toHaveLength(2);

    act(() => {
      result.current.clearFavorites();
    });
    expect(result.current.favorites).toHaveLength(0);
  });

  it("handles corrupted localStorage gracefully", () => {
    localStorage.setItem("tftsr-favorites", "invalid json{");
    const { result } = renderHook(() => useFavorites());
    expect(result.current.favorites).toEqual([]);
  });
});
