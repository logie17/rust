/*
Module: str

String manipulation.
*/

export eq, lteq, hash, is_empty, is_not_empty, is_whitespace, byte_len,
       byte_len_range, index,
       rindex, find, starts_with, ends_with, substr, slice, split, splitn,
       split_str, split_func, split_char, lines, lines_any, words,
       concat, connect, to_lower, to_upper, replace, char_slice,
       trim_left, trim_right, trim, unshift_char, shift_char, pop_char,
       push_char, is_utf8, from_chars, to_chars, char_len, char_len_range,
       char_at, bytes, is_ascii, shift_byte, pop_byte,
       unsafe_from_byte, unsafe_from_bytes, from_char, char_range_at,
       from_cstr, sbuf, as_buf, push_byte, utf8_char_width, safe_slice,
       contains, iter_chars, loop_chars, loop_chars_sub,
       escape;

#[abi = "cdecl"]
native mod rustrt {
    fn rust_str_push(&s: str, ch: u8);
}

/*
Function: eq

Bytewise string equality
*/
pure fn eq(&&a: str, &&b: str) -> bool { a == b }

/*
Function: lteq

Bytewise less than or equal
*/
pure fn lteq(&&a: str, &&b: str) -> bool { a <= b }

/*
Function: hash

String hash function
*/
fn hash(&&s: str) -> uint {
    // djb hash.
    // FIXME: replace with murmur.

    let u: uint = 5381u;
    for c: u8 in s { u *= 33u; u += c as uint; }
    ret u;
}

// UTF-8 tags and ranges
const tag_cont_u8: u8 = 128u8;
const tag_cont: uint = 128u;
const max_one_b: uint = 128u;
const tag_two_b: uint = 192u;
const max_two_b: uint = 2048u;
const tag_three_b: uint = 224u;
const max_three_b: uint = 65536u;
const tag_four_b: uint = 240u;
const max_four_b: uint = 2097152u;
const tag_five_b: uint = 248u;
const max_five_b: uint = 67108864u;
const tag_six_b: uint = 252u;

/*
Function: is_utf8

Determines if a vector uf bytes contains valid UTF-8
*/
fn is_utf8(v: [u8]) -> bool {
    let i = 0u;
    let total = vec::len::<u8>(v);
    while i < total {
        let chsize = utf8_char_width(v[i]);
        if chsize == 0u { ret false; }
        if i + chsize > total { ret false; }
        i += 1u;
        while chsize > 1u {
            if v[i] & 192u8 != tag_cont_u8 { ret false; }
            i += 1u;
            chsize -= 1u;
        }
    }
    ret true;
}

/*
Function: is_ascii

Determines if a string contains only ASCII characters
*/
fn is_ascii(s: str) -> bool {
    let i: uint = byte_len(s);
    while i > 0u { i -= 1u; if s[i] & 128u8 != 0u8 { ret false; } }
    ret true;
}

/*
Predicate: is_empty

Returns true if the string has length 0
*/
pure fn is_empty(s: str) -> bool { for c: u8 in s { ret false; } ret true; }

/*
Predicate: is_not_empty

Returns true if the string has length greater than 0
*/
pure fn is_not_empty(s: str) -> bool { !is_empty(s) }

/*
Function: is_whitespace

Returns true if the string contains only whitespace
*/
fn is_whitespace(s: str) -> bool {
    ret loop_chars(s, char::is_whitespace);
}

/*
Function: byte_len

Returns the length in bytes of a string
*/
pure fn byte_len(s: str) -> uint unsafe {
    let v: [u8] = unsafe::reinterpret_cast(s);
    let vlen = vec::len(v);
    unsafe::leak(v);
    // There should always be a null terminator
    assert (vlen > 0u);
    ret vlen - 1u;
}

/*
Function: byte_len_range

As byte_len but for a substring

Parameters:
s - A string
byte_offset - The byte offset at which to start in the string
char_len    - The number of chars (not bytes!) in the range

Returns:
The number of bytes in the substring starting at `byte_offset` and
containing `char_len` chars.

Safety note:

This function fails if `byte_offset` or `char_len` do not represent
valid positions in `s`
*/
fn byte_len_range(s: str, byte_offset: uint, char_len: uint) -> uint {
    let i = byte_offset;
    let chars = 0u;
    while chars < char_len {
        let chsize = utf8_char_width(s[i]);
        assert (chsize > 0u);
        i += chsize;
        chars += 1u;
    }
    ret i - byte_offset;
}

/*
Function: bytes

Converts a string to a vector of bytes. The result vector is not
null-terminated.
*/
fn bytes(s: str) -> [u8] unsafe {
    let v = unsafe::reinterpret_cast(s);
    let vcopy = vec::slice(v, 0u, vec::len(v) - 1u);
    unsafe::leak(v);
    ret vcopy;
}

/*
Function: unsafe_from_bytes

Converts a vector of bytes to a string. Does not verify that the
vector contains valid UTF-8.
*/
fn unsafe_from_bytes(v: [const u8]) -> str unsafe {
    let vcopy: [u8] = v + [0u8];
    let scopy: str = unsafe::reinterpret_cast(vcopy);
    unsafe::leak(vcopy);
    ret scopy;
}

