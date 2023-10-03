import Settings from "./Settings";

export default function Overlay() {
  return (
    <aside class="absolute inset-0 -translate-x-80 w-80 z-50 bg-neutral-50 md:static md:z-auto md:translate-x-0">
      <Settings />
    </aside>
  );
}
