package main

import (
	"os"

	"github.com/alecthomas/chroma/v2"
	"github.com/alecthomas/chroma/v2/lexers"
	"github.com/gohugoio/hugo/commands"
)

var mufLexer = chroma.MustNewLexer(
	&chroma.Config{
		Name:      "MUF",
		Aliases:   []string{"muf", "muff"},
		Filenames: []string{"*.muf", "*.muff"},
	},
	func() chroma.Rules {
		return chroma.Rules{
			"Root": {
				{`;;.*$`, chroma.CommentSingle, nil},
				{`!muf\b.*$`, chroma.Keyword, nil},
				{`\.\.(?=\s|$)`, chroma.Punctuation, nil},
				{`\[[^\]]+\]`, chroma.NameTag, nil},
				{`\.(set|make|output|takes|emits|exec|include|define|libdir|lib|needs|ref)\b`, chroma.Keyword, nil},
				{`"([^"\\]|\\.)*"`, chroma.LiteralString, nil},
				{`-?\d+(\.\d+)?`, chroma.LiteralNumber, nil},
				{`@[A-Za-z_][\w-]*`, chroma.NameVariable, nil},
				{`[A-Za-z_][\w.-]*`, chroma.Name, nil},
				{`[-+*/=]+`, chroma.Operator, nil},
				{`\s+`, chroma.Text, nil},
				{`[^\s]+`, chroma.Text, nil},
			},
		}
	},
)

func init() {
	lexers.Register(mufLexer)
}

func main() {
	commands.Execute(os.Args[1:])
}
