module.exports = grammar({
  name: "kanso",

  extras: $ => [/[\s\n\t]/, $.comment],

  rules: {
    program: $ => repeat(/\w/),
/*
    program: $ => repeat(choice(
      $._command_or_entry,
      $.comment,
      $.junk
    )),
*/

    comment: $ => token(seq('%', /.*/)),
    junk: $ => /[^%@\s\n\t][^%@]*/,

    _command_or_entry: $ => seq("@", /[a-zA-Z]+/)
  }
});
