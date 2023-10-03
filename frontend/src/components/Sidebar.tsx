import { Search } from "lucide-preact";
import Settings from "./Settings";
import { group, subjectDisplay } from "../scripts/state";
import { computed } from "@preact/signals";

const subjectName = computed(() => subjectDisplay.value.name);
const subjectGroupAndDegree = computed(
  () => `${subjectDisplay.value.degree}, ${group.value.split("(")[0]}`,
);

const backdropClassList = () => document.querySelector("#backdrop")?.classList;

export function showSidebar() {
  backdropClassList()?.replace("pointer-events-none", "pointer-events-auto");
  backdropClassList()?.replace("bg-black/0", "bg-black/30");
  document.querySelector("body > div")?.classList.add("translate-x-72");
}

export function hideSidebar() {
  document.querySelector("body > div")?.classList.remove("translate-x-72");
  backdropClassList()?.replace("bg-black/30", "bg-black/0");

  setTimeout(() => {
    backdropClassList()?.replace("pointer-events-auto", "pointer-events-none");
  }, 250);
}

window.addEventListener("resize", (event) => {
  if (window.innerWidth >= 1024) {
    hideSidebar();
  }
});

export function Overlay() {
  return (
    <aside class="absolute inset-0 -translate-x-72 w-72 z-50 bg-neutral-50 lg:static lg:z-auto lg:translate-x-0 flex flex-col">
      <button class="m-2 px-5 py-3 border border-neutral-300 rounded-md overflow-hidden flex items-center justify-between">
        <div class="max-w-full flex flex-col items-start gap-1 overflow-hidden text-start">
          <span class="text-lg leading-none font-bold text-ellipsis">
            {subjectName}
          </span>
          <span class="text-xs leading-none font-semibold text-neutral-700 bg-transparent">
            {subjectGroupAndDegree}
          </span>
        </div>
        {/* <Search size={26} class="flex-shrink-0" /> */}
      </button>

      <Settings />
    </aside>
  );
}

export function Backdrop() {
  return (
    <div
      id="backdrop"
      class={
        "absolute inset-0 cursor-pointer z-50 transition-colors duration-200 pointer-events-none bg-black/0"
      }
      onClick={() => hideSidebar()}
    ></div>
  );
}
