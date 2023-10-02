export type Faculty = {
  id: string;
  name: string;
  subjects: Subject[];
};

export type Subject = {
  id: string;
  name: string;
  groups: Group[];
};

export type Group = {
  id: string;
};

export async function allFaculties(url: string) {
  const res = await fetch(url);
  const faculties: Faculty[] = await res.json();
  return faculties;
}

export async function allSubjects(url: string) {
  const res = await fetch(url);
  const faculties: Subject[] = await res.json();
  return faculties;
}
