#ifndef TYPES_H
#define TYPES_H

typedef unsigned char uint8_t;

typedef enum{ Object, Num, Int, String, Function, Invalid} Type;

typedef enum TokenType{
    Tok_Greater, //Used to signal the initialization of variables as well as comparing values
    Tok_Identifier, //These first few double as opcodes
    Tok_Print,
    Tok_Function,
    Tok_Num,
    Tok_String,
    Tok_Int,

    Tok_Invalid,
    Tok_Begin,
	Tok_Assign,
	Tok_Multiply,
	Tok_Divide,
	Tok_Plus,
	Tok_Minus,
	Tok_PlusEquals,
	Tok_MinusEquals,
    Tok_EqualsEquals,
	Tok_GreaterEquals,
	Tok_Equals,
	Tok_LesserEquals,
	Tok_Lesser,
    Tok_Modulus,
	Tok_BraceOpen,
	Tok_BraceClose,
	Tok_ParenOpen,
	Tok_ParenClose,
	Tok_BracketOpen,
	Tok_BracketClose,
	Tok_Underscore,
	Tok_Comma,
	Tok_Colon,
    Tok_ListInitializer, // The | in the example list of strings: string|>myStringList = "This", "is", "an", "example"
    Tok_Boolean,
    Tok_BooleanOr,
    Tok_BooleanAnd,
    Tok_BooleanTrue,
    Tok_BooleanFalse,
	Tok_IntegerLiteral,
	Tok_DoubleLiteral,
	Tok_StringLiteral,
	Tok_MultiplyEquals,
    Tok_DivideEquals,
	Tok_Return,
	Tok_If,
	Tok_Else,
	Tok_For,
	Tok_While,
	Tok_Continue,
	Tok_Break,
    Tok_Import,
    Tok_Newline,
    Tok_TypeDef,
    Tok_Indent,
    Tok_Unindent,
	Tok_EndOfInput,
    Tok_StrConcat,
    Tok_MalformedString,
    Tok_Exponent
} TokenType;

//The basic Token construct.
typedef struct Token{
    TokenType type;
    char *lexeme;
    unsigned short row;
    unsigned short col;
} Token;

#define ARR_SIZE(a) (sizeof(a) / sizeof(*a))

#define NFREE(x) if(x!=NULL) free(x)

typedef void* Value;

typedef void (*funcPtr)();

extern char *typeDictionary[];

typedef struct{
    int x;
    int y;
} Coords;

typedef struct{
    Value value;
    Type type;
    char dynamic;
    char *name;
} Variable;

//creates a non-user var for intermediate values in expressions
#define VAR(v, t) ((Variable){v, t, 0, NULL})

//function pointer for an operator which takes two
//variables and returns another
typedef Variable (*opFunc)(Variable, Variable);

typedef struct{
    unsigned char isOp;
    Variable v;
    TokenType t;
} ExprValue;

#endif
