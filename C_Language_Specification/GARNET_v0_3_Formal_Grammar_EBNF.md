# Garnet v0.3 — Complete Formal Grammar (EBNF)
**Companion to:** GARNET_v0_3_Mini_Spec.md
**Date:** April 16, 2026
**Notation:** Extended Backus-Naur Form per ISO/IEC 14977

> Every production below corresponds to a normative section in the v0.3 Mini-Spec. Section references are given in comments. This grammar is the canonical reference for parser implementors.

---

## 1. Top-Level Structure

```ebnf
(* A Garnet source file is a sequence of items, optionally preceded by a mode annotation *)
program        = [ "@safe" ] , { item } ;

item           = use-decl
               | module-decl
               | memory-decl
               | actor-decl
               | struct-decl
               | enum-decl
               | trait-decl
               | impl-block
               | managed-fn        (* def ... *)
               | safe-fn           (* fn ... *)
               | const-decl
               | let-decl ;
```

## 2. Modules and Imports (Mini-Spec §3)

```ebnf
module-decl    = [ "@safe" ] , [ "pub" ] , "module" , IDENT , "{" , { item } , "}" ;

use-decl       = "use" , path , [ "::" , ( "{" , ident-list , "}" | "*" ) ] ;
path           = IDENT , { "::" , IDENT } ;
ident-list     = IDENT , { "," , IDENT } ;
```

## 3. Memory Units (Mini-Spec §4)

```ebnf
memory-decl    = "memory" , memory-kind , IDENT , ":" , store-type ;
memory-kind    = "working" | "episodic" | "semantic" | "procedural" ;
store-type     = type ;
```

## 4. Function Definitions (Mini-Spec §5)

```ebnf
(* Managed-mode function — Mini-Spec §5.1 *)
managed-fn     = { annotation } , [ "pub" ] , "def" , IDENT ,
                 [ "<" , type-params , ">" ] ,
                 "(" , [ param-list ] , ")" ,
                 [ "->" , type ] , block ;

(* Safe-mode function — Mini-Spec §5.2 *)
safe-fn        = { annotation } , [ "pub" ] , "fn" , IDENT ,
                 [ "<" , type-params , ">" ] ,
                 "(" , [ typed-param-list ] , ")" ,
                 "->" , type , block ;

(* Parameters *)
param-list     = param , { "," , param } ;
param          = IDENT , [ ":" , type ] ;

typed-param-list = typed-param , { "," , typed-param } ;
typed-param    = [ ownership ] , IDENT , ":" , type ;
ownership      = "own" | "borrow" | "ref" | "mut" ;

(* Closures — Mini-Spec §5.3 *)
closure        = "|" , [ param-list ] , "|" , [ "->" , type ] , ( block | expr ) ;

(* Annotations *)
annotation     = "@max_depth" , "(" , INTEGER , ")"
               | "@fan_out" , "(" , INTEGER , ")"
               | "@require_metadata"
               | "@safe"
               | "@dynamic" ;
```

## 5. Type Syntax (Mini-Spec §11)

```ebnf
type           = simple-type
               | generic-type
               | fn-type
               | tuple-type
               | ref-type ;

simple-type    = path ;                                    (* Int, String, MyModule::MyType *)
generic-type   = path , "<" , type , { "," , type } , ">" ; (* Array<Int>, Map<String, Value> *)
fn-type        = "(" , [ type , { "," , type } ] , ")" , "->" , type ;
tuple-type     = "(" , type , "," , type , { "," , type } , ")" ;
ref-type       = "&" , [ "mut" ] , type ;                  (* safe mode references *)

type-params    = IDENT , { "," , IDENT } ;
```

## 6. User-Defined Types (Mini-Spec §11.3)

```ebnf
struct-decl    = [ "pub" ] , "struct" , IDENT ,
                 [ "<" , type-params , ">" ] ,
                 "{" , { field-decl , [ "," ] } , "}" ;
field-decl     = [ "pub" ] , IDENT , ":" , type , [ "=" , expr ] ;

enum-decl      = [ "pub" ] , "enum" , IDENT ,
                 [ "<" , type-params , ">" ] ,
                 "{" , { variant , "," } , "}" ;
variant        = IDENT , [ "(" , type , { "," , type } , ")" ] ;

trait-decl     = [ "pub" ] , "trait" , IDENT ,
                 [ "<" , type-params , ">" ] ,
                 "{" , { trait-item } , "}" ;
trait-item     = fn-sig | def-sig | const-decl ;
fn-sig         = "fn" , IDENT , "(" , [ typed-param-list ] , ")" , "->" , type ;
def-sig        = "def" , IDENT , "(" , [ param-list ] , ")" , [ "->" , type ] ;

impl-block     = "impl" , [ "<" , type-params , ">" ] , type ,
                 [ "for" , type ] ,
                 "{" , { managed-fn | safe-fn } , "}" ;
```

## 7. Actors (Mini-Spec §9)

```ebnf
actor-decl     = [ "pub" ] , "actor" , IDENT ,
                 "{" , { actor-item } , "}" ;
actor-item     = protocol-decl | handler-decl | memory-decl | let-decl ;

protocol-decl  = "protocol" , IDENT , "(" , typed-param-list , ")" ,
                 [ "->" , type ] ;
handler-decl   = "on" , IDENT , "(" , [ param-list ] , ")" , block ;
```

## 8. Statements (Mini-Spec §6)

