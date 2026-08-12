// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use super::*;

fn protocol() -> StudyProtocol {
    let mut protocol =
        StudyProtocol::new("Sleep and Mood", "979b408d-784e-4b1b-bb1e-ff9204e072f3");
    carp_protocol::builder::add_device(&mut protocol, carp_protocol::DeviceKind::Smartphone);
    protocol
}

#[test]
fn a_protocol_survives_a_write_and_read() {
    let directory = std::env::temp_dir().join("carp-studio-storage-test");
    let _ = std::fs::remove_dir_all(&directory);
    let path = directory.join("study/carp/resources/protocol.json");

    let original = protocol();
    write(&original, &path).unwrap();
    let (read_back, resolved) = read(&path).unwrap();

    assert_eq!(read_back, original);
    assert_eq!(resolved, path);
    std::fs::remove_dir_all(&directory).unwrap();
}

/// The output has to be readable, since it is checked into a repository
/// and reviewed as a diff.
#[test]
fn the_written_file_is_pretty_printed() {
    let directory = std::env::temp_dir().join("carp-studio-pretty-test");
    let _ = std::fs::remove_dir_all(&directory);
    let path = directory.join("protocol.json");

    write(&protocol(), &path).unwrap();
    let contents = std::fs::read_to_string(&path).unwrap();

    assert!(contents.contains("\n  \"id\""), "{contents}");
    assert!(contents.lines().count() > 10, "{contents}");
    std::fs::remove_dir_all(&directory).unwrap();
}

/// Pointing at a study directory has to find the protocol inside it.
#[test]
fn a_study_directory_resolves_to_its_protocol() {
    let directory = std::env::temp_dir().join("carp-studio-resolve-test");
    let _ = std::fs::remove_dir_all(&directory);
    let study = directory.join("neuropathy");
    std::fs::create_dir_all(study.join("carp/resources")).unwrap();

    assert_eq!(resolve(&study), study.join(STUDY_RELATIVE_PATH));
    // A path that is already a file is left alone.
    let file = study.join(STUDY_RELATIVE_PATH);
    assert_eq!(resolve(&file), file);

    std::fs::remove_dir_all(&directory).unwrap();
}

/// A name with no extension is taken as a study directory, which is what
/// makes `carp protocol new sleep-study` land in the right place.
#[test]
fn a_bare_name_becomes_a_study_directory() {
    let path = Path::new("/tmp/does-not-exist-sleep-study");
    assert_eq!(resolve(path), path.join(STUDY_RELATIVE_PATH));
}

#[test]
fn a_default_path_is_derived_from_the_name() {
    let path = default_path(&protocol(), Path::new("/data"));
    assert_eq!(
        path,
        Path::new("/data/protocols/sleep-and-mood").join(STUDY_RELATIVE_PATH)
    );
}

#[test]
fn slugs_are_file_name_safe() {
    assert_eq!(slug("Sleep and Mood"), "sleep-and-mood");
    assert_eq!(slug("WHO-5 / Wellbeing!"), "who-5-wellbeing");
    assert_eq!(slug("   "), "protocol");
}

/// A missing file has to say what was expected, since the layout is a
/// convention rather than something the user can guess.
#[test]
fn a_missing_protocol_says_what_was_expected() {
    let error = read_checked(Path::new("/tmp/carp-studio-absent-study"))
        .unwrap_err()
        .to_string();
    assert!(error.contains("no protocol at"), "{error}");
    assert!(error.contains(STUDY_RELATIVE_PATH), "{error}");
}

/// A file that is not a protocol has to say so rather than failing with a
/// serde message about a missing field.
#[test]
fn a_file_that_is_not_a_protocol_says_so() {
    let path = std::env::temp_dir().join("carp-studio-not-a-protocol.json");
    std::fs::write(&path, "{\"hello\": \"world\"}").unwrap();

    let error = format!("{:#}", read(&path).unwrap_err());
    assert!(error.contains("is not a CARP protocol"), "{error}");
    std::fs::remove_file(&path).unwrap();
}
