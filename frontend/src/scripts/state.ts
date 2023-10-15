import { signal, computed, effect } from "@preact/signals";
import { allSubjects } from "./faculties";
import type { RefObject } from "preact";
import type FullCalendar from "@fullcalendar/react";
import { paramStore } from "./utils";

export const calendarRef = signal<RefObject<FullCalendar>>({ current: null });

const group = paramStore("group", "23ARB-4");

export const getGroup = () => group.value;
export const setGroup = (value: string) => (group.value = value);

effect(() => {
  console.log("group effect", group.value);
  const calendar = calendarRef.value.current?.getApi();
  if (calendar === undefined || group.value === undefined) {
    console.error("calendar", calendar, "group", group.value);
    return;
  }

  calendar.removeAllEventSources();
  // TODO This causes the 404 responses
  const url = `${import.meta.env.SITE}/api/events/${group.value}`;
  console.log(url);

  calendar.addEventSource({
    url,
    extraParams: ["notes", "type", "type_display", "rooms"],
  });
  calendar.refetchEvents();
});

export const subjects = (
  await allSubjects(import.meta.env.SITE + "/api/subjects")
).map((subject) => ({
  ...subject,
  groups: subject.groups.toReversed().map((localGroup) => ({
    id: localGroup.id,
    selected: computed(() => group.value === localGroup.id),
  })),
}));

export const subjectsMap = new Map(
  subjects.map((subject) => [subject.id, subject]),
);
