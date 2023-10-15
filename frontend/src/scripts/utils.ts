import { type Signal, signal, effect } from "@preact/signals";

export function setSearchParam(name: string, value: string) {
  const newParams = new URLSearchParams(window.location.search);
  newParams.set(name, value);
  window.history.pushState(
    undefined,
    "",
    window.location.pathname + "?" + newParams.toString(),
  );
}

export function getSearchParam(name: string) {
  const maybeValue = new URLSearchParams(window.location.search).get(name);
  if (maybeValue === null) return undefined;
  return maybeValue;
}

export function paramStore(name: string, fallback: string): Signal<string> {
  const store = signal(
    getSearchParam(name) ?? window.localStorage.getItem(name) ?? fallback,
  );
  effect(() => {
    setSearchParam(name, store.value);
    window.localStorage.setItem(name, store.value);
  });
  return store;
}
