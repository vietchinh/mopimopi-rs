# 0003 Pet merging is a view, not a mutation
Decision: merging is a side-effect-free method on the Encounter returning a view; the application layer reads the
`pets` option and decides when to ask. Nothing is merged on arrival.
Principle: side-effect-free functions (DDD video; talk 3 "separate calculating from doing"). Status: accepted.
