import {
  Component,
  createRef,
  type FunctionComponent,
  type JSX,
  type RefObject,
} from "preact";
import { signal } from "@preact/signals";
import { ChevronLeft, ChevronRight, Menu, Square } from "lucide-preact";
import {
  autoUpdate,
  computePosition,
  offset,
  flip,
  shift,
} from "@floating-ui/dom";

import type { EventApi, CalendarOptions } from "@fullcalendar/core";
import FullCalendar from "@fullcalendar/react";
import de from "@fullcalendar/core/locales/de";

import listPlugin from "@fullcalendar/list";
import dayGridPlugin from "@fullcalendar/daygrid";
import multiMonthPlugin from "@fullcalendar/multimonth";
import timeGridPlugin from "@fullcalendar/timegrid";

import { calendarRef } from "../scripts/state";
import { IconButton } from "./Buttons";
import { showSidebar } from "./Sidebar";

type EventClickFn = CalendarOptions["eventClick"];
type EventContentFn = CalendarOptions["eventContent"];

function formatTwoDigits(number: number): string {
  if (number < 10) {
    return "0" + number.toString();
  } else {
    return number.toString();
  }
}

const headerTitle = signal("...");
const CalendarHeader: FunctionComponent<{
  calendar: RefObject<FullCalendar>;
}> = ({ calendar }) => (
  <nav class="flex justify-between items-center px-2">
    <IconButton class="lg:hidden" onClick={() => showSidebar()}>
      <Menu size={26} />
    </IconButton>
    <span class="text-lg font-bold">{headerTitle}</span>
    <div class="flex text-white">
      {/* TODO Add search function */}
      {/* <SquareButton>
          <Search size={24} />
        </SquareButton> */}
      <IconButton
        class="relative"
        onClick={() => calendar.current?.getApi().today()}
      >
        <span class="text-xs font-mono font-black absolute">
          {new Date().getDay() + 1}
        </span>
        <Square size={26} />
      </IconButton>
      <IconButton onClick={() => calendar.current?.getApi().prev()}>
        <ChevronLeft size={26} />
      </IconButton>
      <IconButton onClick={() => calendar.current?.getApi().next()}>
        <ChevronRight size={26} />
      </IconButton>
    </div>
  </nav>
);

function eventToStrings({ title, start, end, extendedProps }: EventApi) {
  const startString = start
    ? `${formatTwoDigits(start.getHours())}:${formatTwoDigits(
        start.getMinutes(),
      )}`
    : "Unbekannt";
  const endString = end
    ? `${formatTwoDigits(end.getHours())}:${formatTwoDigits(end.getMinutes())}`
    : "Unbekannt";

  return {
    title,
    description: extendedProps.notes ?? "",
    rooms: extendedProps.rooms?.join(", ") ?? "",
    kind: extendedProps.kind_display ?? "Unbekannter Event Typ",
    time: `${startString} - ${endString}`,
  };
}

export default class Calendar extends Component {
  state = {};
  calendarRef = createRef<FullCalendar>();

  // TODO This is cursed but I want to see a runtime error if something fails, not have to write tons of checks and make typescript happy
  popover_el: HTMLDivElement = undefined as any as HTMLDivElement;

  showPopover(event: EventApi) {
    this.popover_el.classList.remove("hidden");

    const title_el = this.popover_el.querySelector("#title") as HTMLDivElement;
    const desc_el = this.popover_el.querySelector("#descr") as HTMLDivElement;
    const rooms_el = this.popover_el.querySelector("#rooms") as HTMLDivElement;
    const kind_el = this.popover_el.querySelector("#kind") as HTMLDivElement;
    const time_el = this.popover_el.querySelector("#time") as HTMLDivElement;

    const { title, description, kind, rooms, time } = eventToStrings(event);

    title_el.textContent = title;
    desc_el.textContent = description;
    rooms_el.textContent = rooms;
    kind_el.textContent = kind;
    time_el.textContent = time;
  }
  cleanupPopover: () => void = () => {};
  hidePopover() {
    this.popover_el.classList.add("hidden");
    this.cleanupPopover();
  }

  componentDidMount() {
    this.popover_el = document.getElementById("popover") as HTMLDivElement;
    this.popover_el
      ?.querySelector("#close_btn")
      ?.addEventListener("click", () => this.hidePopover());

    calendarRef.value = this.calendarRef;
  }

  eventClick: EventClickFn = ({ event, el }) => {
    function onElementRemoved(element: Element, callback: () => void) {
      new MutationObserver(function (mutations, observer) {
        if (!document.body.contains(element)) {
          callback();
          observer.disconnect();
        }
      }).observe(
        element.parentElement?.parentElement?.parentElement
          ?.parentElement as Node,
        {
          childList: true,
        },
      );
    }

    onElementRemoved(el, () => this.hidePopover());
    this.cleanupPopover();
    this.showPopover(event);

    this.cleanupPopover = autoUpdate(el, this.popover_el, () =>
      computePosition(el, this.popover_el, {
        placement: "top",
        middleware: [offset(10), flip(), shift({ padding: 10 })],
      }).then(({ x, y }) => {
        Object.assign(this.popover_el.style, {
          left: `${x}px`,
          top: `${y}px`,
        });
      }),
    );
  };

  eventContent: EventContentFn = ({ event, view }) => {
    const { title, description, kind, rooms } = eventToStrings(event);

    switch (view.type) {
      case "multiMonthYear":
      case "multiMonth":
        return true;

      case "dayGridYear":
      case "dayGridMonth":
      case "dayGridWeek":
        return true;

      case "listYear":
      case "listMonth":
      case "listWeek":
      case "listDay":
        return (
          <>
            <div class="font-semibold">{title}</div>
            <div>{description}</div>
            <div class="mt-2">{kind}</div>
            <div>{rooms}</div>
          </>
        );

      default:
        return (
          <div class="flex flex-col box-border h-full max-w-full">
            <div class="flex-shrink-0 font-semibold truncate">{title}</div>
            <div class="truncate">{description}</div>
          </div>
        );
    }
  };

  render() {
    const options: CalendarOptions = {
      height: "100%",
      locales: [de],
      locale: "de",
      timeZone: "none",
      // weekends: false,
      allDaySlot: false,
      headerToolbar: false,

      // plugins,
      plugins: [listPlugin, dayGridPlugin, timeGridPlugin, multiMonthPlugin],

      // initialView: "listYear", // yes
      // initialView: "multiMonthYear", // yes (slow)
      // initialView: "dayGridYear", // no

      initialView: "dayGridMonth", // yes (default)

      // initialView: "dayGridWeek", // maybe
      // initialView: "timeGridWeek", // yes

      // initialView: "timeGridDay", // yes

      eventClick: this.eventClick,
      eventContent: this.eventContent,

      eventsSet: () => {
        headerTitle.value =
          this.calendarRef.current?.getApi().view.title ?? "...";
      },
    };

    return (
      <div class="h-full flex flex-col">
        <CalendarHeader calendar={this.calendarRef} />
        <FullCalendar ref={this.calendarRef} {...options} />
      </div>
    );
  }
}
