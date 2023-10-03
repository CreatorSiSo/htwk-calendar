import { subjectDisplay } from "../scripts/state";
import { Menu } from "lucide-preact";

export default () => (
  <button class="m-2 px-2 py-3 border border-neutral-300 rounded-md gap-2 overflow-hidden flex items-center sm:hidden">
    <Menu size={26} class="flex-shrink-0" />
    <div class="flex flex-col items-start gap-1">
      <span class="text-lg leading-none font-bold truncate">
        {subjectDisplay.value.name}
      </span>
      <span class="text-xs leading-none font-semibold text-neutral-700 bg-transparent">
        {subjectDisplay.value.degree}
      </span>
    </div>
  </button>
);
