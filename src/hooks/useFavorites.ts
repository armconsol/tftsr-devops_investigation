import { useState, useEffect, useCallback } from "react";

interface FavoriteResource {
  id: string;
  type: string;
  name: string;
  namespace?: string;
  clusterId: string;
  timestamp: number;
}

const FAVORITES_KEY = "tftsr-favorites";

function loadFavorites(): FavoriteResource[] {
  try {
    const stored = localStorage.getItem(FAVORITES_KEY);
    return stored ? JSON.parse(stored) : [];
  } catch (err) {
    console.error("Failed to load favorites:", err);
    return [];
  }
}

function saveFavorites(favorites: FavoriteResource[]): void {
  try {
    localStorage.setItem(FAVORITES_KEY, JSON.stringify(favorites));
  } catch (err) {
    console.error("Failed to save favorites:", err);
  }
}

export function useFavorites() {
  const [favorites, setFavorites] = useState<FavoriteResource[]>(loadFavorites);

  useEffect(() => {
    saveFavorites(favorites);
  }, [favorites]);

  const isFavorite = useCallback(
    (resourceId: string): boolean => {
      return favorites.some((fav) => fav.id === resourceId);
    },
    [favorites]
  );

  const toggleFavorite = useCallback(
    (resource: Omit<FavoriteResource, "timestamp">): void => {
      setFavorites((prev) => {
        const exists = prev.find((fav) => fav.id === resource.id);
        if (exists) {
          return prev.filter((fav) => fav.id !== resource.id);
        }
        return [...prev, { ...resource, timestamp: Date.now() }];
      });
    },
    []
  );

  const removeFavorite = useCallback((resourceId: string): void => {
    setFavorites((prev) => prev.filter((fav) => fav.id !== resourceId));
  }, []);

  const clearFavorites = useCallback((): void => {
    setFavorites([]);
  }, []);

  const getFavoritesByType = useCallback(
    (type: string): FavoriteResource[] => {
      return favorites.filter((fav) => fav.type === type);
    },
    [favorites]
  );

  const getFavoritesByCluster = useCallback(
    (clusterId: string): FavoriteResource[] => {
      return favorites.filter((fav) => fav.clusterId === clusterId);
    },
    [favorites]
  );

  return {
    favorites,
    isFavorite,
    toggleFavorite,
    removeFavorite,
    clearFavorites,
    getFavoritesByType,
    getFavoritesByCluster,
  };
}