/*
Function: unsafe_from_byte

Converts a byte to a string. Does not verify that the byte is
valid UTF-8.
*/
fn unsafe_from_byte(u: u8) -> str { unsafe_from_bytes([u]) }

fn push_utf8_bytes(&s: str, ch: char) {
    let code = ch as uint;
    let bytes =
        if code < max_one_b {
            [code as u8]
        } else if code < max_two_b {
            [code >> 6u & 31u | tag_two_b as u8, code & 63u | tag_cont as u8]
        } else if code < max_three_b {
            [code >> 12u & 15u | tag_three_b as u8,
             code >> 6u & 63u | tag_cont as u8, code & 63u | tag_cont as u8]
        } else if code < max_four_b {
            [code >> 18u & 7u | tag_four_b as u8,
             code >> 12u & 63u | tag_cont as u8,
             code >> 6u & 63u | tag_cont as u8, code & 63u | tag_cont as u8]
        } else if code < max_five_b {
            [code >> 24u & 3u | tag_five_b as u8,
             code >> 18u & 63u | tag_cont as u8,
             code >> 12u & 63u | tag_cont as u8,
             code >> 6u & 63u | tag_cont as u8, code & 63u | tag_cont as u8]
        } else {
            [code >> 30u & 1u | tag_six_b as u8,
             code >> 24u & 63u | tag_cont as u8,
             code >> 18u & 63u | tag_cont as u8,
             code >> 12u & 63u | tag_cont as u8,
             code >> 6u & 63u | tag_cont as u8, code & 63u | tag_cont as u8]
        };
    push_bytes(s, bytes);
}

/*
Function: from_char

Convert a char to a string
*/
fn from_char(ch: char) -> str {
    let buf = "";
    push_utf8_bytes(buf, ch);
    ret buf;
}

/*
Function: from_chars

Convert a vector of chars to a string
*/
fn from_chars(chs: [char]) -> str {
    let buf = "";
    for ch: char in chs { push_utf8_bytes(buf, ch); }
    ret buf;
}

/*
Function: utf8_char_width

Given a first byte, determine how many bytes are in this UTF-8 character
*/
pure fn utf8_char_width(b: u8) -> uint {
    let byte: uint = b as uint;
    if byte < 128u { ret 1u; }
    if byte < 192u {
        ret 0u; // Not a valid start byte

    }
    if byte < 224u { ret 2u; }
    if byte < 240u { ret 3u; }
    if byte < 248u { ret 4u; }
    if byte < 252u { ret 5u; }
    ret 6u;
}

/*
Function: char_range_at

Pluck a character out of a string and return the index of the next character.
This function can be used to iterate over the unicode characters of a string.

Example:
> let s = "中华Việt Nam";
> let i = 0u;
> while i < str::byte_len(s) {
>    let {ch, next} = str::char_range_at(s, i);
>    std::io::println(#fmt("%u: %c",i,ch));
>    i = next;
> }

Example output:

      0: 中
      3: 华
      6: V
      7: i
      8: ệ
      11: t
      12:
      13: N
      14: a
      15: m

Parameters:

s - The string
i - The byte offset of the char to extract

Returns:

A record {ch: char, next: uint} containing the char value and the byte
index of the next unicode character.

Failure:

If `i` is greater than or equal to the length of the string.
If `i` is not the index of the beginning of a valid UTF-8 character.
*/
fn char_range_at(s: str, i: uint) -> {ch: char, next: uint} {
    let b0 = s[i];
    let w = utf8_char_width(b0);
    assert (w != 0u);
    if w == 1u { ret {ch: b0 as char, next: i + 1u}; }
    let val = 0u;
    let end = i + w;
    let i = i + 1u;
    while i < end {
        let byte = s[i];
        assert (byte & 192u8 == tag_cont_u8);
        val <<= 6u;
        val += byte & 63u8 as uint;
        i += 1u;
    }
    // Clunky way to get the right bits from the first byte. Uses two shifts,
    // the first to clip off the marker bits at the left of the byte, and then
    // a second (as uint) to get it to the right position.
    val += (b0 << (w + 1u as u8) as uint) << ((w - 1u) * 6u - w - 1u);
    ret {ch: val as char, next: i};
}

/*
Function: char_at

Pluck a character out of a string
*/
fn char_at(s: str, i: uint) -> char { ret char_range_at(s, i).ch; }

/*
Function: iter_chars

Iterate over the characters in a string
*/

fn iter_chars(s: str, it: block(char)) {
    let pos = 0u, len = byte_len(s);
    while (pos < len) {
        let {ch, next} = char_range_at(s, pos);
        pos = next;
        it(ch);
    }
}

/*
Function: loop_chars

Loop through a string, char by char

Parameters:
s  - A string to traverse. It may be empty.
it - A block to execute with each consecutive character of `s`.
Return `true` to continue, `false` to stop.

Returns:

`true` If execution proceeded correctly, `false` if it was interrupted,
that is if `it` returned `false` at any point.
 */
fn loop_chars(s: str, it: block(char) -> bool) -> bool{
    ret loop_chars_sub(s, 0u, byte_len(s), it);
}

