use cpm_core::{Document, EditError, PackInfo, ReadError};

#[test]
fn round_trip_keeps_bom_comments_unknown_fields_and_crlf() {
    let source = "\u{feff}// essai\r\nPackName: Démo\r\nDcFileVersion: 12.5.0\r\n\r\nRequiredAddon{\r\n  Id: addon:custom\r\n}\r\n/* ignoré\r\nName: faux\r\n*/\r\nCustom: {opaque}";
    let document = Document::read_from(source.as_bytes()).unwrap();
    let mut output = Vec::new();
    document.write_to(&mut output).unwrap();
    assert_eq!(output, source.as_bytes());
    assert_eq!(document.properties().len(), 4);
    assert_eq!(
        PackInfo::from_document(&document).unwrap().pack_name,
        "Démo"
    );
}

#[test]
fn scopes_and_duplicates_keep_their_identity() {
    let document = Document::parse("Name: root\nSeat_front{\n Name: first\n Custom{\n Name: nested\n }\n}\nSeat_front{\n Name: second\n}\n").unwrap();
    assert_eq!(document.sections()[1].parent, Some(0));
    assert_eq!(document.sections()[2].parent, None);
    assert_eq!(
        document
            .properties()
            .iter()
            .map(|p| p.section)
            .collect::<Vec<_>>(),
        vec![None, Some(0), Some(1), Some(2)]
    );
}

#[test]
fn exact_keys_and_resource_locations_are_preserved() {
    let document = Document::parse("PlayerPosition: 0 1 2\nPosition: 3 4 5\nModel: dynamxmod:obj/car.obj\nWebsite: https://example.test/a:b\n").unwrap();
    assert_eq!(document.properties()[1].key, "Position");
    assert_eq!(document.properties()[2].value, "dynamxmod:obj/car.obj");
    assert_eq!(document.properties()[3].value, "https://example.test/a:b");
}

#[test]
fn edits_change_only_the_selected_value_and_offsets_stay_correct() {
    let mut document =
        Document::parse("\u{feff}Name: é  \r\nSeat_a{\r\n\tName : old\r\n}\r\nName: last").unwrap();
    document.set_value(0, "Véhicule").unwrap();
    document.set_value(1, "新: siège").unwrap();
    document.set_value(2, "fin").unwrap();
    assert_eq!(
        document.source(),
        "\u{feff}Name: Véhicule  \r\nSeat_a{\r\n\tName : 新: siège\r\n}\r\nName: fin"
    );
}

#[test]
fn empty_values_can_be_set_and_cleared() {
    let mut document = Document::parse("Name:   \nOther:\n").unwrap();
    document.set_value(0, "test").unwrap();
    document.set_value(1, "value").unwrap();
    document.set_value(0, "").unwrap();
    assert_eq!(document.source(), "Name:   \nOther:value\n");
}

#[test]
fn invalid_edits_leave_source_unchanged() {
    let mut document = Document::parse("Name: test").unwrap();
    assert_eq!(
        document.set_value(0, "x\nInjected: y"),
        Err(EditError::MultilineValue)
    );
    assert_eq!(
        document.set_value(8, "x"),
        Err(EditError::UnknownProperty(8))
    );
    assert_eq!(document.source(), "Name: test");
}

#[test]
fn comments_hide_properties_and_braces() {
    let document =
        Document::parse("// }\n/* test */\n/*\nFake{\nKey: hidden\n}\n*/\nName: visible\n")
            .unwrap();
    assert!(document.sections().is_empty());
    assert_eq!(document.properties().len(), 1);
}

#[test]
fn malformed_syntax_reports_the_relevant_line() {
    for (source, expected_line) in [
        ("Name: x\n}\n", 2),
        ("Name: x\nSeat_a{\n", 2),
        ("Name: x\n/* open", 2),
        ("Name: x\nbroken", 2),
        ("Name: x\n: empty key", 2),
        ("Seat_a {\n}", 1),
        ("Seat_a\n{\n}", 1),
    ] {
        assert_eq!(
            Document::parse(source).unwrap_err().line,
            expected_line,
            "{source}"
        );
    }
}

#[test]
fn empty_document_round_trips() {
    let document = Document::parse("").unwrap();
    assert_eq!(document.source(), "");
    assert!(document.properties().is_empty());
}

#[test]
fn invalid_utf8_is_reported() {
    assert!(matches!(
        Document::read_from(&[0xff][..]),
        Err(ReadError::Io(_))
    ));
}

#[test]
fn write_errors_are_propagated() {
    struct FailingWriter;
    impl std::io::Write for FailingWriter {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("disk full"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    assert!(Document::parse("Name: x")
        .unwrap()
        .write_to(FailingWriter)
        .is_err());
}

#[test]
fn pack_info_requires_nonempty_unique_root_properties() {
    let document =
        Document::parse("PackName: \nRequiredAddon{\n DcFileVersion: hidden\n}\n").unwrap();
    assert_eq!(PackInfo::from_document(&document).unwrap_err().len(), 2);
    let duplicate = Document::parse("PackName: A\nPackName: B\nDcFileVersion: 12.5.0").unwrap();
    assert_eq!(
        PackInfo::from_document(&duplicate).unwrap_err()[0].property,
        "PackName"
    );
}

#[test]
fn pack_info_keeps_future_and_addon_properties() {
    let source = "PackName: Test\nDcFileVersion: future\nCustom: preserved\nRequiredAddon{\n Id: custom\n}\n";
    let document = Document::parse(source).unwrap();
    assert_eq!(
        PackInfo::from_document(&document).unwrap().dc_file_version,
        "future"
    );
    assert_eq!(document.source(), source);
}
