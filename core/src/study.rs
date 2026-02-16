use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyEntryStart {
    pub fen: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyEntryExpected {
    pub fen: String,
    pub r#move: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyEntry {
    pub description: Option<String>,
    pub start: StudyEntryStart,
    pub expected: Vec<StudyEntryExpected>,
}

// Include the study data as a JSON string at compile time
const STUDY_CASTLING: &str = include_str!("../data/castling.json");
const STUDY_CHECKMATES: &str = include_str!("../data/checkmates.json");
const STUDY_FAMOUS: &str = include_str!("../data/famous.json");
const STUDY_PAWNS: &str = include_str!("../data/pawns.json");
const STUDY_PROMOTIONS: &str = include_str!("../data/promotions.json");
const STUDY_STALEMATES: &str = include_str!("../data/stalemates.json");
const STUDY_STANDARD: &str = include_str!("../data/standard.json");
const STUDY_TAXING: &str = include_str!("../data/taxing.json");

// Deserialize the JSON strings into vectors of StudyEntry
fn load_study_entries(json_data: &str) -> Vec<StudyEntry> {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct StudyData {
        description: Option<String>,
        #[serde(rename = "testCases")]
        test_cases: Vec<StudyEntry>,
    }

    let study_data: StudyData =
        serde_json::from_str(json_data).expect("Failed to parse study data");
    study_data.test_cases
}

// Public functions to access the study entries
pub fn get_castling_study() -> Vec<StudyEntry> {
    load_study_entries(STUDY_CASTLING)
}

pub fn get_checkmates_study() -> Vec<StudyEntry> {
    load_study_entries(STUDY_CHECKMATES)
}

pub fn get_famous_study() -> Vec<StudyEntry> {
    load_study_entries(STUDY_FAMOUS)
}

pub fn get_pawns_study() -> Vec<StudyEntry> {
    load_study_entries(STUDY_PAWNS)
}

pub fn get_promotions_study() -> Vec<StudyEntry> {
    load_study_entries(STUDY_PROMOTIONS)
}

pub fn get_stalemates_study() -> Vec<StudyEntry> {
    load_study_entries(STUDY_STALEMATES)
}

pub fn get_standard_study() -> Vec<StudyEntry> {
    load_study_entries(STUDY_STANDARD)
}

pub fn get_taxing_study() -> Vec<StudyEntry> {
    load_study_entries(STUDY_TAXING)
}
