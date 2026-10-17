# 0005 Splitting javascript_compat
`parse_leading_float` -> ACT edge. `p_float` -> Rate/Percent constructors. `to_fixed`, `round_to_two_decimals`,
`number_to_javascript_string` -> small domain shared kernel.
Principle: shared kernel, modules by meaning. Status: accepted.
