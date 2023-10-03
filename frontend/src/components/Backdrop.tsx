import { getSidebarVisible, toggleSidebar } from "../scripts/state";

export default function Backdrop() {
  return (
    <div
      class={
        getSidebarVisible()
          ? "absolute inset-0 bg-black/30 z-50 cursor-pointer"
          : "hidden"
      }
      onClick={() => toggleSidebar()}
    ></div>
  );
}
