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

## Unimplemented
* Exponent numbers.
    * For example, `5e10` will not parse.
    * Why? Exponent numbers are currently not supported in the `raystack_core` dependency.
* Hexadecimal numbers.
* Hour format [0-9]?[0-9] in time parsing.
    * For example, `09:30:00` will parse, but `9:30:00` will not.
    * Why? To simplify the LALRPOP grammar code.
* Fractional seconds in time parsing.
    * For example, `09:30:00` will parse, but `09:30:00.123` will not.
    * Why? To simplify the LALRPOP grammar code.
* Raw strings may not work, it depends how `parseAst` serializes them, I need to test this.
    * For example, `r"abc123"` in Axon may not parse.