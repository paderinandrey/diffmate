use super::super::parse;
use proptest::prelude::*;

fn piece() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("diff --git a/x b/x"),
        Just("--- a/x"),
        Just("+++ b/x"),
        Just("--- /dev/null"),
        Just("@@ -1,2 +3 @@"),
        Just("@@ -4294967295,3 +4294967295,5 @@"),
        Just("@@ -0,0 +0,0 @@"),
        Just(" ctx"),
        Just("-del"),
        Just("+add"),
        Just(""),
        Just("\\ No newline at end of file"),
        Just("\"a/\\303"),
        Just("+++ \"b/\\777\\\\\""),
    ]
}

proptest! {
    #[test]
    fn arbitrary_bytes_never_panic(bytes in prop::collection::vec(any::<u8>(), 0..512)) {
        let _ = parse(&bytes);
    }

    #[test]
    fn diff_shaped_text_never_panics(parts in prop::collection::vec(piece(), 0..40)) {
        let _ = parse(parts.join("\n").as_bytes());
    }
}
