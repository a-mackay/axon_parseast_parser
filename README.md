# Axon parseAst Parser

Parses the output of SkySpark's `parseAst` function.

## Usage
1. Get the string output of SkySpark's `parseAst` function.
    * For example, run `read(func and name == "yourFunction")->src.parseAst().toAxonCode()`.
1. Use this library's `parse` function on that string.

## Why parse the output of parseAst, instead of parsing Axon itself?
1. It's substantially more involved to parse Axon, instead of `parseAst`'s output.
    * We can parse `parseAst` output in under 200 lines of LALRPOP grammar.
1. Axon appears to be an ambiguous language to parse (at least LALRPOP was claiming it was ambiguous).