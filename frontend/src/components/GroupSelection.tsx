import { Component, createRef } from "preact";
import { allSubjects, type Subject } from "../scripts/faculties";
import { signal, type Signal } from "@preact/signals";
import { subjects, subjectsMap } from "../scripts/state";

const subject: Signal<Subject | undefined> = signal(undefined);

function getSubjectId(selectEl: HTMLSelectElement) {
  const index = selectEl.selectedIndex;
  const option = selectEl.options.item(index);
  return option?.id ?? "";
}

function updateSubject(id: string) {
  subject.value = subjectsMap.get(id);
}

class SubjectSelect extends Component {
  ref = createRef<HTMLSelectElement>();

  componentDidMount() {
    this.ref.current && updateSubject(getSubjectId(this.ref.current));
  }

  render() {
    return (
      <select
        ref={this.ref}
        name="subject"
        id="subject_select"
        class="w-full px-4 py-2 rounded-md text-white bg-slate-700 "
        onChange={(event) => {
          updateSubject(getSubjectId(event.currentTarget));
        }}
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
        class="w-full px-4 py-2 rounded-md text-white bg-slate-700"
      >
        {subject.value?.groups
          .reverse()
          .map(({ id }) => <option id={id}>{id}</option>)}
      </select>
    );
  }
}

export default () => (
  <div class="p-2 grid grid-cols-[auto_1fr] gap-2 md:flex md:gap-4">
    <label for="subject_select">Studiengang</label>
    <SubjectSelect />

    <label for="group_select" class="text-center align-middle">
      Seminargruppe
    </label>
    <GroupSelect />
  </div>
);