/*
Function: loop_chars_sub

Loop through a substring, char by char

Parameters:
s           - A string to traverse. It may be empty.
byte_offset - The byte offset at which to start in the string.
byte_len    - The number of bytes to traverse in the string
it          - A block to execute with each consecutive character of `s`.
Return `true` to continue, `false` to stop.

Returns:

`true` If execution proceeded correctly, `false` if it was interrupted,
that is if `it` returned `false` at any point.

Safety note:
- This function does not check whether the substring is valid.
- This function fails if `byte_offset` or `byte_len` do not
 represent valid positions inside `s`
 */
fn loop_chars_sub(s: str, byte_offset: uint, byte_len: uint,
              it: block(char) -> bool) -> bool {
   let i = byte_offset;
   let result = true;
   while i < byte_len {
      let {ch, next} = char_range_at(s, i);
      if !it(ch) {result = false; break;}
      i = next;
   }
   ret result;
}


/*
Function: char_len

Count the number of unicode characters in a string
*/
fn char_len(s: str) -> uint {
    ret char_len_range(s, 0u, byte_len(s));
}

/*
Function: char_len_range

As char_len but for a slice of a string

Parameters:
 s           - A valid string
 byte_start  - The position inside `s` where to start counting in bytes.
 byte_len    - The number of bytes of `s` to take into account.

Returns:
 The number of Unicode characters in `s` in
segment [byte_start, byte_start+len( .

Safety note:
- This function does not check whether the substring is valid.
- This function fails if `byte_offset` or `byte_len` do not
 represent valid positions inside `s`
*/
fn char_len_range(s: str, byte_start: uint, byte_len: uint) -> uint {
    let i     = byte_start;
    let len   = 0u;
    while i < byte_len {
        let chsize = utf8_char_width(s[i]);
        assert (chsize > 0u);
        len += 1u;
        i += chsize;
    }
    assert (i == byte_len);
    ret len;
}

/*
Function: to_chars

Convert a string to a vector of characters
*/
fn to_chars(s: str) -> [char] {
    let buf: [char] = [];
    let i = 0u;
    let len = byte_len(s);
    while i < len {
        let cur = char_range_at(s, i);
        buf += [cur.ch];
        i = cur.next;
    }
    ret buf;
}

/*
Function: push_char

Append a character to a string
*/
fn push_char(&s: str, ch: char) { s += from_char(ch); }

/*
Function: pop_char

Remove the final character from a string and return it.

Failure:

If the string does not contain any characters.
*/
fn pop_char(&s: str) -> char {
    let end = byte_len(s);
    while end > 0u && s[end - 1u] & 192u8 == tag_cont_u8 { end -= 1u; }
    assert (end > 0u);
    let ch = char_at(s, end - 1u);
    s = substr(s, 0u, end - 1u);
    ret ch;
}

/*
Function: shift_char

Remove the first character from a string and return it.

Failure:

If the string does not contain any characters.
*/
fn shift_char(&s: str) -> char {
    let r = char_range_at(s, 0u);
    s = substr(s, r.next, byte_len(s) - r.next);
    ret r.ch;
}

/*
Function: unshift_char

Prepend a char to a string
*/
fn unshift_char(&s: str, ch: char) { s = from_char(ch) + s; }

/*
Function: index

Returns the index of the first matching byte. Returns -1 if
no match is found.
*/
fn index(s: str, c: u8) -> int {
    let i: int = 0;
    for k: u8 in s { if k == c { ret i; } i += 1; }
    ret -1;
}

/*
Function: rindex

Returns the index of the last matching byte. Returns -1
if no match is found.
*/
fn rindex(s: str, c: u8) -> int {
    let n: int = byte_len(s) as int;
    while n >= 0 { if s[n] == c { ret n; } n -= 1; }
    ret n;
}

/*
Function: find

Finds the index of the first matching substring.
Returns -1 if `haystack` does not contain `needle`.

Parameters:

haystack - The string to look in
needle - The string to look for

Returns:

The index of the first occurance of `needle`, or -1 if not found.
*/
fn find(haystack: str, needle: str) -> int {
    let haystack_len: int = byte_len(haystack) as int;
    let needle_len: int = byte_len(needle) as int;
    if needle_len == 0 { ret 0; }
    fn match_at(haystack: str, needle: str, i: int) -> bool {
        let j: int = i;
        for c: u8 in needle { if haystack[j] != c { ret false; } j += 1; }
        ret true;
    }
    let i: int = 0;
    while i <= haystack_len - needle_len {
        if match_at(haystack, needle, i) { ret i; }
        i += 1;
    }
    ret -1;
}

/*
Function: contains

Returns true if one string contains another

Parameters:

haystack - The string to look in
needle - The string to look for
*/
fn contains(haystack: str, needle: str) -> bool {
    0 <= find(haystack, needle)
}

/*
Function: starts_with

Returns true if one string starts with another

Parameters:

haystack - The string to look in
needle - The string to look for
*/
fn starts_with(haystack: str, needle: str) -> bool {
    let haystack_len: uint = byte_len(haystack);
    let needle_len: uint = byte_len(needle);
    if needle_len == 0u { ret true; }
    if needle_len > haystack_len { ret false; }
    ret eq(substr(haystack, 0u, needle_len), needle);
}

/*
Function: ends_with

Returns true if one string ends with another

haystack - The string to look in
needle - The string to look for
*/
fn ends_with(haystack: str, needle: str) -> bool {
    let haystack_len: uint = byte_len(haystack);
    let needle_len: uint = byte_len(needle);
    ret if needle_len == 0u {
            true
        } else if needle_len > haystack_len {
            false
        } else {
            eq(substr(haystack, haystack_len - needle_len, needle_len),
               needle)
        };
}

