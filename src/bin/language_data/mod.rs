use super::{LangSnippet, Language, ShellDialect};

pub(super) mod algol;
pub(super) mod ada;
pub(super) mod apex;
pub(super) mod assembly;
pub(super) mod c;
pub(super) mod c_holyc_shared;
pub(super) mod clojure;
pub(super) mod cobol;
pub(super) mod coffeescript;
pub(super) mod cpp;
pub(super) mod crystal;
pub(super) mod csharp;
pub(super) mod dart;
pub(super) mod elixir;
pub(super) mod erlang;
pub(super) mod fish;
pub(super) mod fortran;
pub(super) mod fsharp;
pub(super) mod go;
pub(super) mod groovy;
pub(super) mod hack;
pub(super) mod haskell;
pub(super) mod holyc;
pub(super) mod java;
pub(super) mod javascript;
pub(super) mod julia;
pub(super) mod kotlin;
pub(super) mod lua;
pub(super) mod makefile_lang;
pub(super) mod matlaboctave;
pub(super) mod move_lang;
pub(super) mod nim;
pub(super) mod ocaml;
pub(super) mod openclc;
pub(super) mod pascal;
pub(super) mod perl;
pub(super) mod php;
pub(super) mod powershell;
pub(super) mod prolog;
pub(super) mod python;
pub(super) mod rlang;
pub(super) mod ruby;
pub(super) mod rust;
pub(super) mod scala;
pub(super) mod scheme;
pub(super) mod shell;
pub(super) mod smalltalk;
pub(super) mod solidity;
pub(super) mod swift;
pub(super) mod tcl;
pub(super) mod typescript;
pub(super) mod verilog;
pub(super) mod vhdl;
pub(super) mod vlang;
pub(super) mod wgsl;
pub(super) mod zig;

pub(super) const fn snippet(
    lang: Language,
    trigger: &'static str,
    label: &'static str,
    body: &'static str,
) -> LangSnippet {
    LangSnippet { lang, trigger, label, body }
}

pub(super) const fn main_func_test_snippets(
    lang: Language,
    main_label: &'static str,
    main_body: &'static str,
    func_label: &'static str,
    func_body: &'static str,
    test_label: &'static str,
    test_body: &'static str,
) -> [LangSnippet; 3] {
    [
        snippet(lang, "main", main_label, main_body),
        snippet(lang, "func", func_label, func_body),
        snippet(lang, "test", test_label, test_body),
    ]
}

pub(super) fn keywords(lang: Language) -> &'static [&'static str] {
    match lang {
        Language::C => c::keywords(),
        Language::Cpp => cpp::keywords(),
        Language::Python => python::keywords(),
        Language::Kotlin => kotlin::keywords(),
        Language::Swift => swift::keywords(),
        Language::Dart => dart::keywords(),
        Language::Elixir => elixir::keywords(),
        Language::Erlang => erlang::keywords(),
        Language::Clojure => clojure::keywords(),
        Language::FSharp => fsharp::keywords(),
        Language::RLang => rlang::keywords(),
        Language::Julia => julia::keywords(),
        Language::MatlabOctave => matlaboctave::keywords(),
        Language::Scala => scala::keywords(),
        Language::Groovy => groovy::keywords(),
        Language::Nim => nim::keywords(),
        Language::Crystal => crystal::keywords(),
        Language::Fortran => fortran::keywords(),
        Language::Cobol => cobol::keywords(),
        Language::Ada => ada::keywords(),
        Language::Assembly => assembly::keywords(),
        Language::VLang => vlang::keywords(),
        Language::Solidity => solidity::keywords(),
        Language::Move => move_lang::keywords(),
        Language::Vhdl => vhdl::keywords(),
        Language::Verilog => verilog::keywords(),
        Language::Prolog => prolog::keywords(),
        Language::Scheme => scheme::keywords(),
        Language::Smalltalk => smalltalk::keywords(),
        Language::Tcl => tcl::keywords(),
        Language::PowerShell => powershell::keywords(),
        Language::Fish => fish::keywords(),
        Language::Makefile => makefile_lang::keywords(),
        Language::Wgsl => wgsl::keywords(),
        Language::OpenClC => openclc::keywords(),
        Language::Hack => hack::keywords(),
        Language::Apex => apex::keywords(),
        Language::JavaScript => javascript::keywords(),
        Language::TypeScript => typescript::keywords(),
        Language::Go => go::keywords(),
        Language::Rust => rust::keywords(),
        Language::Php => php::keywords(),
        Language::Lua => lua::keywords(),
        Language::Shell => shell::keywords(),
        Language::Perl => perl::keywords(),
        Language::Ruby => ruby::keywords(),
        Language::Haskell => haskell::keywords(),
        Language::CoffeeScript => coffeescript::keywords(),
        Language::Pascal => pascal::keywords(),
        Language::Algol => algol::keywords(),
        Language::HolyC => holyc::keywords(),
        Language::Java => java::keywords(),
        Language::Ocaml => ocaml::keywords(),
        Language::Zig => zig::keywords(),
        Language::CSharp => csharp::keywords(),
        Language::Steelconf | Language::Other => &[],
    }
}

