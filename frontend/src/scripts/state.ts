import { signal, computed, effect } from "@preact/signals";
import { allSubjects, type Subject } from "./faculties";
import type { RefObject } from "preact";
import type FullCalendar from "@fullcalendar/react";

export const calendarRef = signal<RefObject<FullCalendar>>({ current: null });

export const subjects = import.meta.env.SSR
  ? []
  : (await allSubjects(import.meta.env.SITE + "/api/subjects")).map(
      (subject) => ({ ...subject, groups: subject.groups.toReversed() }),
    );
export const subjectsMap = new Map<string, Subject>(
  subjects.map((subject) => [subject.id, subject]),
);

export const subject = signal<Subject | undefined>(undefined);
export const subjectDisplay = computed(() => {
  if (!subject.value) {
    return { name: "Unbekannt", degree: "Unbekannt" };
  }
  const long = subject.value.name;
  const [name, degree] = long.split("(", 2);

  return {
    long,
    name: name.trim(),
    degree: degree.replace(")", "").trim(),
  };
});

export const group = signal("");
effect(() => {
  const calendar = calendarRef.value.current?.getApi();
  if (!calendar) return;

  calendar.removeAllEventSources();
  const url = `${import.meta.env.SITE}/api/events/${group.value}`;
  console.log(url);

  calendar.addEventSource({
    url,
    extraParams: ["notes", "type", "type_display", "rooms"],
  });
  calendar.refetchEvents();
});

const sidebarVisible = signal(false);
export function getSidebarVisible() {
  return sidebarVisible.value;
}
export function toggleSidebar() {
  document.querySelector("body > div")?.classList.toggle("translate-x-80");
  sidebarVisible.value = !sidebarVisible.value;
}
export function hideSidebar() {
  document.querySelector("body > div")?.classList.remove("translate-x-80");
  sidebarVisible.value = false;
}
