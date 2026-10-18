//! C4: uppercase / text transform.
//!
//! The roadmap's prescription: a transform field in the style model,
//! applied at render time, never to the stored text, so the transcript
//! keeps its casing. Test first: a preset with uppercase renders
//! uppercase, and the stored caption text is unchanged. The reader
//! follows the wire pattern of every other named enum here: an absent
//! key or an unrecognised name loads as the default, so every older
//! document reads as `none` and nothing migrates.

use concat_project::model::{TextStyle, TextTransform};

#[test]
fn a_style_with_uppercase_renders_uppercase_and_the_stored_text_is_unchanged() {
    let style = TextStyle {
        content: "halo dunia".to_owned(),
        transform: TextTransform::Uppercase,
        ..TextStyle::default()
    };

    assert_eq!(style.display_content(), "HALO DUNIA");
    assert_eq!(
        style.content, "halo dunia",
        "the transcript keeps its casing; the transform is render-time only"
    );
}

#[test]
fn an_old_document_without_the_transform_loads_as_none_and_renders_as_stored() {
    let style: TextStyle =
        serde_json::from_str(r#"{ "content": "Halo Dunia" }"#).expect("an old style loads");
    assert_eq!(style.transform, TextTransform::None);
    assert_eq!(style.display_content(), "Halo Dunia");
}

#[test]
fn an_unrecognised_transform_name_loads_as_none() {
    let style: TextStyle = serde_json::from_str(r#"{ "content": "x", "transform": "smallcaps" }"#)
        .expect("an unknown name is not a load failure");
    assert_eq!(style.transform, TextTransform::None);
}

#[test]
fn the_transform_round_trips_through_the_document_wire_and_none_stays_off_it() {
    let upper = TextStyle {
        transform: TextTransform::Uppercase,
        ..TextStyle::default()
    };
    let json = serde_json::to_string(&upper).expect("serialises");
    assert!(
        json.contains(r#""transform":"uppercase""#),
        "the wire name is the stable lowercase one: {json}"
    );
    let back: TextStyle = serde_json::from_str(&json).expect("deserialises");
    assert_eq!(back.transform, TextTransform::Uppercase);

    let plain = TextStyle::default();
    let json = serde_json::to_string(&plain).expect("serialises");
    assert!(
        !json.contains("transform"),
        "a style with no transform stays off the wire, as older builds wrote it: {json}"
    );
}
