" Vim syntax file
" Language: Devlog
" Maintainer: Will Daly
" Latest Revision: 2019-06-05

if exists("b:current_syntax")
  finish
endif

syntax match taskSymbol "^\(+\|-\|*\|\^\)"
syntax match startedTask "^\^..*"
syntax match blockedTask "^\-..*"
syntax match doneTask "^+..*"
syntax match inlineCodeSnippet "\`[^`]*\`"
syntax region blockCodeSnippet start="```" end="```"
syntax match divider "^\-\-\-*"

hi def link taskSymbol Operator
hi def link startedTask Type
hi def link blockedTask Error
hi def link doneTask Identifier
hi def link inlineCodeSnippet PreProc
hi def link blockCodeSnippet PreProc
hi def link divider Special
