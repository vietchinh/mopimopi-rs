# Review checklist from the three talks

Used on every function and type touched. Sources: T1 "how to write the perfect function" (video essay on Tony Van Eerd),
T2 Sean Parent "C++ Seasoning", T3 Tony Van Eerd, functions and value-oriented programming part 1.

1. **Calculate vs act** (T1, T3): a function computes a value or performs an effect, not both; return values, not `()` or out-params.
2. **Honest signature** (T1): clock, storage, config, signals come in as arguments; effects live in the outermost layer.
3. **Minimal inputs** (T3): take the fields used, not the whole state.
4. **Weakest parameter type** (T1): `&[T]`/iterators, not `&Vec<T>`; newtypes instead of bool flags.
5. **Preconditions in types** (T1): smart constructors (Rate, Percent, EncounterKey).
6. **One abstraction level** (T1, T3): section comments and "step one/two" are cut points.
7. **No raw loops** (T2): iterator adapters or a named helper; closures at most a composition of two functions.
8. **Name for what it does** (T3): no parameter types in the name, no `get_` on anything that computes.
9. **Value types by default** (T2, T3): plain, cloneable, comparable; strong ids instead of handles.
10. **Avoid shared mutable state** (T2): `Rc<RefCell>`/`Mutex` is a smell.
11. **Polymorphism as a value** (T2): enums over hierarchies.
12. **Behaviour near the data** (T3): free functions over public basis operations.
