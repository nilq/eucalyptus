## eucalyptus

a tree-walk interpreted programming language inspired by fsharp and moonscript

## syntax

data
```
123
+123
-123
```

```
0.123
.123

+0.123
+.123

+.123
-.123
```

```
"normal string"
r"raw string"
```

```
[1, 2, 3, 4]
{a: 10, b: 10}
```

bindings
```
let a = 10
let
  c = "hey1"
  b = "hey2"
```

variables
```
var a = 10
a = 11
```

functions
```
let add1 a b = a + b
let add2 = fun a b -> a + b
```

```
let a () =
  [ 1
  , 2
  , 3
  ]

let b =
  [ 1
  , 2
  , 3
  ]
```
