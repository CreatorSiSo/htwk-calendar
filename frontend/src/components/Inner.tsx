// TODO Temporary element, undo this when removing astro

import Calendar from "./Calendar";
import { Backdrop, Overlay } from "./Sidebar";

export function Inner() {
  return (
    <>
      <Overlay />

      <div class="flex-grow flex flex-col">
        <Backdrop />
        {/* 	<!-- <GroupSelection /> -->*/}
        <Calendar />
        <div
          class="z-50 max-w-sm top-0 left-0 px-4 py-3 absolute hidden shadow-md rounded-md bg-white text-sm"
          id="popover"
        >
          <div class="flex gap-2 justify-between">
            <span id="title" class="font-bold">
              Title
            </span>
            <button id="close_btn" class="w-5 h-5">
              ✕
            </button>
          </div>
          <div id="descr"></div>

          <div class="pt-2">
            <div id="rooms"></div>
            <div id="staff"></div>
            <div id="kind"></div>
            <div id="time"></div>
          </div>
        </div>
      </div>
    </>
  );
}
