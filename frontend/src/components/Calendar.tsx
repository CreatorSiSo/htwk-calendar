import {
  Component,
  createRef,
  type FunctionComponent,
  type JSX,
  type RefObject,
} from "preact";
import { signal } from "@preact/signals";

import type {
  EventApi,
  CalendarApi,
  CalendarOptions,
} from "@fullcalendar/core";
import FullCalendar from "@fullcalendar/react";
import de from "@fullcalendar/core/locales/de";
import dayGridMonth from "@fullcalendar/daygrid";
import multiMonthPlugin from "@fullcalendar/multimonth";
import timeGridPlugin from "@fullcalendar/timegrid";

import { ChevronLeft, ChevronRight, Menu, Square } from "lucide-preact";

import {
  autoUpdate,
  computePosition,
  offset,
  flip,
  shift,
} from "@floating-ui/dom";
import { group, subject, calendarRef, toggleSidebar } from "../scripts/state";

type EventClickFn = CalendarOptions["eventClick"];
type EventContentFn = CalendarOptions["eventContent"];

function formatTwoDigits(number: number): string {
  if (number < 10) {
    return "0" + number.toString();
  } else {
    return number.toString();
  }
}

const Button: FunctionComponent<JSX.HTMLAttributes<HTMLButtonElement>> = ({
  children,
  class: classes,
  ...props
}) => (
  <button
    class={
      "h-11 w-9 flex items-center justify-center text-neutral-800 " + classes
    }
    {...props}
  >
    {children}
  </button>
);

const title = signal("...");
const CalendarHeader: FunctionComponent<{
  calendar: RefObject<FullCalendar>;
}> = ({ calendar }) => (
  <nav class="flex justify-between items-center px-2">
    <Button class="md:hidden" onClick={() => toggleSidebar()}>
      <Menu size={26} />
    </Button>
    <span class="text-lg font-bold">{title}</span>
    <div class="flex text-white">
      {/* TODO Add search function */}
      {/* <Button>
          <Search size={24} />
        </Button> */}
      <Button
        class="relative"
        onClick={() => calendar.current?.getApi().today()}
      >
        <span class="text-xs font-mono font-black absolute">
          {new Date().getDay() + 1}
        </span>
        <Square size={26} />
      </Button>
      <Button onClick={() => calendar.current?.getApi().prev()}>
        <ChevronLeft size={26} />
      </Button>
      <Button onClick={() => calendar.current?.getApi().next()}>
        <ChevronRight size={26} />
      </Button>
    </div>
  </nav>
);

export default class Calendar extends Component {
  state = {};
  calendarRef = createRef<FullCalendar>();

  // TODO This is cursed but I want to see a runtime error if something fails, not have to write tons of checks and make typescript happy
  popover_el: HTMLDivElement = undefined as any as HTMLDivElement;

  showPopover({ title, start, end, extendedProps }: EventApi) {
    this.popover_el.classList.remove("hidden");

    const title_el = this.popover_el.querySelector("#title") as HTMLDivElement;
    const desc_el = this.popover_el.querySelector("#descr") as HTMLDivElement;
    const rooms_el = this.popover_el.querySelector("#rooms") as HTMLDivElement;
    const kind_el = this.popover_el.querySelector("#kind") as HTMLDivElement;
    const time_el = this.popover_el.querySelector("#time") as HTMLDivElement;

    title_el.textContent = title;
    desc_el.textContent = extendedProps.notes ?? "";
    rooms_el.textContent = (extendedProps.rooms ?? []).join(", ");
    kind_el.textContent = extendedProps.kind_display ?? "Unbekannter Event Typ";

    const startString = start
      ? `${formatTwoDigits(start.getHours())}:${formatTwoDigits(
          start.getMinutes(),
        )}`
      : "Unbekannt";
    const endString = end
      ? `${formatTwoDigits(end.getHours())}:${formatTwoDigits(
          end.getMinutes(),
        )}`
      : "Unbekannt";
    time_el.textContent = `${startString} - ${endString}`;
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
    if (view.type === "multiMonthYear" || view.type === "dayGridMonth")
      return true;

    // this.calendarRef.current?.getApi().setOption("plugins", )

    return (
      <div class="flex flex-col box-border h-full max-w-full">
        <div class="flex-shrink-0 font-semibold truncate">{event.title}</div>
        <div class="truncate">{event.extendedProps.notes}</div>
      </div>
    );
  };

  render() {
    const options: CalendarOptions = {
      height: "100%",
      locales: [de],
      locale: "de",
      timeZone: "none",
      // weekends: false,
      // allDaySlot: false,

      plugins: [dayGridMonth, timeGridPlugin, multiMonthPlugin],
      // initialView: "multiMonthYear",
      initialView: "dayGridMonth",
      // initialView: "timeGridWeek",
      // initialView: "timeGridDay",
      headerToolbar: false,

      eventClick: this.eventClick,
      eventContent: this.eventContent,

      eventsSet: () => {
        title.value = this.calendarRef.current?.getApi().view.title ?? "...";
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
