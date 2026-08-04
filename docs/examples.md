# Examples

Every example below is runnable from the CLI and available in the
[browser demo](https://rh-jfuller.github.io/txt2data/).
Grammar and input files live in [`etc/`](../etc/).

## Table of contents

- [CSV](#csv) -- Multi-row comma-separated values
- [Greeting](#greeting) -- Match a literal prefix and extract a name
- [Key-Value](#key-value) -- Simple key=value pair
- [Date](#date) -- ISO 8601 date with fixed-width fields
- [HTTP Request](#http-request) -- HTTP request line (method, path, version)
- [Attribute Marks](#attribute-marks) -- The `@` mark produces flat attributes
- [Email Address](#email-address) -- Split local and domain parts
- [URL](#url) -- Decompose scheme, host, and path
- [Hex Color](#hex-color) -- Parse CSS hex color into RGB components
- [Semver](#semver) -- Semantic version with optional pre-release tag
- [Log Entry](#log-entry) -- Structured log with nested timestamp
- [Env File](#env-file) -- Dotenv-style config (multi-line, exclusion charset)
- [Math Expression](#math-expression) -- Arithmetic with repetition (`*`) for operator chains
- [IPv4 Address](#ipv4-address) -- Dotted-quad address with shared `octet` rule
- [PII Scanner](#pii-scanner) -- Scan free text for SSNs, phone numbers, and emails
- [Markdown](#markdown) -- Parse headings and paragraphs from Markdown
- [Postal Address](#postal-address) -- US mailing address with exclusion charsets
- [Todo List](#todo-list) -- Markdown-style heading + bullet list

---

## CSV

Multi-row comma-separated values.

**Grammar** (`etc/csv.ixml`)

```
csv: row+.
row: name, -",", age, -nl?.
name: char+.
age: digit+.
-char: ["a"-"z"; "A"-"Z"].
-digit: ["0"-"9"].
-nl: #0A.
```

**Input** (`etc/csv.txt`)

```
alice,30
bob,42
```

**JSON**

```json
{
  "row": [
    {
      "name":       {
        "#text": "alice"
      },
      "age":       {
        "#text": "30"
      }
    },
    {
      "name":       {
        "#text": "bob"
      },
      "age":       {
        "#text": "42"
      }
    }
  ]
}
```

**XML**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<csv><row>
  <name>alice</name>
  <age>30</age>

</row>
<row>
  <name>bob</name>
  <age>42</age>
</row>
</csv>
```

---

## Greeting

Match a literal prefix and extract a name.

**Grammar** (`etc/greeting.ixml`)

```
greeting: "Hello, ", name, -"!".
name: letter+.
-letter: ["a"-"z"; "A"-"Z"].
```

**Input** (`etc/greeting.txt`)

```
Hello, World!
```

**JSON**

```json
{
  "name":   {
    "#text": "World"
  }
}
```

**XML**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<greeting>
Hello,   <name>World</name>
</greeting>
```

---

## Key-Value

Simple key=value pair.

**Grammar** (`etc/keyvalue.ixml`)

```
pair: key, -"=", value.
key: letter+.
value: letter+.
-letter: ["a"-"z"].
```

**Input** (`etc/keyvalue.txt`)

```
foo=bar
```

**JSON**

```json
{
  "key":   {
    "#text": "foo"
  },
  "value":   {
    "#text": "bar"
  }
}
```

**XML**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<pair>
  <key>foo</key>
  <value>bar</value>
</pair>
```

---

## Date

ISO 8601 date with fixed-width fields.

**Grammar** (`etc/date.ixml`)

```
date: year, -"-", month, -"-", day.
year: digit, digit, digit, digit.
month: digit, digit.
day: digit, digit.
-digit: ["0"-"9"].
```

**Input** (`etc/date.txt`)

```
2026-08-03
```

**JSON**

```json
{
  "year":   {
    "#text": "2026"
  },
  "month":   {
    "#text": "08"
  },
  "day":   {
    "#text": "03"
  }
}
```

**XML**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<date>
  <year>2026</year>
  <month>08</month>
  <day>03</day>
</date>
```

---

## HTTP Request

HTTP request line (method, path, version).

**Grammar** (`etc/http.ixml`)

```
request: method, -" ", path, -" ", version.
method: upper+.
path: "/", segment.
segment: pchar+.
version: "HTTP/", digit, -".", digit.
-upper: ["A"-"Z"].
-pchar: ["a"-"z"; "/"].
-digit: ["0"-"9"].
```

**Input** (`etc/http.txt`)

```
GET /api/users HTTP/1.1
```

**JSON**

```json
{
  "method":   {
    "#text": "GET"
  },
  "path":   {
    "segment":     {
      "#text": "api/users"
    }
  },
  "version":   {
    "#text": "HTTP/11"
  }
}
```

**XML**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<request>
  <method>GET</method>
  <path>
/    <segment>api/users</segment>
  </path>
  <version>HTTP/11</version>
</request>
```

---

## Attribute Marks

The `@` mark produces flat attributes.

**Grammar** (`etc/attribute.ixml`)

```
decl: @prop, -":", @val.
@prop: letter+.
@val: vchar+.
-letter: ["a"-"z"].
-vchar: ["a"-"z"; "0"-"9"].
```

**Input** (`etc/attribute.txt`)

```
width:100px
```

**JSON**

```json
{
  "@prop": "width",
  "@val": "100px"
}
```

**XML**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<decl prop="width" val="100px"/>
```

---

## Email Address

Split local and domain parts.

**Grammar** (`etc/email.ixml`)

```
email: local, -"@", domain.
local: lchar+.
domain: segment, -".", segment.
segment: dchar+.
-lchar: ["a"-"z"; "0"-"9"; "."].
-dchar: ["a"-"z"; "0"-"9"].
```

**Input** (`etc/email.txt`)

```
user@example.com
```

**JSON**

```json
{
  "local":   {
    "#text": "user"
  },
  "domain":   {
    "segment": [
      {
        "#text": "example"
      },
      {
        "#text": "com"
      }
    ]
  }
}
```

**XML**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<email>
  <local>user</local>
  <domain>
    <segment>example</segment>
    <segment>com</segment>
  </domain>
</email>
```

---

## URL

Decompose scheme, host, and path.

**Grammar** (`etc/url.ixml`)

```
url: scheme, -"://", host, -"/", path.
scheme: schar+.
host: hchar+.
path: pchar+.
-schar: ["a"-"z"].
-hchar: ["a"-"z"; "0"-"9"; "."].
-pchar: ["a"-"z"; "0"-"9"; "/"].
```

**Input** (`etc/url.txt`)

```
https://example.com/api/users
```

**JSON**

```json
{
  "scheme":   {
    "#text": "https"
  },
  "host":   {
    "#text": "example.com"
  },
  "path":   {
    "#text": "api/users"
  }
}
```

**XML**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<url>
  <scheme>https</scheme>
  <host>example.com</host>
  <path>api/users</path>
</url>
```

---

## Hex Color

Parse CSS hex color into RGB components.

**Grammar** (`etc/hexcolor.ixml`)

```
color: -"#", red, green, blue.
red: hex, hex.
green: hex, hex.
blue: hex, hex.
-hex: ["0"-"9"; "a"-"f"].
```

**Input** (`etc/hexcolor.txt`)

```
#ff8800
```

**JSON**

```json
{
  "red":   {
    "#text": "ff"
  },
  "green":   {
    "#text": "88"
  },
  "blue":   {
    "#text": "00"
  }
}
```

**XML**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<color>
  <red>ff</red>
  <green>88</green>
  <blue>00</blue>
</color>
```

---

## Semver

Semantic version with optional pre-release tag.

**Grammar** (`etc/semver.ixml`)

```
semver: major, -".", minor, -".", patch, pre?.
major: digit+.
minor: digit+.
patch: digit+.
pre: -"-", tag.
tag: tchar+.
-digit: ["0"-"9"].
-tchar: ["a"-"z"; "0"-"9"].
```

**Input** (`etc/semver.txt`)

```
1.24.0-beta
```

**JSON**

```json
{
  "major":   {
    "#text": "1"
  },
  "minor":   {
    "#text": "24"
  },
  "patch":   {
    "#text": "0"
  },
  "pre":   {
    "tag":     {
      "#text": "beta"
    }
  }
}
```

**XML**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<semver>
  <major>1</major>
  <minor>24</minor>
  <patch>0</patch>
  <pre>
    <tag>beta</tag>
  </pre>
</semver>
```

---

## Log Entry

Structured log with nested timestamp.

**Grammar** (`etc/logentry.ixml`)

```
entry: timestamp, -" ", level, -" ", message.
timestamp: date, -"T", time, -"Z".
date: year, -"-", month, -"-", day.
time: hour, -":", minute, -":", second.
year: d, d, d, d.
month: d, d.
day: d, d.
hour: d, d.
minute: d, d.
second: d, d.
level: upper+.
message: mchar+.
-d: ["0"-"9"].
-upper: ["A"-"Z"].
-mchar: ~[].
```

**Input** (`etc/logentry.txt`)

```
2026-08-03T14:30:00Z ERROR connection timeout
```

**JSON**

```json
{
  "timestamp":   {
    "date":     {
      "year":       {
        "#text": "2026"
      },
      "month":       {
        "#text": "08"
      },
      "day":       {
        "#text": "03"
      }
    },
    "time":     {
      "hour":       {
        "#text": "14"
      },
      "minute":       {
        "#text": "30"
      },
      "second":       {
        "#text": "00"
      }
    }
  },
  "level":   {
    "#text": "ERROR"
  },
  "message":   {
    "#text": "connection timeout"
  }
}
```

**XML**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<entry>
  <timestamp>
    <date>
      <year>2026</year>
      <month>08</month>
      <day>03</day>
    </date>
    <time>
      <hour>14</hour>
      <minute>30</minute>
      <second>00</second>
    </time>
  </timestamp>
  <level>ERROR</level>
  <message>connection timeout</message>
</entry>
```

---

## Env File

Dotenv-style config (multi-line, exclusion charset).

**Grammar** (`etc/envfile.ixml`)

```
env: entry+.
entry: key, -"=", value, -nl?.
key: kchar+.
value: vchar+.
-kchar: ["A"-"Z"; "_"].
-vchar: ~[#0A].
-nl: #0A.
```

**Input** (`etc/envfile.txt`)

```
DB_HOST=localhost
DB_PORT=5432
DEBUG=true
```

**JSON**

```json
{
  "entry": [
    {
      "key":       {
        "#text": "DB_HOST"
      },
      "value":       {
        "#text": "localhost"
      }
    },
    {
      "key":       {
        "#text": "DB_PORT"
      },
      "value":       {
        "#text": "5432"
      }
    },
    {
      "key":       {
        "#text": "DEBUG"
      },
      "value":       {
        "#text": "true"
      }
    }
  ]
}
```

**XML**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<env><entry>
  <key>DB_HOST</key>
  <value>localhost</value>

</entry>
<entry>
  <key>DB_PORT</key>
  <value>5432</value>

</entry>
<entry>
  <key>DEBUG</key>
  <value>true</value>

</entry>
</env>
```

---

## Math Expression

Arithmetic with repetition (`*`) for operator chains.

**Grammar** (`etc/math.ixml`)

```
expr: term, rest*.
rest: op, term.
term: digit+.
op: ["+"; "-"; "*"; "/"].
-digit: ["0"-"9"].
```

**Input** (`etc/math.txt`)

```
3+42*7
```

**JSON**

```json
{
  "term":   {
    "#text": "3"
  },
  "rest": [
    {
      "op":       {
        "#text": "+"
      },
      "term":       {
        "#text": "42"
      }
    },
    {
      "op":       {
        "#text": "*"
      },
      "term":       {
        "#text": "7"
      }
    }
  ]
}
```

**XML**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<expr>
  <term>3</term>
  <rest>
    <op>+</op>
    <term>42</term>
  </rest>
  <rest>
    <op>*</op>
    <term>7</term>
  </rest>
</expr>
```

---

## IPv4 Address

Dotted-quad address with shared `octet` rule.

**Grammar** (`etc/ipv4.ixml`)

```
ipv4: a, -".", b, -".", c, -".", d.
a: octet.
b: octet.
c: octet.
d: octet.
octet: digit+.
-digit: ["0"-"9"].
```

**Input** (`etc/ipv4.txt`)

```
192.168.1.100
```

**JSON**

```json
{
  "a":   {
    "octet":     {
      "#text": "192"
    }
  },
  "b":   {
    "octet":     {
      "#text": "168"
    }
  },
  "c":   {
    "octet":     {
      "#text": "1"
    }
  },
  "d":   {
    "octet":     {
      "#text": "100"
    }
  }
}
```

**XML**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<ipv4>
  <a>
    <octet>192</octet>
  </a>
  <b>
    <octet>168</octet>
  </b>
  <c>
    <octet>1</octet>
  </c>
  <d>
    <octet>100</octet>
  </d>
</ipv4>
```

---

## PII Scanner

Scan free text for SSNs, phone numbers, and emails.

**Grammar** (`etc/pii.ixml`)

```
scan: -noise+.
-noise: ssn; phone; email; -skip.
ssn: @value.
@value: d, d, d, "-", d, d, "-", d, d, d, d.
phone: @number.
@number: d, d, d, "-", d, d, d, "-", d, d, d, d.
email: @addr.
@addr: local, "@", dom.
-local: ["a"-"z"; "0"-"9"; "."]+.
-dom: ["a"-"z"; "."]+.
-skip: ~[].
-d: ["0"-"9"].
```

**Input** (`etc/pii.txt`)

```
Please update the account for Jane Doe.
Her SSN is 123-45-6789 and she can be
reached at 555-867-5309 or via email
at jane.doe@acme.com for confirmation.
```

**JSON**

```json
{
  "ssn":   {
    "@value": "123-45-6789"
  },
  "phone":   {
    "@number": "555-867-5309"
  },
  "email":   {
    "@addr": "jane.doe@acme.com"
  }
}
```

**XML**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<scan>Please update the account for Jane Doe.
Her SSN is <ssn value="123-45-6789"/>
 and she can be
reached at <phone number="555-867-5309"/>
 or via email
at <email addr="jane.doe@acme.com"/>
 for confirmation.</scan>
```

---

## Markdown

Parse headings and paragraphs from Markdown.

**Grammar** (`etc/markdown.ixml`)

```
doc: block, -more*.
-more: -blank, block.
block: h1; h2; para.
h1: -"# ", @text.
h2: -"## ", @text.
para: @text.
@text: ~[#0A]+.
-blank: #0A, #0A.
```

**Input** (`etc/markdown.txt`)

```
# Title

First paragraph here.

## Section

Second paragraph here.
```

**JSON**

```json
{
  "block": [
    {
      "h1":       {
        "@text": "Title"
      }
    },
    {
      "para":       {
        "@text": "First paragraph here."
      }
    },
    {
      "h2":       {
        "@text": "Section"
      }
    },
    {
      "para":       {
        "@text": "Second paragraph here."
      }
    }
  ]
}
```

**XML**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<doc>
  <block/>


  <block/>


  <block/>


  <block/>
</doc>
```

---

## Postal Address

US mailing address with exclusion charsets.

**Grammar** (`etc/address.ixml`)

```
address: name, -nl, street, -nl,
          city, -", ", state, -" ", zip.
name: ~[#0A]+.
street: ~[#0A]+.
city: ~[","; #0A]+.
state: ["A"-"Z"], ["A"-"Z"].
zip: ["0"-"9"]+.
-nl: #0A.
```

**Input** (`etc/address.txt`)

```
Jane Doe
742 Evergreen Terrace
Springfield, IL 62704
```

**JSON**

```json
{
  "name":   {
    "#text": "Jane Doe"
  },
  "street":   {
    "#text": "742 Evergreen Terrace"
  },
  "city":   {
    "#text": "Springfield"
  },
  "state":   {
    "#text": "IL"
  },
  "zip":   {
    "#text": "62704"
  }
}
```

**XML**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<address>
  <name>Jane Doe</name>

  <street>742 Evergreen Terrace</street>

  <city>Springfield</city>
  <state>IL</state>
  <zip>62704</zip>
</address>
```

---

## Todo List

Markdown-style heading + bullet list.

**Grammar** (`etc/todolist.ixml`)

```
doc: heading, -nl, -nl, list.
heading: -"# ", @title.
@title: ~[#0A]+.
list: item, -more*.
-more: -nl, item.
item: -"- ", @text.
@text: ~[#0A]+.
-nl: #0A.
```

**Input** (`etc/todolist.txt`)

```
# Shopping List

- eggs
- milk
- bread
```

**JSON**

```json
{
  "heading":   {
    "@title": "Shopping List"
  },
  "list":   {
    "item": [
      {
        "@text": "eggs"
      },
      {
        "@text": "milk"
      },
      {
        "@text": "bread"
      }
    ]
  }
}
```

**XML**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<doc>
  <heading title="Shopping List"/>


  <list>
    <item text="eggs"/>

    <item text="milk"/>

    <item text="bread"/>
  </list>
</doc>
```

---

*Generated by `make examples`.*