/*
Function: substr

Take a substring of another. Returns a string containing `len` bytes
starting at byte offset `begin`.

This function is not unicode-safe.

Failure:

If `begin` + `len` is is greater than the byte length of the string
*/
fn substr(s: str, begin: uint, len: uint) -> str {
    ret slice(s, begin, begin + len);
}

/*
Function: slice

Takes a bytewise slice from a string. Returns the substring from
[`begin`..`end`).

This function is not unicode-safe.

Failure:

- If begin is greater than end.
- If end is greater than the length of the string.
*/
fn slice(s: str, begin: uint, end: uint) -> str unsafe {
    // FIXME: Typestate precondition
    assert (begin <= end);
    assert (end <= byte_len(s));

    let v: [u8] = unsafe::reinterpret_cast(s);
    let v2 = vec::slice(v, begin, end);
    unsafe::leak(v);
    v2 += [0u8];
    let s2: str = unsafe::reinterpret_cast(v2);
    unsafe::leak(v2);
    ret s2;
}

/*
Function: safe_slice
*/
fn safe_slice(s: str, begin: uint, end: uint) : uint::le(begin, end) -> str {
    // would need some magic to make this a precondition
    assert (end <= byte_len(s));
    ret slice(s, begin, end);
}

/*
Function: shift_byte

Removes the first byte from a string and returns it.

This function is not unicode-safe.
*/
fn shift_byte(&s: str) -> u8 {
    let len = byte_len(s);
    assert (len > 0u);
    let b = s[0];
    s = substr(s, 1u, len - 1u);
    ret b;
}

/*
Function: pop_byte

Removes the last byte from a string and returns it.

This function is not unicode-safe.
*/
fn pop_byte(&s: str) -> u8 {
    let len = byte_len(s);
    assert (len > 0u);
    let b = s[len - 1u];
    s = substr(s, 0u, len - 1u);
    ret b;
}

/*
Function: push_byte

Appends a byte to a string.

This function is not unicode-safe.
*/
fn push_byte(&s: str, b: u8) { rustrt::rust_str_push(s, b); }

/*
Function: push_bytes

Appends a vector of bytes to a string.

This function is not unicode-safe.
*/
fn push_bytes(&s: str, bytes: [u8]) {
    for byte in bytes { rustrt::rust_str_push(s, byte); }
}

/*
Function: split

Split a string at each occurance of a given separator

Returns:

A vector containing all the strings between each occurance of the separator

FIXME: should be renamed to split_byte
*/
fn split(s: str, sep: u8) -> [str] {
    let v: [str] = [];
    let accum: str = "";
    let ends_with_sep: bool = false;
    for c: u8 in s {
        if c == sep {
            v += [accum];
            accum = "";
            ends_with_sep = true;
        } else { accum += unsafe_from_byte(c); ends_with_sep = false; }
    }
    if byte_len(accum) != 0u || ends_with_sep { v += [accum]; }
    ret v;
}

/*
Function: splitn

Split a string at each occurance of a given separator up to count times.

Returns:

A vector containing all the strings between each occurance of the separator
*/
fn splitn(s: str, sep: u8, count: uint) -> [str] {
    let v = [];
    let accum = "";
    let n = count;
    let ends_with_sep: bool = false;
    for c in s {
        if n > 0u && c == sep {
            n -= 1u;
            v += [accum];
            accum = "";
            ends_with_sep = true;
        } else { accum += unsafe_from_byte(c); ends_with_sep = false; }
    }
    if byte_len(accum) != 0u || ends_with_sep { v += [accum]; }
    ret v;
}

/*
Function: split_str

Splits a string at each occurrence of the given separator string. Empty
leading fields are suppressed, and empty trailing fields are preserved.

Returns:

A vector containing all the strings between each occurrence of the separator.

FIXME: should behave like split and split_char:
         assert ["", "XXX", "YYY", ""] == split_str(".XXX.YYY.", ".");
*/
fn split_str(s: str, sep: str) -> [str] {
    assert byte_len(sep) > 0u;
    let v: [str] = [], accum = "", sep_match = 0u, leading = true;
    for c: u8 in s {
        // Did we match the entire separator?
        if sep_match == byte_len(sep) {
            if !leading { v += [accum]; }
            accum = "";
            sep_match = 0u;
        }

        if c == sep[sep_match] {
            sep_match += 1u;
        } else {
            sep_match = 0u;
            accum += unsafe_from_byte(c);
            leading = false;
        }
    }

    if byte_len(accum) > 0u { v += [accum]; }
    if sep_match == byte_len(sep) { v += [""]; }

    ret v;
}

/*
Function: split_func

Splits a string into substrings using a function
(unicode safe)

FIXME: will be renamed to split.
*/
fn split_func(ss: str, sepfn: fn&(cc: char)->bool) -> [str] {
    let vv: [str] = [];
    let accum: str = "";
    let ends_with_sep: bool = false;

    str::iter_chars(ss, {|cc| if sepfn(cc) {
            vv += [accum];
            accum = "";
            ends_with_sep = true;
        } else {
            str::push_char(accum, cc);
            ends_with_sep = false;
        }
    });

    if char_len(accum) >= 0u || ends_with_sep {
        vv += [accum];
    }

    ret vv;
}