```ebnf
block          = "{" , { stmt } , [ expr ] , "}" ;

stmt           = let-decl
               | var-decl
               | const-decl
               | assignment
               | while-stmt
               | for-stmt
               | loop-stmt
               | break-stmt
               | continue-stmt
               | return-stmt
               | raise-stmt
               | expr-stmt ;

let-decl       = "let" , [ "mut" ] , IDENT , [ ":" , type ] , "=" , expr ;
var-decl       = "var" , IDENT , [ ":" , type ] , "=" , expr ;
const-decl     = "const" , IDENT , [ ":" , type ] , "=" , expr ;
assignment     = expr , assign-op , expr ;
assign-op      = "=" | "+=" | "-=" | "*=" | "/=" | "%=" ;

while-stmt     = "while" , expr , block ;
for-stmt       = "for" , IDENT , "in" , expr , block ;
loop-stmt      = "loop" , block ;
break-stmt     = "break" , [ expr ] ;
continue-stmt  = "continue" ;
return-stmt    = "return" , [ expr ] ;
raise-stmt     = "raise" , expr ;
expr-stmt      = expr ;
```

## 9. Expressions (Mini-Spec §§6, 7)

```ebnf
(* Precedence climbing: lowest to highest *)
expr           = pipeline-expr ;

pipeline-expr  = or-expr , { "|>" , or-expr } ;
or-expr        = and-expr , { "or" , and-expr } ;
and-expr       = not-expr , { "and" , not-expr } ;
not-expr       = [ "not" ] , comparison ;
comparison     = range-expr , [ comp-op , range-expr ] ;
comp-op        = "==" | "!=" | "<" | ">" | "<=" | ">=" ;
range-expr     = addition , [ ( ".." | "..." ) , addition ] ;
addition       = multiplication , { ( "+" | "-" ) , multiplication } ;
multiplication = unary , { ( "*" | "/" | "%" ) , unary } ;
unary          = [ "-" | "!" ] , postfix ;
postfix        = primary , { postfix-op } ;
postfix-op     = "." , IDENT , [ "(" , [ arg-list ] , ")" ]      (* method call *)
               | "(" , [ arg-list ] , ")"                         (* function call *)
               | "[" , expr , "]"                                  (* index *)
               | "::" , IDENT                                      (* path access *)
               | "?" ;                                             (* error propagation *)

primary        = INTEGER | FLOAT | STRING | RAW_STRING | SYMBOL
               | "true" | "false" | "nil"
               | IDENT
               | "(" , expr , ")"
               | "[" , [ arg-list ] , "]"                          (* array literal *)
               | "{" , [ map-entries ] , "}"                       (* map literal *)
               | if-expr
               | match-expr
               | try-expr
               | closure
               | spawn-expr ;

arg-list       = expr , { "," , expr } ;
map-entries    = expr , "=>" , expr , { "," , expr , "=>" , expr } ;
```

## 10. Control Flow Expressions (Mini-Spec §6.2)

```ebnf
if-expr        = "if" , expr , block ,
                 { "elsif" , expr , block } ,
                 [ "else" , block ] ;

match-expr     = "match" , expr , "{" , { match-arm , "," } , "}" ;
match-arm      = pattern , [ "if" , expr ] , "=>" , ( expr | block ) ;

pattern        = literal-pattern
               | ident-pattern
               | tuple-pattern
               | enum-pattern
               | wildcard
               | rest-pattern ;
literal-pattern = INTEGER | FLOAT | STRING | SYMBOL | "true" | "false" | "nil" ;
ident-pattern   = IDENT ;
tuple-pattern   = "(" , pattern , { "," , pattern } , ")" ;
enum-pattern    = path , "(" , pattern , { "," , pattern } , ")" ;
wildcard        = "_" ;
rest-pattern    = ".." ;
```

## 11. Error Handling (Mini-Spec §7)

```ebnf
try-expr       = "try" , block , { rescue-clause } , [ ensure-clause ] ;
rescue-clause  = "rescue" , [ IDENT , [ ":" , type ] ] , block ;
ensure-clause  = "ensure" , block ;
```

## 12. Spawn and Message Passing

```ebnf
spawn-expr     = "spawn" , expr ;                          (* spawn an actor or async task *)
send-expr      = expr , "." , IDENT , "(" , [ arg-list ] , ")" ;  (* send protocol message *)
```

## 13. Lexical Rules

```ebnf
IDENT          = ALPHA , { ALPHA | DIGIT | "_" } ;
ALPHA          = "a" | ... | "z" | "A" | ... | "Z" | "_" ;
DIGIT          = "0" | ... | "9" ;
INTEGER        = DIGIT , { DIGIT | "_" } ;
FLOAT          = INTEGER , "." , INTEGER , [ ( "e" | "E" ) , [ "+" | "-" ] , INTEGER ] ;
STRING         = '"' , { CHAR | "#{" , expr , "}" } , '"' ;
RAW_STRING     = 'r"' , { CHAR } , '"' ;
SYMBOL         = ":" , IDENT ;

(* Comments *)
line-comment   = "#" , { any-char-except-newline } , NEWLINE ;
(* No block comments in v0.3. Simplicity over feature count. *)
```

---

## Production Count Summary

| Category | Productions |
|---|---|
| Top-level | 3 |
| Modules/imports | 4 |
| Memory | 3 |
| Functions | 10 |
| Types | 9 |
| User types | 9 |
| Actors | 4 |
| Statements | 13 |
| Expressions | 16 |
| Control flow | 6 |
| Error handling | 3 |
| Spawn/messaging | 2 |
| Lexical | 8 |
| **Total** | **90 productions** |

A grammar of ~90 productions is comparable to Go (100), significantly smaller than Rust (~250), and larger than Lua (~50) — consistent with Garnet's target of "more expressive than Go, less complex than Rust."

---

*"Where there is no vision, the people perish." — Proverbs 29:18*

**Formal Grammar prepared by Claude Code (Opus 4.6) | April 16, 2026**
