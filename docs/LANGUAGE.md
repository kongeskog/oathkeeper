# Oathkeeper 0.1 language and semantics

## Declarations

```text
module dotted.name;
use package.module.Name;
use package.module.{Name, Other as Alias};
type Record<T> = { field: T, other: Bool };
type Alias = Int;
enum Option<T> { None, Some(T) }
const named_value [: Type] = expression;
fn function<T>(argument: T) -> T { argument }
query question(argument: Type) -> Answer {
    scope = boolean_expression;
    applies = boolean_expression;
    meta = typed_constant;
}
rule named_rule for question(argument) {
    when = boolean_expression;
    yield = expression;
    meta = typed_constant;
}
priority named_priority for question(argument) {
    higher = named_rule;
    lower = another_rule;
    when = boolean_expression;
    meta = typed_constant;
}
```

The `meta` field is optional on queries, rules and priorities. Other legal
annotations are ordinary typed constants referencing the relevant symbol or
source. No legal source category is a reserved declaration keyword.

Identifiers are ASCII letters/underscore followed by letters/digits/underscore.
Names are case-sensitive. Modules are explicitly imported. All declarations in
the entry's transitive import closure participate, including rules not reachable
by expression calls. There is no discovery from directory proximity. The locked
source snapshot is parsed and the included closure is typechecked; syntax errors
anywhere in the snapshot are diagnosed. Mutually importing modules are allowed;
executable dependencies must remain acyclic.

Declarations are globally identified by module and name. Fully qualified
references still require the declaration's module to be in the import closure.
Duplicate declarations, import aliases, fields, parameters and priority cycles
are errors. Cycles of legal IDs in data do not constitute executable recursion.

## Types and expressions

Kernel types are `Unit`, `Bool`, `Text`, `Int`, `Rational`, finite
`List<T>`, nominal records and nominal variants. Function declarations are
statically resolved; function values, closures, partial application and dynamic
dispatch are not available in v0.1.

Type parameters are inferred from arguments and context. Explicit arguments use
`Box::<Int> { value: 1 }`, `Option.None::<Int>()` where a contextual type is
unavailable, or `function::<Int>(1)`. Prefer a type annotation for an empty
list or payload-free generic variant. Record fields must exactly match the
declared schema. Currencies use nominal phantom parameters, e.g. `Money<USD>`.

Expressions include literals, record/variant construction, field access, calls,
`if condition { yes } else { no }`, exhaustive `match`, and block bindings:

```oath
fn twice(x: Int) -> Int { x * 2 }
fn total(acc: Int, x: Int) -> Int { acc + x }
fn example() -> Int {
    let values = map([1, 2, 3], twice);
    fold(values, 0, total)
}
```

`map`, `filter`, `fold`, `all` and `any` accept named, statically resolved
functions. All traversal follows list order. `all` and `any` stop on a decisive
value. A missing filter predicate prevents a known filtered list.

Operator precedence, from weakest to strongest: `or`; `and`;
`== !=`; `< <= > >=`; `+ -`; `* / %`; unary `- not !`;
calls and field access. Division produces Rational, even for Int operands.
Other arithmetic requires matching numeric types. Int literals can acquire a
Rational contextual type; arbitrary implicit conversions are absent.

Integer remainder follows truncation toward zero. The explicit
`floor_div(a,b)` primitive uses floor division. Decimal literals are exact
fractions; all wire rationals are reduced with positive denominators.
Division by zero, invalid indexing and `fail(message)` are execution errors,
not query outcomes. Conditions are Bool, not arbitrary truthy values.

Primitive functions are:
`len, at, append, concat, range, to_rational, floor, floor_div, text_len,
text_at, parse_int, int_text, fail`.
`range(start,end)` is half-open, ascending and empty when end <= start.
Text indexing counts Unicode scalar values, not UTF-8 bytes or graphemes.
No locale-dependent ordering is used.

Ordering is structural and deterministic: numeric order, Unicode scalar text
order, field-name order for records, variant-name then payload order for enums,
and lexicographic list order. Compared values must have the same static type.
Set/map constructors in std.collections normalize entries to this order.
Public record construction can bypass these library invariants; use the
normalizing functions for values claimed to be sets/maps.

## Knowledge and outcomes

Every scenario field is Known, Missing, or Disputed. A known record may contain
missing fields; accessing a different known field remains possible. Known false,
known None, missing Option, and a known empty list are distinct. A list does not
assert that its real-world inventory is complete.

Agreeing disputed claims combine evidence. Differing claims remain disputed.
Evidence IDs are supplied labels, not authenticated facts. The evaluator
propagates blockers through relevant operations; it does not assume negation
when proof is absent.

`false and unknown` and `unknown and false` are false.
`true or unknown` and `unknown or true` are true. Left-to-right short circuit
avoids evaluating the right operand when the left decides the result. An unknown
conditional does not select either branch, even if its branches look equal.

Query evaluation:

1. False scope -> OutsideCoverage; unresolved scope -> Undetermined.
2. False applicability -> Inapplicable; unresolved applicability -> Undetermined.
3. Evaluate every included candidate guard and every priority condition.
4. Any unresolved guard/priority -> Undetermined in v0.1.
5. Compute maximal active rules using enabled priority's transitive closure.
6. Evaluate only maximal yields. Missing yielded data -> Undetermined.
7. Equal normalized answers combine support. Different answers -> Conflicted.
8. No active rule -> Undetermined with NoSupportingRule.

Priority is a strict partial order, scoped to one query. Its transitive relation
passes through inactive intermediate rules. All potential cycles are rejected,
even if conditions appear mutually exclusive. Source order, directory depth,
source dates, and source citations confer no priority.

Consuming an unresolved query propagates issues. Consuming an inapplicable,
unsupported or conflicted query produces a named dependency issue, never false,
zero or an empty list. Resolved false remains false.

Independent obligations should be explicitly collected, with an authored empty
collection for no contributions. They should not be encoded as incompatible
answers to a single-valued query.

## Calendars and effects

`std.date.Date` is a nominal record containing a civil-day index relative to
1970-01-01, using the proleptic Gregorian calendar and astronomical year numbering.
Its constructors return Result for invalid dates; `date` is a checked convenience
that raises an error. ISO parsing accepts exactly YYYY-MM-DD, including year 0000.
`add_months` requires Reject or Clamp overflow policy. Dates are not instants.

`std.numeric.round` takes an explicit positive quantum and mode. No money or
calendar operation reads the clock. Currency conversion requires an explicit
rate. Domain deadlines, enactment, commencement, savings clauses, authority
cutoffs and transitions are ordinary authored functions/queries in lawlib.

Execution does not cause legal or institutional acts. An event must be supplied
as input to an authored pure state transition.
