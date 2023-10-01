import { Component, createRef } from "preact";
import { allSubjects, type Subject } from "../scripts/faculties";
import { signal, type Signal } from "@preact/signals";

const subjects = await allSubjects(import.meta.env.SITE + "/api/faculties");
const subject: Signal<Subject | undefined> = signal(undefined);

function updateSubject(selectEl: HTMLSelectElement) {
  const newSubject = subjects.find(
    (subject) => subject.id === selectEl.selectedOptions[0].id,
  );
  subject.value = newSubject;
}

class SubjectSelect extends Component {
  ref = createRef<HTMLSelectElement>();

  componentDidMount() {
    this.ref.current && updateSubject(this.ref.current);
  }

  render() {
    return (
      <select
        ref={this.ref}
        name="subject"
        id="subject_select"
        class="w-full px-4 py-2 rounded-md text-white bg-slate-700"
        onChange={(event) => {
          updateSubject(event.currentTarget);
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
        {subject.value === undefined ? (
          <option disabled>Bitte wähle einen Studiengang</option>
        ) : (
          subject.value.groups
            .reverse()
            .map(({ id }) => <option id={id}>{id}</option>)
        )}
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