pub(super) fn builtins(lang: Language, shell_dialect: ShellDialect) -> &'static [&'static str] {
    match lang {
        Language::C => c::builtins(),
        Language::Python => python::builtins(),
        Language::Kotlin => kotlin::builtins(),
        Language::Swift => swift::builtins(),
        Language::Dart => dart::builtins(),
        Language::Elixir => elixir::builtins(),
        Language::Erlang => erlang::builtins(),
        Language::Clojure => clojure::builtins(),
        Language::FSharp => fsharp::builtins(),
        Language::RLang => rlang::builtins(),
        Language::Julia => julia::builtins(),
        Language::MatlabOctave => matlaboctave::builtins(),
        Language::Scala => scala::builtins(),
        Language::Groovy => groovy::builtins(),
        Language::Nim => nim::builtins(),
        Language::Crystal => crystal::builtins(),
        Language::Fortran => fortran::builtins(),
        Language::Cobol => cobol::builtins(),
        Language::Ada => ada::builtins(),
        Language::Assembly => assembly::builtins(),
        Language::VLang => vlang::builtins(),
        Language::Solidity => solidity::builtins(),
        Language::Move => move_lang::builtins(),
        Language::Vhdl => vhdl::builtins(),
        Language::Verilog => verilog::builtins(),
        Language::Prolog => prolog::builtins(),
        Language::Scheme => scheme::builtins(),
        Language::Smalltalk => smalltalk::builtins(),
        Language::Tcl => tcl::builtins(),
        Language::PowerShell => powershell::builtins(),
        Language::Fish => fish::builtins(),
        Language::Makefile => makefile_lang::builtins(),
        Language::Wgsl => wgsl::builtins(),
        Language::OpenClC => openclc::builtins(),
        Language::Hack => hack::builtins(),
        Language::Apex => apex::builtins(),
        Language::Go => go::builtins(),
        Language::Rust => rust::builtins(),
        Language::Zig => zig::builtins(),
        Language::Java => java::builtins(),
        Language::Haskell => haskell::builtins(),
        Language::Lua => lua::builtins(),
        Language::JavaScript => javascript::builtins(),
        Language::TypeScript => typescript::builtins(),
        Language::CoffeeScript => coffeescript::builtins(),
        Language::Php => php::builtins(),
        Language::HolyC => holyc::builtins(),
        Language::Pascal => pascal::builtins(),
        Language::Algol => algol::builtins(),
        Language::Ruby => ruby::builtins(),
        Language::Perl => perl::builtins(),
        Language::Shell => shell::builtins(shell_dialect),
        Language::Steelconf
        | Language::Cpp
        | Language::Ocaml
        | Language::CSharp
        | Language::Other => &[],
    }
}

pub(super) fn snippets(lang: Language) -> &'static [LangSnippet] {
    match lang {
        Language::Ruby => ruby::snippets(),
        Language::Kotlin => kotlin::snippets(),
        Language::Swift => swift::snippets(),
        Language::Dart => dart::snippets(),
        Language::Solidity => solidity::snippets(),
        Language::PowerShell => powershell::snippets(),
        Language::Makefile => makefile_lang::snippets(),
        Language::Wgsl => wgsl::snippets(),
        Language::OpenClC => openclc::snippets(),
        Language::Hack => hack::snippets(),
        Language::Apex => apex::snippets(),
        Language::Go => go::snippets(),
        Language::Zig => zig::snippets(),
        Language::Java => java::snippets(),
        Language::HolyC => holyc::snippets(),
        Language::Pascal => pascal::snippets(),
        Language::Algol => algol::snippets(),
        Language::Haskell => haskell::snippets(),
        Language::Lua => lua::snippets(),
        Language::JavaScript => javascript::snippets(),
        Language::TypeScript => typescript::snippets(),
        Language::CoffeeScript => coffeescript::snippets(),
        Language::Php => php::snippets(),
        Language::Rust => rust::snippets(),
        _ => &[],
    }
}

pub(super) fn types(lang: Language) -> &'static [&'static str] {
    match lang {
        Language::C => c::types(),
        Language::Cpp => cpp::types(),
        _ => &[],
    }
}

pub(super) fn canonical_trigger(lang: Language, trigger: &str) -> Option<&'static str> {
    canonical_trigger_table()
        .iter()
        .find(|(l, alias, _)| *l == lang && *alias == trigger)
        .map(|(_, _, canonical)| *canonical)
}

pub(super) fn canonical_trigger_table() -> &'static [(Language, &'static str, &'static str)] {
    &[
        (Language::Ruby, "rspec", "test"),
        (Language::Ruby, "spec", "test"),
        (Language::Ruby, "example", "test"),
    ]
}

#[allow(dead_code)]
pub(super) fn new30_languages() -> &'static [Language] {
    &[
        Language::Kotlin,
        Language::Swift,
        Language::Dart,
        Language::Elixir,
        Language::Erlang,
        Language::Clojure,
        Language::FSharp,
        Language::RLang,
        Language::Julia,
        Language::MatlabOctave,
        Language::Scala,
        Language::Groovy,
        Language::Nim,
        Language::Crystal,
        Language::Fortran,
        Language::Cobol,
        Language::Ada,
        Language::Assembly,
        Language::VLang,
        Language::Solidity,
        Language::Move,
        Language::Vhdl,
        Language::Verilog,
        Language::Prolog,
        Language::Scheme,
        Language::Smalltalk,
        Language::Tcl,
        Language::PowerShell,
        Language::Fish,
        Language::Makefile,
    ]
}

#[allow(dead_code)]
pub(super) fn is_advanced_new30(lang: Language) -> bool {
    matches!(
        lang,
        Language::Kotlin
            | Language::Swift
            | Language::Dart
            | Language::Solidity
            | Language::PowerShell
            | Language::Makefile
    )
}