/*
Function: split_char

Splits a string into a vector of the substrings separated by a given character
*/
fn split_char(ss: str, cc: char) -> [str] {
   split_func(ss, {|kk| kk == cc})
}

/*
Function: lines

Splits a string into a vector of the substrings
separated by LF ('\n')
*/
fn lines(ss: str) -> [str] {
    split_func(ss, {|cc| cc == '\n'})
}

/*
Function: lines_any

Splits a string into a vector of the substrings
separated by LF ('\n') and/or CR LF ('\r\n')
*/
fn lines_any(ss: str) -> [str] {
    vec::map(lines(ss), {|s| trim_right(s)})
}

/*
Function: words

Splits a string into a vector of the substrings
separated by whitespace
*/
fn words(ss: str) -> [str] {
    ret vec::filter( split_func(ss, {|cc| char::is_whitespace(cc)}),
                     {|w| 0u < str::char_len(w)});
}

/*
Function: concat

Concatenate a vector of strings
*/
fn concat(v: [str]) -> str {
    let s: str = "";
    for ss: str in v { s += ss; }
    ret s;
}

/*
Function: connect

Concatenate a vector of strings, placing a given separator between each
*/
fn connect(v: [str], sep: str) -> str {
    let s: str = "";
    let first: bool = true;
    for ss: str in v {
        if first { first = false; } else { s += sep; }
        s += ss;
    }
    ret s;
}

/*
Function: to_lower

Convert a string to lowercase
*/
fn to_lower(s: str) -> str {
    let outstr = "";
    iter_chars(s) { |c|
        push_char(outstr, char::to_lower(c));
    }
    ret outstr;
}
/*
Function: to_upper

Convert a string to uppercase
*/
fn to_upper(s: str) -> str {
    let outstr = "";
    iter_chars(s) { |c|
        push_char(outstr, char::to_upper(c));
    }
    ret outstr;
}

// FIXME: This is super-inefficient
/*
Function: replace

Replace all occurances of one string with another

Parameters:

s - The string containing substrings to replace
from - The string to replace
to - The replacement string

Returns:

The original string with all occurances of `from` replaced with `to`
*/
fn replace(s: str, from: str, to: str) : is_not_empty(from) -> str {
    // FIXME (694): Shouldn't have to check this
    check (is_not_empty(from));
    if byte_len(s) == 0u {
        ret "";
    } else if starts_with(s, from) {
        ret to + replace(slice(s, byte_len(from), byte_len(s)), from, to);
    } else {
        let idx = find(s, from);
        if idx == -1 {
            ret s;
        }
        ret char_slice(s, 0u, idx as uint) + to +
            replace(char_slice(s, idx as uint + char_len(from), char_len(s)),
                    from, to);
    }
}

// FIXME: Also not efficient
/*
Function: char_slice

Unicode-safe slice. Returns a slice of the given string containing
the characters in the range [`begin`..`end`). `begin` and `end` are
character indexes, not byte indexes.

Failure:

- If begin is greater than end
- If end is greater than the character length of the string
*/
fn char_slice(s: str, begin: uint, end: uint) -> str {
    from_chars(vec::slice(to_chars(s), begin, end))
}

/*
Function: trim_left

Returns a string with leading whitespace removed.
*/
fn trim_left(s: str) -> str {
    fn count_whities(s: [char]) -> uint {
        let i = 0u;
        while i < vec::len(s) {
            if !char::is_whitespace(s[i]) { break; }
            i += 1u;
        }
        ret i;
    }
    let chars = to_chars(s);
    let whities = count_whities(chars);
    ret from_chars(vec::slice(chars, whities, vec::len(chars)));
}

/*
Function: trim_right

Returns a string with trailing whitespace removed.
*/
fn trim_right(s: str) -> str {
    fn count_whities(s: [char]) -> uint {
        let i = vec::len(s);
        while 0u < i {
            if !char::is_whitespace(s[i - 1u]) { break; }
            i -= 1u;
        }
        ret i;
    }
    let chars = to_chars(s);
    let whities = count_whities(chars);
    ret from_chars(vec::slice(chars, 0u, whities));
}

/*
Function: trim

Returns a string with leading and trailing whitespace removed
*/
fn trim(s: str) -> str { trim_left(trim_right(s)) }

/*
Type: sbuf

An unsafe buffer of bytes. Corresponds to a C char pointer.
*/
type sbuf = *u8;

// NB: This is intentionally unexported because it's easy to misuse (there's
// no guarantee that the string is rooted). Instead, use as_buf below.
unsafe fn buf(s: str) -> sbuf {
    let saddr = ptr::addr_of(s);
    let vaddr: *[u8] = unsafe::reinterpret_cast(saddr);
    let buf = vec::to_ptr(*vaddr);
    ret buf;
}

/*
Function: as_buf

Work with the byte buffer of a string. Allows for unsafe manipulation
of strings, which is useful for native interop.

Example:

> let s = str::as_buf("PATH", { |path_buf| libc::getenv(path_buf) });

*/
fn as_buf<T>(s: str, f: block(sbuf) -> T) -> T unsafe {
    let buf = buf(s); f(buf)
}

