import { signal } from "@preact/signals";
import { allSubjects, type Subject } from "./faculties";

export const subjects = import.meta.env.SSR
  ? []
  : await allSubjects(import.meta.env.SITE + "/api/subjects");
export const subjectsMap = new Map<string, Subject>(
  subjects.map((subject) => [subject.id, subject]),
);