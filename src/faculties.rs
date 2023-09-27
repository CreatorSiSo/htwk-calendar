use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Study {
	#[serde(default, rename = "$value")]
	pub faculties: Vec<Faculty>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Faculty {
	#[serde(rename = "@id")]
	pub id: String,
	#[serde(rename = "@name")]
	pub name: String,
	#[serde(default, rename = "$value")]
	pub subjects: Vec<Subject>,
}

impl PartialEq for Faculty {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}

#[derive(Debug, Deserialize, Clone)]
pub struct Subject {
	#[serde(rename = "@id")]
	pub id: String,
	#[serde(rename = "@name")]
	pub name: String,
	#[serde(default, rename = "$value")]
	pub groups: Vec<Group>,
}

impl PartialEq for Subject {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}

#[derive(Debug, Deserialize, Clone)]
pub struct Group {
	#[serde(rename = "@id")]
	pub id: String,
}
