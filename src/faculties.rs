use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Study {
	#[serde(default, rename = "$value")]
	pub faculties: Vec<Faculty>,
}

#[derive(Debug, Deserialize)]
pub struct Faculty {
	#[serde(rename = "@id")]
	id: String,
	#[serde(rename = "@name")]
	name: String,
	#[serde(default, rename = "$value")]
	subjects: Vec<Subject>,
}

#[derive(Debug, Deserialize)]
pub struct Subject {
	#[serde(rename = "@id")]
	id: String,
	#[serde(rename = "@name")]
	name: String,
}
