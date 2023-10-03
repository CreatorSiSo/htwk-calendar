import { Component, createRef } from "preact";
import {
  calendarRef,
  group,
  subject,
  subjects,
  subjectsMap,
  toggleSidebar,
  hideSidebar,
} from "../scripts/state";
import { signal } from "@preact/signals";
import { Calendar, CalendarDays, CalendarRange } from "lucide-preact";

function getAndSetActiveSubject(selectEl: HTMLSelectElement) {
  const option = selectEl.selectedOptions[0];
  subject.value = subjectsMap.get(option.id);
}

// TODO Toggle sidebar when resizing window

export default function Settings() {
  return (
    <div class="h-full p-4 flex flex-col justify-between text-neutral-800">
      <div class="flex flex-col gap-4">
        <div class="flex flex-col">
          <label for="subject_select" class="mb-1">
            Studiengang
          </label>
          <SubjectSelect />
        </div>

        <div class="flex flex-col">
          <label for="group_select" class="mb-1">
            Seminargruppe
          </label>
          <GroupSelect />
        </div>
      </div>

      <ViewSwitch />
    </div>
  );
}

const viewSignal = signal("dayGridMonth");
const viewsData = [
  ["listYear", ["Liste", CalendarList]],
  ["multiMonthYear", ["Jahr", CalendarYear]],
  ["dayGridMonth", ["Monat", CalendarMonth]],
  ["timeGridWeek", ["Woche", CalendarWeek]],
  ["timeGridDay", ["Tag", CalendarDay]],
] as const;
const setView = (view: string) => {
  viewSignal.value = view;
  console.debug(`Changing view to '${viewSignal.value}'`);
  calendarRef.value.current?.getApi().changeView(viewSignal.value);
};
function ViewSwitch() {
  return (
    <nav class="flex flex-col">
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

class SubjectSelect extends Component {
  ref = createRef<HTMLSelectElement>();

  componentDidMount() {
    this.ref.current && getAndSetActiveSubject(this.ref.current);
  }

  render() {
    return (
      <select
        ref={this.ref}
        name="subject"
        id="subject_select"
        class="w-full px-4 py-2 rounded-md bg-neutral-300 font-semibold"
        onChange={({ currentTarget }) => getAndSetActiveSubject(currentTarget)}
      >
        {subjects.map(({ name, id }) => (
          <option id={id}>{name}</option>
        ))}
      </select>
    );
  }
}

class GroupSelect extends Component {
  ref = createRef<HTMLSelectElement>();

  componentDidUpdate() {
    this.ref.current?.dispatchEvent(new Event("change"));
  }

  render() {
    return (
      <select
        ref={this.ref}
        name="group"
        id="group_select"
        class="w-full px-4 py-2 rounded-md bg-neutral-300 font-semibold"
        onChange={({ currentTarget }) => {
          group.value = currentTarget.selectedOptions[0].id;
        }}
      >
        {subject.value?.groups.map(({ id }) => <option id={id}>{id}</option>)}
      </select>
    );
  }
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
