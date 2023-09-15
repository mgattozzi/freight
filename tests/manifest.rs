use freight::config::Manifest;

#[test]
fn can_parse_good_manifest() {
    assert!(Manifest::parse_from_file("tests/Freight_Fixture.toml").is_ok());
}

#[test]
fn will_fail_bad_manifest() {
    assert_eq!(
        Manifest::parse_from_file("tests/Freight_Bad_Fixture.toml")
            .unwrap_err()
            .to_string(),
        "Field bad_field is unsupported".to_string()
    );
}
