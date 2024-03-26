use revm_interpreter::analysis::{validate_raw_eof, EofError};
use revm_primitives::{Bytes, Eof, HashMap};
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use walkdir::{DirEntry, WalkDir};

// #[test]
// fn eof_run_all_tests() {
//     let eof_tests = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/EOFTests");
//     run_test(&eof_tests)
// }

#[test]
fn eof_validation_eip3540() {
    let eof_tests = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/EOFTests/EIP3540");
    run_test(&eof_tests)
}

#[test]
fn eof_validation_eip3670() {
    let eof_tests = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/EOFTests/EIP3670");
    run_test(&eof_tests)
}

#[test]
fn eof_validation_eip4200() {
    let eof_tests = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/EOFTests/EIP4200");
    run_test(&eof_tests)
}

#[test]
fn eof_validation_eip4750() {
    let eof_tests = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/EOFTests/EIP4750");
    run_test(&eof_tests)
}

#[test]
fn eof_validation_eip5450() {
    let eof_tests = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/EOFTests/EIP5450");
    run_test(&eof_tests)
}

pub fn run_test(path: &Path) {
    let test_files = find_all_json_tests(path);
    let mut test_sum = 0;
    let mut passed_tests = 0;
    let mut types_of_error: HashMap<EofError, usize> = HashMap::new();
    for test_file in test_files {
        let s = std::fs::read_to_string(test_file).unwrap();
        let suite: TestSuite = serde_json::from_str(&s).unwrap();
        for (name, test_unit) in suite.0 {
            for (vector_name, test_vector) in test_unit.vectors {
                test_sum += 1;
                let res = validate_raw_eof(test_vector.code.clone());
                if res.is_ok() != test_vector.results.prague.result {
                    let eof = Eof::decode(test_vector.code.clone());
                    println!(
                        "Test failed: {} - {}\nresult:{:?}\nrevm result:{:?}\nbytes:{:?}\neof: {eof:?}",
                        name, vector_name, test_vector.results.prague, res, test_vector.code
                    );
                    *types_of_error
                        .entry(res.err().unwrap_or(EofError::TEST))
                        .or_default() += 1;
                } else {
                    println!("Test passed: {} - {}", name, vector_name);
                    passed_tests += 1;
                }
            }
        }
    }
    println!("Types of error: {:#?}", types_of_error);
    println!("Passed tests: {}/{}", passed_tests, test_sum);
}

pub fn find_all_json_tests(path: &Path) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".json"))
        .map(DirEntry::into_path)
        .collect::<Vec<PathBuf>>()
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct TestSuite(pub BTreeMap<String, TestUnit>);

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestUnit {
    /// Test info is optional
    #[serde(default, rename = "_info")]
    pub info: Option<serde_json::Value>,

    pub vectors: BTreeMap<String, TestVector>,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestVector {
    code: Bytes,
    results: PragueResult,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PragueResult {
    #[serde(rename = "Prague")]
    prague: Result,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct Result {
    result: bool,
    exception: Option<String>,
}
