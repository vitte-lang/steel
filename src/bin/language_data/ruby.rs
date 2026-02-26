use super::super::LangSnippet;

const SNIPPETS: &[LangSnippet] = &[
    LangSnippet { lang: super::super::Language::Ruby, trigger: "func", label: "Ruby method", body: "def ${1:name}(${2:args})\n  ${3:# TODO}\nend" },
    LangSnippet { lang: super::super::Language::Ruby, trigger: "test", label: "RSpec describe", body: "describe \"${1:subject}\" do\n  it \"${2:works}\" do\n    expect(${3:actual}).to eq(${4:expected})\n  end\nend" },
    LangSnippet { lang: super::super::Language::Ruby, trigger: "example", label: "RSpec describe", body: "describe \"${1:subject}\" do\n  it \"${2:works}\" do\n    expect(${3:actual}).to eq(${4:expected})\n  end\nend" },
    LangSnippet { lang: super::super::Language::Ruby, trigger: "spec", label: "RSpec describe", body: "describe \"${1:subject}\" do\n  it \"${2:works}\" do\n    expect(${3:actual}).to eq(${4:expected})\n  end\nend" },
    LangSnippet { lang: super::super::Language::Ruby, trigger: "main", label: "Ruby main guard", body: "if __FILE__ == $0\n  ${1:# TODO}\nend" },
    LangSnippet { lang: super::super::Language::Ruby, trigger: "rspec", label: "RSpec describe", body: "describe \"${1:subject}\" do\n  it \"${2:works}\" do\n    expect(${3:actual}).to eq(${4:expected})\n  end\nend" },
    LangSnippet { lang: super::super::Language::Ruby, trigger: "class", label: "Ruby class", body: "class ${1:Name}\n  def initialize(${2:args})\n  end\nend" },
];

pub(super) fn keywords() -> &'static [&'static str] { super::super::RUBY_KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { super::super::RUBY_BUILTINS }
pub(super) fn snippets() -> &'static [LangSnippet] { SNIPPETS }
