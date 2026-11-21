# Struct-args follow-up tasks

Copied from the task tracker rows (as of Nov 15, 2026). Status in brackets.

- [To do, High] Reply to the open points in the Rust internals thread. Unanswered as of Nov 15: Kixunil's ImplicitDefault + #[non_exhaustive] idea (post 26), robinm's crater run / edition suggestion (27, 28), scottmcm's point that field order should at most be a lint (37, 40). Last post was Nov 5.
- [To do, High] Decide whether records stay nominal. Identically shaped struct { } types are distinct, so forwarding and FRU across them fail (REPORT.md cases 05, 07). mathstuf's annotated closure needs structural equivalence.
- [To do, Medium] Allow struct { .. } in closure parameters and Fn bounds. Prototype rejects both: "only allowed as parameter types" (REPORT.md cases 02 and 12).
- [To do, Medium] Make a left-out record work through closures. closure(width: 200) works but closure() fails with E0057 (case 13). Guess, unverified: omittable_record only handles fn-item callees.
- [To do, Medium] Design the $p:param macro fragment. Proposed Oct 23. Elided struct patterns and Pair<T> { a, b } params aren't matched by $p:pat_param, and macro_rules has no alternation.
- [To do, Low] Show full record names in the E0308 mismatch note. Case 05 prints {record@...} with no spans; case 07 does show spans.
- [To do, Low] Revisit the ; parameter sugar. Set aside on Oct 23. Named-argument call syntax f(x: 1) stays.
- [Waiting] Settle _ { } vs RFC 3444's .{ } spelling. RFC 3444 Path Inference was still an open PR on Nov 8 and overlaps struct_pattern_inference.
- [Waiting] Track rust-lang/rust#162264 (: after $p:pat_param). In lang FCP as of Oct 23. If it lands, the FIXME(struct_args) macro fallthrough in parser/pat.rs also breaks $p:pat_param : arms.