/*
Function: from_cstr

Create a Rust string from a null-terminated C string
*/
unsafe fn from_cstr(cstr: sbuf) -> str {
    let res = "";
    let start = cstr;
    let curr = start;
    let i = 0u;
    while *curr != 0u8 {
        push_byte(res, *curr);
        i += 1u;
        curr = ptr::offset(start, i);
    }
    ret res;
}

/*
Function: escape_char

Escapes a single character.
*/
fn escape_char(c: char) -> str {
    alt c {
      '"' { "\\\"" }
      '\\' { "\\\\" }
      '\n' { "\\n" }
      '\t' { "\\t" }
      '\r' { "\\r" }
      // FIXME: uncomment this when extfmt is moved to core
      // in a snapshot.
      // '\x00' to '\x1f' { #fmt["\\x%02x", c as uint] }
      v { from_char(c) }
    }
}

/*
Function: escape

Escapes special characters inside the string, making it safe for transfer.
*/
fn escape(s: str) -> str {
    let r = "";
    loop_chars(s, { |c| r += escape_char(c); true });
    r
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_eq() {
        assert (eq("", ""));
        assert (eq("foo", "foo"));
        assert (!eq("foo", "bar"));
    }

    #[test]
    fn test_lteq() {
        assert (lteq("", ""));
        assert (lteq("", "foo"));
        assert (lteq("foo", "foo"));
        assert (!eq("foo", "bar"));
    }

    #[test]
    fn test_bytes_len() {
        assert (byte_len("") == 0u);
        assert (byte_len("hello world") == 11u);
        assert (byte_len("\x63") == 1u);
        assert (byte_len("\xa2") == 2u);
        assert (byte_len("\u03c0") == 2u);
        assert (byte_len("\u2620") == 3u);
        assert (byte_len("\U0001d11e") == 4u);
    }

    #[test]
    fn test_index_and_rindex() {
        assert (index("hello", 'e' as u8) == 1);
        assert (index("hello", 'o' as u8) == 4);
        assert (index("hello", 'z' as u8) == -1);
        assert (rindex("hello", 'l' as u8) == 3);
        assert (rindex("hello", 'h' as u8) == 0);
        assert (rindex("hello", 'z' as u8) == -1);
    }

    #[test]
    fn test_split() {
        fn t(s: str, c: char, u: [str]) {
            log(debug, "split: " + s);
            let v = split(s, c as u8);
            #debug("split to: ");
            log(debug, v);
            assert (vec::all2(v, u, { |a,b| a == b }));
        }
        t("abc.hello.there", '.', ["abc", "hello", "there"]);
        t(".hello.there", '.', ["", "hello", "there"]);
        t("...hello.there.", '.', ["", "", "", "hello", "there", ""]);
    }

    #[test]
    fn test_splitn() {
        fn t(s: str, c: char, n: uint, u: [str]) {
            log(debug, "splitn: " + s);
            let v = splitn(s, c as u8, n);
            #debug("split to: ");
            log(debug, v);
            #debug("comparing vs. ");
            log(debug, u);
            assert (vec::all2(v, u, { |a,b| a == b }));
        }
        t("abc.hello.there", '.', 0u, ["abc.hello.there"]);
        t("abc.hello.there", '.', 1u, ["abc", "hello.there"]);
        t("abc.hello.there", '.', 2u, ["abc", "hello", "there"]);
        t("abc.hello.there", '.', 3u, ["abc", "hello", "there"]);
        t(".hello.there", '.', 0u, [".hello.there"]);
        t(".hello.there", '.', 1u, ["", "hello.there"]);
        t("...hello.there.", '.', 3u, ["", "", "", "hello.there."]);
        t("...hello.there.", '.', 5u, ["", "", "", "hello", "there", ""]);
    }

    #[test]
    fn test_split_str() {
        fn t(s: str, sep: str, i: int, k: str) {
            let v = split_str(s, sep);
            assert eq(v[i], k);
        }

        //FIXME: should behave like split and split_char:
        //assert ["", "XXX", "YYY", ""] == split_str(".XXX.YYY.", ".");

        t("abc::hello::there", "::", 0, "abc");
        t("abc::hello::there", "::", 1, "hello");
        t("abc::hello::there", "::", 2, "there");
        t("::hello::there", "::", 0, "hello");
        t("hello::there::", "::", 2, "");
        t("::hello::there::", "::", 2, "");
        t("ประเทศไทย中华Việt Nam", "中华", 0, "ประเทศไทย");
        t("ประเทศไทย中华Việt Nam", "中华", 1, "Việt Nam");
    }

    #[test]
    fn test_split_func () {
        let data = "ประเทศไทย中华Việt Nam";
        assert ["ประเทศไทย中", "Việt Nam"]
            == split_func (data, {|cc| cc == '华'});

        assert ["", "", "XXX", "YYY", ""]
            == split_func("zzXXXzYYYz", char::is_lowercase);

        assert ["zz", "", "", "z", "", "", "z"]
            == split_func("zzXXXzYYYz", char::is_uppercase);

        assert ["",""] == split_func("z", {|cc| cc == 'z'});
        assert [""] == split_func("", {|cc| cc == 'z'});
        assert ["ok"] == split_func("ok", {|cc| cc == 'z'});
    }

    #[test]
    fn test_split_char () {
        let data = "ประเทศไทย中华Việt Nam";
        assert ["ประเทศไทย中", "Việt Nam"]
            == split_char(data, '华');

        assert ["", "", "XXX", "YYY", ""]
            == split_char("zzXXXzYYYz", 'z');
        assert ["",""] == split_char("z", 'z');
        assert [""] == split_char("", 'z');
        assert ["ok"] == split_char("ok", 'z');
    }

    #[test]
    fn test_lines () {
        let lf = "\nMary had a little lamb\nLittle lamb\n";
        let crlf = "\r\nMary had a little lamb\r\nLittle lamb\r\n";

        assert ["", "Mary had a little lamb", "Little lamb", ""]
            == lines(lf);

        assert ["", "Mary had a little lamb", "Little lamb", ""]
            == lines_any(lf);

        assert ["\r", "Mary had a little lamb\r", "Little lamb\r", ""]
            == lines(crlf);

        assert ["", "Mary had a little lamb", "Little lamb", ""]
            == lines_any(crlf);

        assert [""] == lines    ("");
        assert [""] == lines_any("");
        assert ["",""] == lines    ("\n");
        assert ["",""] == lines_any("\n");
        assert ["banana"] == lines    ("banana");
        assert ["banana"] == lines_any("banana");
    }

    #[test]
    fn test_words () {
        let data = "\nMary had a little lamb\nLittle lamb\n";
        assert ["Mary","had","a","little","lamb","Little","lamb"]
            == words(data);

        assert ["ok"] == words("ok");
        assert [] == words("");
    }

    #[test]
    fn test_find() {
        fn t(haystack: str, needle: str, i: int) {
            let j: int = find(haystack, needle);
            log(debug, "searched for " + needle);
            log(debug, j);
            assert (i == j);
        }
        t("this is a simple", "is a", 5);
        t("this is a simple", "is z", -1);
        t("this is a simple", "", 0);
        t("this is a simple", "simple", 10);
        t("this", "simple", -1);
    }

    #[test]
    fn test_substr() {
        fn t(a: str, b: str, start: int) {
            assert (eq(substr(a, start as uint, byte_len(b)), b));
        }
        t("hello", "llo", 2);
        t("hello", "el", 1);
        t("substr should not be a challenge", "not", 14);
    }

    #[test]
    fn test_concat() {
        fn t(v: [str], s: str) { assert (eq(concat(v), s)); }
        t(["you", "know", "I'm", "no", "good"], "youknowI'mnogood");
        let v: [str] = [];
        t(v, "");
        t(["hi"], "hi");
    }

    #[test]
    fn test_connect() {
        fn t(v: [str], sep: str, s: str) {
            assert (eq(connect(v, sep), s));
        }
        t(["you", "know", "I'm", "no", "good"], " ", "you know I'm no good");
        let v: [str] = [];
        t(v, " ", "");
        t(["hi"], " ", "hi");
    }

    #[test]
    fn test_to_upper() {
        // to_upper doesn't understand unicode yet,
        // but we need to at least preserve it

        let unicode = "\u65e5\u672c";
        let input = "abcDEF" + unicode + "xyz:.;";
        let expected = "ABCDEF" + unicode + "XYZ:.;";
        let actual = to_upper(input);
        assert (eq(expected, actual));
    }

    #[test]
    fn test_slice() {
        assert (eq("ab", slice("abc", 0u, 2u)));
        assert (eq("bc", slice("abc", 1u, 3u)));
        assert (eq("", slice("abc", 1u, 1u)));
        fn a_million_letter_a() -> str {
            let i = 0;
            let rs = "";
            while i < 100000 { rs += "aaaaaaaaaa"; i += 1; }
            ret rs;
        }
        fn half_a_million_letter_a() -> str {
            let i = 0;
            let rs = "";
            while i < 100000 { rs += "aaaaa"; i += 1; }
            ret rs;
        }
        assert (eq(half_a_million_letter_a(),
                        slice(a_million_letter_a(), 0u, 500000u)));
    }

    #[test]
    fn test_starts_with() {
        assert (starts_with("", ""));
        assert (starts_with("abc", ""));
        assert (starts_with("abc", "a"));
        assert (!starts_with("a", "abc"));
        assert (!starts_with("", "abc"));
    }

    #[test]
    fn test_ends_with() {
        assert (ends_with("", ""));
        assert (ends_with("abc", ""));
        assert (ends_with("abc", "c"));
        assert (!ends_with("a", "abc"));
        assert (!ends_with("", "abc"));
    }

    #[test]
    fn test_is_empty() {
        assert (is_empty(""));
        assert (!is_empty("a"));
    }

    #[test]
    fn test_is_not_empty() {
        assert (is_not_empty("a"));
        assert (!is_not_empty(""));
    }

    #[test]
    fn test_replace() {
        let a = "a";
        check (is_not_empty(a));
        assert (replace("", a, "b") == "");
        assert (replace("a", a, "b") == "b");
        assert (replace("ab", a, "b") == "bb");
        let test = "test";
        check (is_not_empty(test));
        assert (replace(" test test ", test, "toast") == " toast toast ");
        assert (replace(" test test ", test, "") == "   ");
    }

    #[test]
    fn test_char_slice() {
        assert (eq("ab", char_slice("abc", 0u, 2u)));
        assert (eq("bc", char_slice("abc", 1u, 3u)));
        assert (eq("", char_slice("abc", 1u, 1u)));
        assert (eq("\u65e5", char_slice("\u65e5\u672c", 0u, 1u)));

        let data = "ประเทศไทย中华";
        assert (eq("ป", char_slice(data, 0u, 1u)));
        assert (eq("ร", char_slice(data, 1u, 2u)));
        assert (eq("华", char_slice(data, 10u, 11u)));
        assert (eq("", char_slice(data, 1u, 1u)));

        fn a_million_letter_X() -> str {
            let i = 0;
            let rs = "";
            while i < 100000 { rs += "华华华华华华华华华华"; i += 1; }
            ret rs;
        }
        fn half_a_million_letter_X() -> str {
            let i = 0;
            let rs = "";
            while i < 100000 { rs += "华华华华华"; i += 1; }
            ret rs;
        }
        assert (eq(half_a_million_letter_X(),
                        char_slice(a_million_letter_X(), 0u, 500000u)));
    }

    #[test]
    fn test_trim_left() {
        assert (trim_left("") == "");
        assert (trim_left("a") == "a");
        assert (trim_left("    ") == "");
        assert (trim_left("     blah") == "blah");
        assert (trim_left("   \u3000  wut") == "wut");
        assert (trim_left("hey ") == "hey ");
    }

    #[test]
    fn test_trim_right() {
        assert (trim_right("") == "");
        assert (trim_right("a") == "a");
        assert (trim_right("    ") == "");
        assert (trim_right("blah     ") == "blah");
        assert (trim_right("wut   \u3000  ") == "wut");
        assert (trim_right(" hey") == " hey");
    }

    #[test]
    fn test_trim() {
        assert (trim("") == "");
        assert (trim("a") == "a");
        assert (trim("    ") == "");
        assert (trim("    blah     ") == "blah");
        assert (trim("\nwut   \u3000  ") == "wut");
        assert (trim(" hey dude ") == "hey dude");
    }

    #[test]
    fn test_is_whitespace() {
        assert (is_whitespace(""));
        assert (is_whitespace(" "));
        assert (is_whitespace("\u2009")); // Thin space
        assert (is_whitespace("  \n\t   "));
        assert (!is_whitespace("   _   "));
    }

    #[test]
    fn test_is_ascii() {
        assert (is_ascii(""));
        assert (is_ascii("a"));
        assert (!is_ascii("\u2009"));
    }

    #[test]
    fn test_shift_byte() {
        let s = "ABC";
        let b = shift_byte(s);
        assert (s == "BC");
        assert (b == 65u8);
    }

    #[test]
    fn test_pop_byte() {
        let s = "ABC";
        let b = pop_byte(s);
        assert (s == "AB");
        assert (b == 67u8);
    }

    #[test]
    fn test_unsafe_from_bytes() {
        let a = [65u8, 65u8, 65u8, 65u8, 65u8, 65u8, 65u8];
        let b = unsafe_from_bytes(a);
        assert (b == "AAAAAAA");
    }

    #[test]
    fn test_from_cstr() unsafe {
        let a = [65u8, 65u8, 65u8, 65u8, 65u8, 65u8, 65u8, 0u8];
        let b = vec::to_ptr(a);
        let c = from_cstr(b);
        assert (c == "AAAAAAA");
    }

    #[test]
    fn test_as_buf() unsafe {
        let a = "Abcdefg";
        let b = as_buf(a, {|buf| assert (*buf == 65u8); 100 });
        assert (b == 100);
    }

    #[test]
    fn test_as_buf_small() unsafe {
        let a = "A";
        let b = as_buf(a, {|buf| assert (*buf == 65u8); 100 });
        assert (b == 100);
    }

    #[test]
    fn test_as_buf2() unsafe {
        let s = "hello";
        let sb = as_buf(s, {|b| b });
        let s_cstr = from_cstr(sb);
        assert (eq(s_cstr, s));
    }

    #[test]
    fn vec_str_conversions() {
        let s1: str = "All mimsy were the borogoves";

        let v: [u8] = bytes(s1);
        let s2: str = unsafe_from_bytes(v);
        let i: uint = 0u;
        let n1: uint = byte_len(s1);
        let n2: uint = vec::len::<u8>(v);
        assert (n1 == n2);
        while i < n1 {
            let a: u8 = s1[i];
            let b: u8 = s2[i];
            log(debug, a);
            log(debug, b);
            assert (a == b);
            i += 1u;
        }
    }

    #[test]
    fn test_contains() {
        assert contains("abcde", "bcd");
        assert contains("abcde", "abcd");
        assert contains("abcde", "bcde");
        assert contains("abcde", "");
        assert contains("", "");
        assert !contains("abcde", "def");
        assert !contains("", "a");
    }

    #[test]
    fn test_iter_chars() {
        let i = 0;
        iter_chars("x\u03c0y") {|ch|
            alt i {
              0 { assert ch == 'x'; }
              1 { assert ch == '\u03c0'; }
              2 { assert ch == 'y'; }
            }
            i += 1;
        }
    }

    #[test]
    fn test_escape() {
        assert(escape("abcdef") == "abcdef");
        assert(escape("abc\\def") == "abc\\\\def");
        assert(escape("abc\ndef") == "abc\\ndef");
        assert(escape("abc\"def") == "abc\\\"def");
    }
}