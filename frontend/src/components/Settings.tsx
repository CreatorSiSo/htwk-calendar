import {
  calendarRef,
  getGroup,
  setGroup,
  subjects,
  subjectsMap,
} from "../scripts/state";
import { effect, computed } from "@preact/signals";
import { Calendar, CalendarDays, CalendarRange } from "lucide-preact";
import { hideSidebar } from "./Sidebar";
import { paramStore } from "../scripts/utils";

const subject = computed(() =>
  subjects.find((subject) =>
    subject.groups.some(({ selected }) => selected.value),
  ),
);

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

// TODO Toggle sidebar when resizing window
export default function Settings() {
  return (
    <div class="flex-grow pt-0 flex flex-col justify-between text-neutral-800">
      <div class="mx-2 flex flex-col gap-2">
        {/* TODO Combined search/select */}

        <select
          name="subject"
          class="p-3 rounded-md font-semibold bg-neutral-300"
          onChange={(event) => {
            const newSubjectId = event.currentTarget.selectedOptions[0].value;
            setGroup(subjectsMap.get(newSubjectId)?.groups[0].id);
          }}
        >
          {subjects.map((elem) => (
            <option value={elem.id} selected={elem.id === subject.value?.id}>
              {elem.name}
            </option>
          ))}
        </select>

        <select
          name="group"
          class="p-3 rounded-md font-semibold bg-neutral-300"
          onChange={(event) =>
            setGroup(event.currentTarget.selectedOptions[0].value)
          }
        >
          {subject.value?.groups.map((elem) => (
            <option value={elem.id} selected={elem.id === getGroup()}>
              {elem.id}
            </option>
          ))}
        </select>
      </div>

      <ViewSelect />
    </div>
  );
}

const viewsData = [
  ["listYear", ["Liste", CalendarList]],
  ["multiMonthYear", ["Jahr", CalendarYear]],
  ["dayGridMonth", ["Monat", CalendarMonth]],
  ["timeGridWeek", ["Woche", CalendarWeek]],
  ["timeGridDay", ["Tag", CalendarDay]],
] as const;
const viewSignal = paramStore("view", "dayGridMonth");
effect(() => {
  calendarRef.value.current?.getApi().changeView(viewSignal.value);
});

const setView = (view: string) => {
  viewSignal.value = view;
  console.debug(`Changing view to '${viewSignal.value}'`);
};

function ViewSelect() {
  return (
    <nav class="p-4 flex flex-col">
      {viewsData.map(([viewId, [name, CalendarIcon]]) => (
        <button
          class={
            "p-3 text-start rounded-md font-semibold flex gap-3 " +
            (viewSignal.value === viewId ? "bg-neutral-300" : "")
          }
          onClick={() => {
            hideSidebar();
            setView(viewId);
          }}
        >
          <CalendarIcon />
          {name}
        </button>
      ))}
    </nav>
  );
}

function CalendarYear() {
  return <CalendarDays class="shrink-0" />;
}

function CalendarMonth() {
  return <CalendarRange class="shrink-0" />;
}

function CalendarWeek() {
  return (
    <div class="relative">
      <Calendar />
      <div class="absolute top-[13px] left-[5px] h-[2px] w-[13px] rounded-sm bg-current"></div>
    </div>
  );
}

function CalendarDay() {
  return (
    <div class="relative">
      <Calendar />
      <div class="absolute top-[13px] left-[6px] h-[2px] w-[3px] rounded-full bg-current"></div>
    </div>
  );
}

function CalendarList() {
  return (
    <div class="relative">
      <Calendar />
      <div class="absolute top-[12.6px] left-[6px] h-[2px] w-[3px] rounded-full bg-current"></div>
      <div class="absolute top-[16.5px] left-[6px] h-[2px] w-[3px] rounded-full bg-current"></div>

      <div class="absolute top-[12.6px] left-[11px] h-[2px] w-[7px] rounded-sm bg-current"></div>
      <div class="absolute top-[16.5px] left-[11px] h-[2px] w-[7px] rounded-sm bg-current"></div>
    </div>
  );
}
