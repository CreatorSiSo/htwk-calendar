export function setSearchParam(name: string, value: string) {
  const newParams = new URLSearchParams(window.location.search);
  newParams.set(name, value);
  window.history.pushState(
    undefined,
    "",
    window.location.pathname + "?" + newParams.toString(),
  );
}

export function getSearchParam(name: string): string | null {
  return new URLSearchParams(window.location.search).get(name);
}
