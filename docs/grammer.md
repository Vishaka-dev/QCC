
program = { NEWLINE } { circuit { NEWLINE } } EOF ;
circuit = "circuit" IDENT "(" INT [ "," INT ] ")" "{" { NEWLINE } { statement } "}" ;
statemnt = ( gate | measure | barrier ) NEWLINE { NEWLINE } ;
gate =  IDENT [ "(" expr ")" ] qubit { qubit } ;
measure = "measure" qubit "->" clbit ;
barrier = "barrier" { qubit } ;
quibt = IDENT ; (*parser check the form q0, q1, ... *)
clbit = IDENT ; (*parser check the form c0, c1, ...*)

expr = term { ( "+" | "-" ) term } ;
term = unary { ( "*" | "/" ) unary };
unary = "-" unary | primary ;
primary = INT | FLOAT | "pi" | "(" expr ")" ;
