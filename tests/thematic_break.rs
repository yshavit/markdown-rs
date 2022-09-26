extern crate micromark;
use micromark::{
    mdast::{Node, Position, Root, ThematicBreak},
    micromark, micromark_to_mdast, micromark_with_options, Constructs, Options,
};
use pretty_assertions::assert_eq;

#[test]
fn thematic_break() -> Result<(), String> {
    assert_eq!(
        micromark("***\n---\n___"),
        "<hr />\n<hr />\n<hr />",
        "should support thematic breaks w/ asterisks, dashes, and underscores"
    );

    assert_eq!(
        micromark("+++"),
        "<p>+++</p>",
        "should not support thematic breaks w/ plusses"
    );

    assert_eq!(
        micromark("==="),
        "<p>===</p>",
        "should not support thematic breaks w/ equals"
    );

    assert_eq!(
        micromark("--"),
        "<p>--</p>",
        "should not support thematic breaks w/ two dashes"
    );

    assert_eq!(
        micromark("**"),
        "<p>**</p>",
        "should not support thematic breaks w/ two asterisks"
    );

    assert_eq!(
        micromark("__"),
        "<p>__</p>",
        "should not support thematic breaks w/ two underscores"
    );

    assert_eq!(
        micromark(" ***"),
        "<hr />",
        "should support thematic breaks w/ 1 space"
    );

    assert_eq!(
        micromark("  ***"),
        "<hr />",
        "should support thematic breaks w/ 2 spaces"
    );

    assert_eq!(
        micromark("   ***"),
        "<hr />",
        "should support thematic breaks w/ 3 spaces"
    );

    assert_eq!(
        micromark("    ***"),
        "<pre><code>***\n</code></pre>",
        "should not support thematic breaks w/ 4 spaces"
    );

    assert_eq!(
        micromark("Foo\n    ***"),
        "<p>Foo\n***</p>",
        "should not support thematic breaks w/ 4 spaces as paragraph continuation"
    );

    assert_eq!(
        micromark("_____________________________________"),
        "<hr />",
        "should support thematic breaks w/ many markers"
    );

    assert_eq!(
        micromark(" - - -"),
        "<hr />",
        "should support thematic breaks w/ spaces (1)"
    );

    assert_eq!(
        micromark(" **  * ** * ** * **"),
        "<hr />",
        "should support thematic breaks w/ spaces (2)"
    );

    assert_eq!(
        micromark("-     -      -      -"),
        "<hr />",
        "should support thematic breaks w/ spaces (3)"
    );

    assert_eq!(
        micromark("- - - -    "),
        "<hr />",
        "should support thematic breaks w/ trailing spaces"
    );

    assert_eq!(
        micromark("_ _ _ _ a"),
        "<p>_ _ _ _ a</p>",
        "should not support thematic breaks w/ other characters (1)"
    );

    assert_eq!(
        micromark("a------"),
        "<p>a------</p>",
        "should not support thematic breaks w/ other characters (2)"
    );

    assert_eq!(
        micromark("---a---"),
        "<p>---a---</p>",
        "should not support thematic breaks w/ other characters (3)"
    );

    assert_eq!(
        micromark(" *-*"),
        "<p><em>-</em></p>",
        "should not support thematic breaks w/ mixed markers"
    );

    assert_eq!(
        micromark("- foo\n***\n- bar"),
        "<ul>\n<li>foo</li>\n</ul>\n<hr />\n<ul>\n<li>bar</li>\n</ul>",
        "should support thematic breaks mixed w/ lists (1)"
    );

    assert_eq!(
        micromark("* Foo\n* * *\n* Bar"),
        "<ul>\n<li>Foo</li>\n</ul>\n<hr />\n<ul>\n<li>Bar</li>\n</ul>",
        "should support thematic breaks mixed w/ lists (2)"
    );

    assert_eq!(
        micromark("Foo\n***\nbar"),
        "<p>Foo</p>\n<hr />\n<p>bar</p>",
        "should support thematic breaks interrupting paragraphs"
    );

    assert_eq!(
        micromark("Foo\n---\nbar"),
        "<h2>Foo</h2>\n<p>bar</p>",
        "should not support thematic breaks w/ dashes interrupting paragraphs (setext heading)"
    );

    assert_eq!(
        micromark("- Foo\n- * * *"),
        "<ul>\n<li>Foo</li>\n<li>\n<hr />\n</li>\n</ul>",
        "should support thematic breaks in lists"
    );

    assert_eq!(
        micromark("> ---\na"),
        "<blockquote>\n<hr />\n</blockquote>\n<p>a</p>",
        "should not support lazyness (1)"
    );

    assert_eq!(
        micromark("> a\n---"),
        "<blockquote>\n<p>a</p>\n</blockquote>\n<hr />",
        "should not support lazyness (2)"
    );

    assert_eq!(
        micromark_with_options(
            "***",
            &Options {
                constructs: Constructs {
                    thematic_break: false,
                    ..Constructs::default()
                },
                ..Options::default()
            }
        )?,
        "<p>***</p>",
        "should support turning off thematic breaks"
    );

    assert_eq!(
        micromark_to_mdast("***", &Options::default())?,
        Node::Root(Root {
            children: vec![Node::ThematicBreak(ThematicBreak {
                position: Some(Position::new(1, 1, 0, 1, 4, 3))
            })],
            position: Some(Position::new(1, 1, 0, 1, 4, 3))
        }),
        "should support thematic breaks as `ThematicBreak`s in mdast"
    );

    Ok(())
}
