import { allSubjects, type Subject } from "../scripts/faculties";
import { signal, type Signal } from "@preact/signals";

const subjects = await allSubjects("http://localhost:5000/faculties");
const subject: Signal<Subject | undefined> = signal(undefined);

export default function GroupSelection() {
	return (
		<div class="p-2 grid grid-cols-[auto_1fr] gap-2 md:flex md:gap-4">
			<label for="subject_select">Studiengang</label>
			<select
				name="subject"
				id="subject_select"
				class="w-full px-4 py-2 rounded-md text-white bg-slate-700"
				onChange={(event) => {
					const selectEl = event.currentTarget;
					const newSubject = subjects.find(
						(subject) => subject.id === selectEl.selectedOptions[0].id
					);
					subject.value = newSubject;
				}}
			>
				{subjects.map(({ name, id }) => (
					<option id={id}>{name}</option>
				))}
			</select>

			<label for="group_select" class="text-center align-middle">
				Seminargruppe
			</label>
			<select
				name="group"
				id="group_select"
				class="w-full px-4 py-2 rounded-md text-white bg-slate-700"
			>
				{subject.value === undefined ? (
					<option selected disabled>
						Bitte wähle einen Studiengang
					</option>
				) : (
					subject.value.groups.map((group) => <option>{group.id}</option>)
				)}
			</select>
		</div>
	);
}
