/**
 * Tree-sitter grammar for Garnet (S53) — adoption infrastructure.
 *
 * This is the SYNTAX grammar used by editors for highlighting / folding /
 * structural navigation. It is intentionally separate from the LSP semantic
 * service (S44), which runs on the compiler frontend. The canonical grammar is
 * the hand-written parser in `garnet-parser`; this mirrors its core surface.
 *
 * Honest scope: this is a CORE grammar covering the headline constructs (enough
 * for editor highlighting), not an exhaustive reproduction of every form. It is
 * structurally validated in CI (`scripts/garnet_tree_sitter_check.py`); running
 * `tree-sitter generate` + corpus tests requires the tree-sitter CLI.
 */

module.exports = grammar({
  name: 'garnet',

  extras: $ => [/\s/, $.line_comment, $.doc_comment],

  word: $ => $.identifier,

  rules: {
    source_file: $ => repeat($._item),

    _item: $ => choice(
      $.function_definition,
      $.struct_definition,
      $.enum_definition,
      $.actor_definition,
      $.memory_declaration,
      $.impl_block,
      $.const_declaration,
      $.let_declaration,
      $.use_declaration,
      $._statement,
    ),

    // ── Annotations: @caps(fs, net), @safe, @bounded(8), @mailbox(64) ─────
    annotation: $ => seq(
      '@',
      field('name', $.identifier),
      optional(seq('(', optional(commaSep($._annotation_arg)), ')')),
    ),
    _annotation_arg: $ => choice($.identifier, $.integer, '*'),

    // ── Functions ────────────────────────────────────────────────────────
    function_definition: $ => seq(
      repeat($.annotation),
      optional('pub'),
      choice('def', 'fn'),
      field('name', $.identifier),
      field('parameters', $.parameter_list),
      optional(seq('->', field('return_type', $.type))),
      field('body', $.block),
    ),
    parameter_list: $ => seq('(', optional(commaSep($.parameter)), ')'),
    parameter: $ => seq(
      optional(choice('mut', 'ref', 'borrow', 'move', 'own')),
      field('name', $.identifier),
      optional(seq(':', field('type', $.type))),
    ),

    // ── Type / struct / enum / trait / impl ──────────────────────────────
    type: $ => seq(
      sepBy('::', $.identifier),
      optional(seq('<', commaSep($.type), '>')),
    ),
    struct_definition: $ => seq(
      repeat($.annotation), optional('pub'), 'struct',
      field('name', $.identifier),
      '{', optional(commaSep($.field_declaration)), '}',
    ),
    field_declaration: $ => seq(field('name', $.identifier), ':', field('type', $.type)),
    enum_definition: $ => seq(
      repeat($.annotation), optional('pub'), 'enum',
      field('name', $.identifier),
      '{', optional(commaSep($.enum_variant)), '}',
    ),
    enum_variant: $ => seq(
      field('name', $.identifier),
      optional(seq('(', commaSep($.type), ')')),
    ),
    impl_block: $ => seq(
      repeat($.annotation), 'impl',
      optional(seq(field('trait', $.type), 'for')),
      field('type', $.type),
      '{', repeat($.function_definition), '}',
    ),

    // ── Actors + memory ──────────────────────────────────────────────────
    actor_definition: $ => seq(
      repeat($.annotation), 'actor', field('name', $.identifier),
      '{', repeat($._actor_item), '}',
    ),
    _actor_item: $ => choice(
      $.function_definition, $.memory_declaration, $.protocol_declaration,
      $.on_handler, $.let_declaration,
    ),
    protocol_declaration: $ => seq(
      'protocol', field('name', $.identifier), $.parameter_list,
      optional(seq('->', $.type)),
    ),
    on_handler: $ => seq('on', field('message', $.identifier), $.parameter_list, $.block),
    memory_declaration: $ => seq(
      'memory',
      field('kind', choice('episodic', 'semantic', 'working', 'procedural')),
      field('name', $.identifier),
    ),

    // ── Statements ───────────────────────────────────────────────────────
    block: $ => seq('{', repeat($._statement), '}'),
    _statement: $ => choice(
      $.let_declaration,
      $.return_statement,
      $.raise_statement,
      $.while_loop,
      $.for_loop,
      $.break_statement,
      $.continue_statement,
      $._expression,
    ),
    let_declaration: $ => seq(
      choice('let', 'var'), optional('mut'),
      field('name', $.identifier),
      optional(seq(':', field('type', $.type))),
      optional(seq('=', field('value', $._expression))),
    ),
    const_declaration: $ => seq(
      optional('pub'), 'const', field('name', $.identifier),
      optional(seq(':', $.type)), '=', field('value', $._expression),
    ),
    use_declaration: $ => seq('use', sepBy('::', choice($.identifier, '*'))),
    return_statement: $ => seq('return', optional($._expression)),
    raise_statement: $ => seq('raise', $._expression),
    break_statement: $ => 'break',
    continue_statement: $ => choice('continue', 'next'),
    while_loop: $ => seq('while', field('condition', $._expression), $.block),
    for_loop: $ => seq('for', field('pattern', $.identifier), 'in', field('iterable', $._expression), $.block),

    // ── Expressions ──────────────────────────────────────────────────────
    _expression: $ => choice(
      $.identifier,
      $.integer,
      $.float,
      $.string,
      $.boolean,
      $.nil,
      $.self,
      $.call_expression,
      $.field_expression,
      $.binary_expression,
      $.pipe_expression,
      $.if_expression,
      $.match_expression,
      $.try_expression,
      $.spawn_expression,
      $.block,
      seq('(', $._expression, ')'),
    ),
    call_expression: $ => prec(10, seq(
      field('function', $._expression),
      '(', optional(commaSep($._expression)), ')',
      optional('?'),
    )),
    field_expression: $ => prec(9, seq(field('object', $._expression), '.', field('field', $.identifier))),
    binary_expression: $ => prec.left(1, seq(
      $._expression,
      choice('+', '-', '*', '/', '%', '==', '!=', '<', '<=', '>', '>=', 'and', 'or', 'not'),
      $._expression,
    )),
    pipe_expression: $ => prec.left(2, seq($._expression, '|>', $._expression)),
    if_expression: $ => seq(
      'if', field('condition', $._expression), $.block,
      repeat(seq('elsif', $._expression, $.block)),
      optional(seq('else', $.block)),
    ),
    match_expression: $ => seq(
      'match', field('subject', $._expression),
      '{', repeat($.match_arm), '}',
    ),
    match_arm: $ => seq(field('pattern', $._pattern), '=>', field('value', $._expression), optional(',')),
    _pattern: $ => choice($.identifier, $.integer, $.string, $.boolean, $.nil, '_', $.call_expression),
    try_expression: $ => seq(
      'try', $.block,
      repeat($.rescue_clause),
      optional(seq('ensure', $.block)),
    ),
    rescue_clause: $ => seq(
      'rescue',
      optional(field('binding', $.identifier)),
      optional(seq(':', field('type', $.type))),
      $.block,
    ),
    spawn_expression: $ => seq('spawn', $._expression),

    // ── Tokens ───────────────────────────────────────────────────────────
    identifier: $ => /[A-Za-z_][A-Za-z0-9_]*/,
    integer: $ => /-?\d+/,
    float: $ => /-?\d+\.\d+/,
    string: $ => /"([^"\\]|\\.)*"/,
    boolean: $ => choice('true', 'false'),
    nil: $ => 'nil',
    self: $ => 'self',
    // `#` line comments and `///` doc comments.
    doc_comment: $ => token(seq('///', /.*/)),
    line_comment: $ => token(seq('#', /.*/)),
  },
});

function commaSep(rule) {
  return seq(rule, repeat(seq(',', rule)), optional(','));
}
function sepBy(sep, rule) {
  return seq(rule, repeat(seq(sep, rule)));
}
