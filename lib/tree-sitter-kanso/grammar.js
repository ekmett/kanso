
// identifier: http://wiki.portal.chalmers.se/agda/pmwiki.php?n=ReferenceManual.Names
const ID = /([^\s\\.\"\(\)\{\}@\'\\_]|\\[^\sa-zA-Z]|_[^\s;\.\"\(\)\{\}@])[^\s;\.\"\(\)\{\}@]*/;
// qualified identifier: http://wiki.portal.chalmers.se/agda/pmwiki.php?n=ReferenceManual.Names

const BRACE1 = [['{', '}']];
const PAREN = [['(', ')']];

// numbers and literals
const number = /0x[0-9a-fA-F]+|[0-9]+/;
const integer = /\-?(0x[0-9a-fA-F]+|[0-9]+)/;
const exponent = /[eE][-+]?(0x[0-9a-fA-F]+|[0-9]+)/;
const float = /(\-?(0x[0-9a-fA-F]+|[0-9]+)\.(0x[0-9a-fA-F]+|[0-9]+)([eE][-+]?(0x[0-9a-fA-F]+|[0-9]+))?)|((0x[0-9a-fA-F]+|[0-9]+)[eE][-+]?(0x[0-9a-fA-F]+|[0-9]+))/;

module.exports = grammar({
  name: 'kanso',


  // match keywords using a 2-step algorithm, first match this
  // then match the keyword
  word: $ => $.id,

  // what we skip
  extras: $ => [
    $.comment,
    /\\n/,
  ],

  // handled by external lexical scanners
  externals: $ => [
    $._newline,
    $._indent,
    $._dedent
  ],

  // rule names to inline in the final parser so they don't clutter up the syntax tree
  inline: $ => [
  ],

  // expected LR(1) conflicts
  conflicts: $ => [
  ],

  rules: {
    comment: $ => token(
      prec(100, seq('#', /.*/)),
    ),

    id: $ => token(ID),

    source_file: $ => repeat(seq($._decl, $._newline)),

    _FORALL: $ => token(choice('forall', '∀')),
    _ARROW: $ => token(choice('->','→')),
    _LAMBDA: $ => token(choice('\\','λ')),
    _ELLIPSIS: $ => token(choice('...','…')),
    bind = $.id <|> "_"

    // Declarations
    // indented, 1 or more declarations
    _decl_block: $ => block($, $._decl),

    // Declaration
    _decl: $ => choice(
       $.function // declaration or definition
    ),

    function: $ => choice(
      seq(
        optional($.attributes),
        alias($.lhs_decl, $.lhs),
        alias(optional($.rhs_decl), $.rhs),
        optional($.where),
      ),
      seq(
        optional($.attributes),
        alias($.lhs_defn, $.lhs),
        alias(optional($.rhs_defn), $.rhs),
        optional($.where),
      ),
    ),

    // LHS
    lhs_decl: $ => seq(
      $.identifier,
      ":",
      $.type
    ),
    lhs_defn: $ => prec(1, 
      seq(
        $.identifier,
        $.arguments,
        "="
        $.expr // TODO: enso-style bodies?
    )),

    // RHS
    rhs_decl: $ => seq(':', $.expr),
    rhs_defn: $ => seq('=', $.expr),

    // WithExpressions
    with_expressions: $ => seq('with', $.expr),

    // RewriteEquations
    rewrite_equations: $ => seq('rewrite', $._with_exprs),

    // WhereClause
    where: $ => seq(
      optional(seq(
        'module',
        $.bid
      )),
      'where',
      optional($._declaration_block)
    ),

    ////////////////////////////////////////////////////////////////////////
    // Declaration: Data
    ////////////////////////////////////////////////////////////////////////

    data_name: $ => alias($.id, 'data_name'),

    data: $ => seq(
      choice('data', 'codata'),
      $.data_name,
      optional($._typed_untyped_bindings),
      optional(seq(':', $.expr)),
      'where',
      optional($._declaration_block)
    ),

    ////////////////////////////////////////////////////////////////////////
    // Declaration: Data Signature
    ////////////////////////////////////////////////////////////////////////

    data_signature: $ => seq(
      'data',
      $.data_name,
      optional($._typed_untyped_bindings),
      ':',
      $.expr,
    ),

    ////////////////////////////////////////////////////////////////////////
    // Declaration: Record
    ////////////////////////////////////////////////////////////////////////

    // Record
    record: $ => seq(
      'record',
      alias($._atom_no_curly, $.record_name),
      optional($._typed_untyped_bindings),
      optional(seq(':', $.expr)),
      $.record_declarations_block,
    ),

    // RecordDeclarations
    record_declarations_block: $ => seq(
      'where',
      indent($,
        // RecordDirectives
        repeat(seq($._record_directive, $._newline)),
        repeat(seq($._declaration, $._newline)),
      ),
    ),

    // RecordDirective
    _record_directive: $ => choice(
        $.record_constructor,
        $.record_constructor_instance,
        $.record_induction,
        $.record_eta
    ),
    // RecordConstructorName
    record_constructor: $ => seq('constructor', $.id),

    // Declaration of record constructor name.
    record_constructor_instance: $ => seq(
        'instance',
        block($, $.record_constructor),
    ),

    // RecordInduction
    record_induction: $ => choice(
        'inductive',
        'coinductive'
    ),

    // RecordEta
    record_eta: $ => choice(
        'eta-equality',
        'no-eta-equality'
    ),


    ////////////////////////////////////////////////////////////////////////
    // Declaration: Record Signature
    ////////////////////////////////////////////////////////////////////////

    record_signature: $ => seq(
      'record',
      alias($._atom_no_curly, $.record_name),
      optional($._typed_untyped_bindings),
      ':',
      $.expr
    ),

    ////////////////////////////////////////////////////////////////////////
    // Declaration: Infix
    ////////////////////////////////////////////////////////////////////////

    infix: $ => seq(
      choice('infix', 'infixl', 'infixr'),
      $.integer,
      repeat1($.bid),
    ),

    ////////////////////////////////////////////////////////////////////////
    // Declaration: Generalize
    ////////////////////////////////////////////////////////////////////////

    generalize: $ => seq(
      'variable',
      optional($._signature_block)
    ),

    ////////////////////////////////////////////////////////////////////////
    // Declaration: Mutual
    ////////////////////////////////////////////////////////////////////////

    mutual: $ => seq(
      'mutual',
      optional($._declaration_block)
    ),

    ////////////////////////////////////////////////////////////////////////
    // Declaration: Abstract
    ////////////////////////////////////////////////////////////////////////

    abstract: $ => seq(
      'abstract',
      optional($._declaration_block)
    ),

    ////////////////////////////////////////////////////////////////////////
    // Declaration: Private
    ////////////////////////////////////////////////////////////////////////

    private: $ => seq(
      'private',
      optional($._declaration_block)
    ),

    ////////////////////////////////////////////////////////////////////////
    // Declaration: Instance
    ////////////////////////////////////////////////////////////////////////

    instance: $ => seq(
      'instance',
      optional($._declaration_block)
    ),

    ////////////////////////////////////////////////////////////////////////
    // Declaration: Macro
    ////////////////////////////////////////////////////////////////////////

    macro: $ => seq(
      'macro',
      optional($._declaration_block)
    ),

    ////////////////////////////////////////////////////////////////////////
    // Declaration: Postulate
    ////////////////////////////////////////////////////////////////////////

    postulate: $ => seq(
      'postulate',
      optional($._declaration_block)
    ),

    ////////////////////////////////////////////////////////////////////////
    // Declaration: Primitive
    ////////////////////////////////////////////////////////////////////////

    primitive: $ => seq(
      'primitive',
      optional($._type_signature_block)
    ),

    // TypeSignatures
    _type_signature_block: $ => block($, $.type_signature),

    // TypeSigs
    type_signature: $ => seq(
      $._field_names,
      ':',
      $.expr
    ),

    ////////////////////////////////////////////////////////////////////////
    // Declaration: Open
    ////////////////////////////////////////////////////////////////////////

    open: $ => seq(
      choice(
        seq(        'import'),
        seq('open', 'import'),
        seq('open'          ),
      ),
      $.module_name,
      optional($._atoms),
      optional($._import_directives),
    ),

    // ModuleName
    module_name: $ => prec.left(alias($.qid, 'module_name')),

    // ImportDirectives and shit
    _import_directives: $ => repeat1($.import_directive),
    import_directive: $ => choice(
      'public',
      seq('using', '(', $._comma_import_names ,')'),
      seq('hiding', '(', $._comma_import_names ,')'),
      seq('renaming', '(', sepR(';', $.renaming) ,')'),
      seq('using', '(' ,')'),
      seq('hiding', '(' ,')'),
      seq('renaming', '(' ,')')
    ),

    // CommaImportNames
    _comma_import_names: $ => sepR(';', $._import_name),

    // Renaming
    renaming: $ => seq(
        optional('module'),
        $.id,
        'to',
        $.id
    ),

    // ImportName
    _import_name: $ => seq(
        optional('module'), $.id
    ),


    ////////////////////////////////////////////////////////////////////////
    // Declaration: Module Macro
    ////////////////////////////////////////////////////////////////////////

    // ModuleMacro
    module_macro: $ => seq(
      choice(
        seq('module', alias($.qid, $.module_name)),
        seq('open', 'module', alias($.qid, $.module_name)),
      ),
      optional($._typed_untyped_bindings),
      '=',
      $.module_application,
      repeat($.import_directive),
    ),

    // ModuleApplication
    module_application: $ => seq(
      $.module_name,
      choice(
        prec(1, brace_double($._ELLIPSIS)),
        optional($._atoms),
      ),
    ),

    ////////////////////////////////////////////////////////////////////////
    // Declaration: Module
    ////////////////////////////////////////////////////////////////////////

    // Module
    module: $ => seq(
        'module',
        alias(choice($.qid, '_'), $.module_name),
        optional($._typed_untyped_bindings),
        'where',
        optional($._declaration_block)
    ),

    ////////////////////////////////////////////////////////////////////////
    // Declaration: Pragma
    ////////////////////////////////////////////////////////////////////////

    // Pragma / DeclarationPragma
    pragma: $ => token(seq(
      '{-#',
      repeat(choice(
        /[^#]/,
        /#[^-]/,
        /#\-[^}]/,
      )),
      '#-}',
    )),

    // CatchallPragma
    catchall_pragma: $ => seq('{-#', 'CATCHALL', '#-}'),

    ////////////////////////////////////////////////////////////////////////
    // Declaration: Syntax
    ////////////////////////////////////////////////////////////////////////

    syntax: $ => seq(
      'syntax',
      $.id,
      $.hole_names,
      '=',
      repeat1($.id)
    ),

    // HoleNames
    hole_names: $ => repeat1($.hole_name),
    hole_name: $ => choice(
      $._simple_top_hole,
      brace(       $._simple_hole),
      brace_double($._simple_hole),
      brace(       $.id, '=', $._simple_hole),
      brace_double($.id, '=', $._simple_hole),
    ),

    // SimpleTopHole
    _simple_top_hole: $ => choice(
      $.id,
      paren($._LAMBDA, $.bid, $._ARROW, $.id),
    ),

    // SimpleHole
    _simple_hole: $ => choice(
      $.id,
      seq($._LAMBDA, $.bid, $._ARROW, $.id),
    ),

    ////////////////////////////////////////////////////////////////////////
    // Declaration: Pattern Synonym
    ////////////////////////////////////////////////////////////////////////

    // PatternSyn
    pattern: $ => seq(
        'pattern',
        $.id,
        optional($._lambda_bindings),  // PatternSynArgs
        '=',
        $.expr
    ),

    ////////////////////////////////////////////////////////////////////////
    // Declaration: Unquoting declarations
    ////////////////////////////////////////////////////////////////////////

    // UnquoteDecl
    unquote_decl: $ => choice(
      seq('unquoteDecl',         '=', $.expr),
      seq('unquoteDecl', $._ids, '=', $.expr),
      seq('unquoteDef' , $._ids, '=', $.expr),
    ),

    ////////////////////////////////////////////////////////////////////////
    // Names
    ////////////////////////////////////////////////////////////////////////

    // QId
    qid: $ => prec.left(alias(choice(QID, $.id), 'qid')),

    // BId
    bid: $ => alias(choice('_', $.id), 'bid'),

    // SpaceIds
    _ids: $ => repeat1($.id),

    _field_name: $ => alias($.id, $.field_name),
    _field_names: $ => repeat1($._field_name),

    // MaybeDottedId
    _maybe_dotted_id: $ => maybeDotted($._field_name),
    _maybe_dotted_ids: $ => repeat1($._maybe_dotted_id),

    // ArgIds
    _arg_ids: $ => repeat1($._arg_id),
    _arg_id: $ => choice(
      $._maybe_dotted_id,

      brace($._maybe_dotted_ids),
      brace_double($._maybe_dotted_ids),

      seq('.', brace($._field_names)),
      seq('.', brace_double($._field_names)),

      seq('..', brace($._field_names)),
      seq('..', brace_double($._field_names)),
    ),

    // CommaBIds / CommaBIdAndAbsurds
    _binding_ids_and_absurds: $ => prec(-1, choice(
      $._application,
      seq($.qid, '=', $.qid),
      seq($.qid, '=', '_'  ),
      seq('-'  , '=', $.qid),
      seq('-'  , '=', '_'  ),
    )),

    // Attribute
    attribute: $ => seq('@', $._expr_or_attr),
    attributes: $ => repeat1($.attribute),

    ////////////////////////////////////////////////////////////////////////
    // Expressions (terms and types)
    ////////////////////////////////////////////////////////////////////////

    // Expr
    expr: $ => choice(
      seq($._typed_bindings, $._ARROW, $.expr),
      seq(optional($.attributes), $._atoms, $._ARROW, $.expr),
      seq($._with_exprs, '=', $.expr),
      prec(-1, $._with_exprs), // lowest precedence
    ),
    stmt: $ => choice(
      seq($._typed_bindings, $._ARROW, $.expr),
      seq(optional($.attributes), $._atoms, $._ARROW, $.expr),
      seq($._with_exprs, '=', $.expr),
      prec(-1, $._with_exprs_stmt), // lowest precedence
    ),

    // WithExprs/Expr1
    _with_exprs: $ => seq(
      repeat(seq($._atoms, '|')),
      $._application,
    ),
    _with_exprs_stmt: $ => seq(
      repeat(seq($._atoms, '|')),
      $._application_stmt,
    ),

    // ExprOrAttr
    _expr_or_attr: $ => choice(
      $.literal,
      $.qid,
      paren($.expr),
    ),

    // Application
    _application: $ => seq(
      optional($._atoms),
      $._expr2,
    ),
    _application_stmt: $ => seq(
      optional($._atoms),
      $._expr2_stmt,
    ),

    // Expr
    _expr2_without_let: $ => choice(
      $.lambda,
      alias($.lambda_extended_or_absurd, $.lambda),
      $.forall,
      $.do,
      prec(-1, $.atom),
      seq('quoteGoal', $.id, 'in', $.expr),
      seq('tactic', $._atoms),
      seq('tactic', $._atoms, '|', $._with_exprs),
    ),
    _expr2: $ => choice(
      $._expr2_without_let,
      $.let,
    ),
    _expr2_stmt: $ => choice(
      $._expr2_without_let,
      alias($.let_in_do, $.let),
    ),

    // Expr3
    atom: $ => choice(
      $._atom_curly,
      $._atom_no_curly,
    ),
    // Application3 / OpenArgs
    _atoms: $ => repeat1($.atom),

    _atom_curly: $ => brace(optional($.expr)),

    _atom_no_curly: $ => choice(
      '_',
      'Prop',
      'Set',
      'quote',
      'quoteTerm',
      'quoteContext',
      'unquote',
      $.SetN,
      $.PropN,
      brace_double($.expr),
      idiom($.expr),
      seq('(', ')'),
      seq('{{', '}}'),
      seq('⦃', '⦄'),
      seq($.id, '@', $.atom),
      seq('.', $.atom),
      $.record_assignments,
      alias($.field_assignments, $.record_assignments),
      $._ELLIPSIS,
      $._expr_or_attr
    ),

    // ForallBindings
    forall: $ => seq($._FORALL, $._typed_untyped_bindings, $._ARROW, $.expr),

    // LetBody
    let: $ => prec.right(seq(
      'let',
      // declarations
      optional($._indent),
      repeat(seq($._declaration, $._newline)),
      $._declaration,
      // in case that there's a newline between declarations and $._let_body
      optional($._newline),

      $._let_body
    )),

    // special `let...in` in do statements
    let_in_do: $ => prec.right(seq(
      'let',
      // declarations
      optional($._indent),
      repeat(seq($._declaration, $._newline)),
      $._declaration,
      //
      choice(
        seq($._newline, $._dedent),
        // covers the newline between declarations and $._let_body
        seq($._newline, $._let_body),
        // covers the rest of the cases
        $._let_body,
      )
    )),

    _let_body: $ => seq(
      'in',
      $.expr
    ),

    // LamBindings
    lambda: $ => seq(
      $._LAMBDA,
      $._lambda_bindings,
      $._ARROW,
      $.expr
    ),

    // LamBinds
    _lambda_bindings: $ => seq(
      repeat($._typed_untyped_binding),
      choice(
        $._typed_untyped_binding,
        seq('(', ')'),
        seq('{', '}'),
        seq('{{', '}}'),
        seq('⦃', '⦄'),
      ),
    ),

    integer: $ => integer,

    string: $ => /\".*\"/,

    literal: $ => choice(
      $.integer,
      $.string
    ),
  }
});

function lexeme($, rule) {
  return seq(rule, $._whitespace));
}

function sepR(sep, rule) {
    return seq(rule, repeat(seq(sep, rule)))
}

function sepL(sep, rule) {
    return seq(repeat(seq(rule, sep)), rule)
}

function indent($, ...rule) {
    return seq(
        $._indent,
        ...rule,
        $._dedent
    );
}

// 1 or more $RULE ending with a NEWLINE
function block($, rules) {
    return indent($, repeat1(seq(rules, $._newline)));
}

function flatten(arrOfArrs) {
  return arrOfArrs.reduce((res, arr) => [...res, ...arr], []);
}

function encloseWith(fn, ...pairs) {
  return choice(...flatten(pairs).map(([left, right]) => fn(left, right)));
}

function enclose(expr, ...pairs) {
  return encloseWith((left, right) => seq(left, expr, right), ...pairs);
}

function paren(...rules) {
  return enclose(seq(...rules), PAREN);
}

function brace(...rules) {
  return enclose(seq(...rules), BRACE1);
}
