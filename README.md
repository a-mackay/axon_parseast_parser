# Axon parseAst Parser

Parses the output of SkySpark's `parseAst` function.

## Usage
1. Get the string output of SkySpark's `parseAst` function.
    * For example, run `read(func and name == "yourFunction")->src.parseAst().toAxonCode()`.
1. Use this library's `parse` function on that string.

## Unimplemented
* Exponent numbers.
    * For example, `5e10` will not parse.
* Hexadecimal numbers.
* Hour format [0-9]?[0-9] in time parsing.
    * For example, `09:30:00` will parse, but `9:30:00` will not.
* Fractional seconds in time parsing.
    * For example, `09:30:00` will parse, but `09:30:00.123` will not.

* Raw strings?