import { Calendar } from "@fullcalendar/core";
import dayGridMonth from "@fullcalendar/daygrid";
import multiMonthPlugin from "@fullcalendar/multimonth";
import timeGridPlugin from "@fullcalendar/timegrid";

document.addEventListener("DOMContentLoaded", function () {
  const calendar_el = document.getElementById("calendar");
  const calendar = new Calendar(calendar_el, {
    plugins: [dayGridMonth, timeGridPlugin, multiMonthPlugin],
    initialView: "multiMonthYear",
    headerToolbar: {
      left: "multiMonthYear,dayGridMonth,timeGridDay",
      center: "title",
      right: "today prev,next",
    },
    events: [],
  });
  // calendar.addEvent();
  calendar.render();
});
