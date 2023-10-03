import { signal, computed, effect } from "@preact/signals";
import { allSubjects, type Subject } from "./faculties";
import type { RefObject } from "preact";
import type FullCalendar from "@fullcalendar/react";
import { useEffect } from "preact/hooks";

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
    return {
      long: "Unbekannt",
      name: "Unbekannt",
      degree: "Unbekannt",
    };
  }

  const long = subject.value.name;
  const [name, degree] = long.split("(", 2);
  return {
    long,
    name: name.trim(),
    degree: degree.replace(")", "").trim(),
  };
});

export const group = signal<string | undefined>(undefined);
effect(() => {
  const calendar = calendarRef.value.current?.getApi();
  if (!calendar) return;

  calendar.removeAllEventSources();
  // TODO This causes the 404 responses
  const url = `${import.meta.env.SITE}/api/events/${group.value ?? ""}`;
  console.log(url);

  calendar.addEventSource({
    url,
    extraParams: ["notes", "type", "type_display", "rooms"],
  });
  calendar.refetchEvents();
});
