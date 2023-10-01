import { Calendar, type EventApi } from "@fullcalendar/core";
import dayGridMonth from "@fullcalendar/daygrid";
import multiMonthPlugin from "@fullcalendar/multimonth";
import timeGridPlugin from "@fullcalendar/timegrid";
import de from "@fullcalendar/core/locales/de";
import {
  autoUpdate,
  computePosition,
  flip,
  offset,
  shift,
} from "@floating-ui/dom";

function formatTwoDigits(number: number): string {
  if (number < 10) {
    return "0" + number.toString();
  } else {
    return number.toString();
  }
}

const calendar_el = document.getElementById("calendar") as HTMLElement;
const popover_el = document.getElementById("popover") as HTMLDivElement;

function showPopover({ title, start, end, extendedProps }: EventApi) {
  popover_el.classList.remove("hidden");

  const title_el = popover_el.querySelector("#title") as HTMLDivElement;
  const desc_el = popover_el.querySelector("#descr") as HTMLDivElement;
  const rooms_el = popover_el.querySelector("#rooms") as HTMLDivElement;
  const kind_el = popover_el.querySelector("#kind") as HTMLDivElement;
  const time_el = popover_el.querySelector("#time") as HTMLDivElement;

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
    ? `${formatTwoDigits(end.getHours())}:${formatTwoDigits(end.getMinutes())}`
    : "Unbekannt";
  time_el.textContent = `${startString} - ${endString}`;
}
let cleanupPopover: () => void = () => {};
function hidePopover() {
  popover_el.classList.add("hidden");
  cleanupPopover();
}

popover_el
  .querySelector("#close_btn")
  ?.addEventListener("click", () => hidePopover());

const calendar = new Calendar(calendar_el, {
  plugins: [dayGridMonth, timeGridPlugin, multiMonthPlugin],
  height: "100%",
  initialView: "multiMonthYear",
  headerToolbar: {
    left: "multiMonthYear,dayGridMonth,timeGridWeek,timeGridDay",
    center: "title",
    right: "today prev,next",
  },
  locales: [de],
  locale: "de",
  eventClick: function ({ event, el }) {
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

    onElementRemoved(el, () => hidePopover());
    cleanupPopover();
    showPopover(event);

    cleanupPopover = autoUpdate(el, popover_el, () =>
      computePosition(el, popover_el, {
        placement: "top",
        middleware: [offset(10), flip(), shift({ padding: 10 })],
      }).then(({ x, y }) => {
        Object.assign(popover_el.style, {
          left: `${x}px`,
          top: `${y}px`,
        });
      }),
    );
  },
  eventContent: ({ event, view }) => {
    if (view.type === "multiMonthYear") return true;

    return {
      html: `<div class="flex flex-col box-border h-full max-w-full"><div class="flex-shrink-0 font-semibold truncate">${event.title}</div><div class="truncate">${event.extendedProps.notes}</div></div>`,
    };
  },
});

const defaultNext = calendar.next.bind(calendar);
calendar.next = () => {
  hidePopover();
  defaultNext();
};

const defaultPrev = calendar.prev.bind(calendar);
calendar.prev = () => {
  hidePopover();
  defaultPrev();
};

function setupView() {
  const view = document.querySelector(".fc-view") as HTMLElement;
  view?.addEventListener("", () => true);
  const classList = view?.classList;
  classList?.add("relative");
  classList?.add("overflow-hidden");
  view?.append(popover_el);
}

const defaultChangeView = calendar.changeView.bind(calendar);
calendar.changeView = (...args) => {
  hidePopover();
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

calendar.render();
