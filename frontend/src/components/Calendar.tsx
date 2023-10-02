import { Component, createRef } from "preact";
import { PureComponent } from "preact/compat";

import type {
  EventApi,
  CalendarApi,
  CalendarOptions,
} from "@fullcalendar/core";
import de from "@fullcalendar/core/locales/de";

import FullCalendar from "@fullcalendar/react";
import dayGridMonth from "@fullcalendar/daygrid";
import multiMonthPlugin from "@fullcalendar/multimonth";
import timeGridPlugin from "@fullcalendar/timegrid";

import {
  autoUpdate,
  computePosition,
  offset,
  flip,
  shift,
} from "@floating-ui/dom";

type EventClickFn = CalendarOptions["eventClick"];
type EventContentFn = CalendarOptions["eventContent"];

function formatTwoDigits(number: number): string {
  if (number < 10) {
    return "0" + number.toString();
  } else {
    return number.toString();
  }
}

export class Calendar extends Component {
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

    const calendar = this.calendarRef.current?.getApi() as CalendarApi;

    const defaultNext = calendar.next.bind(calendar);
    calendar.next = () => {
      this.hidePopover();
      defaultNext();
    };

    const defaultPrev = calendar.prev.bind(calendar);
    calendar.prev = () => {
      this.hidePopover();
      defaultPrev();
    };

    const outerThis = this;
    function setupView() {
      const view = document.querySelector(".fc-view") as HTMLElement;
      view?.addEventListener("", () => true);
      const classList = view?.classList;
      classList?.add("relative");
      classList?.add("overflow-hidden");
      view?.append(outerThis.popover_el);
    }

    const defaultChangeView = calendar.changeView.bind(calendar);
    calendar.changeView = (...args) => {
      this.hidePopover();
      defaultChangeView(...args);
      setupView();
    };

    const groupSelectEl = document.querySelector(
      "#group_select",
    ) as HTMLSelectElement;
    groupSelectEl.addEventListener("change", () => {
      calendar.removeAllEventSources();
      const url = `${import.meta.env.SITE}/api/events/${
        groupSelectEl.selectedOptions[0].value
      }`;
      console.log(url);
      calendar.addEventSource({
        url,
        extraParams: ["notes", "type", "type_display", "rooms"],
      });
      calendar.refetchEvents();
    });
  }

  eventClick: EventClickFn = ({ event, el }) => {
    function onElementRemoved(element: Element, callback: () => void) {
      new MutationObserver(function (mutations, observer) {
        console.log(document.body.contains(element));

        if (!document.body.contains(element)) {
          console.log("removed");
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

      plugins: [dayGridMonth, timeGridPlugin, multiMonthPlugin],
      initialView: "multiMonthYear",
      headerToolbar: {
        left: "multiMonthYear,dayGridMonth,timeGridWeek,timeGridDay",
        center: "title",
        right: "today prev,next",
      },

      eventClick: this.eventClick,
      eventContent: this.eventContent,
    };

    return <FullCalendar ref={this.calendarRef} {...options} />;
  }
}
