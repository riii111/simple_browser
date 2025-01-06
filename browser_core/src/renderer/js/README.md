## 概要

JavaScriptエンジンを実装する。

## 実装内容

JavaScriptのコードを解析し、実行する。

- JavaScriptエンジンにはインタプリタ、コンパイラ、JITコンパイラ（Just In Time Compiler）などの実装があるが、ここではインタプリタのみを実装する。
- CSR（Client Side Rendering）、かつ以下の簡易機能のみを実装。
  - 足し算・引き算
  - 変数の定義と使用
  - 関数の定義と呼び出し

## 文法規則

EBNF（Extended Backus-Naur Form）記法を参考に、以下の規則で定義する。

```
Program ::= ( SourceElements )? <EOF>
SourceElements ::= ( SourceElement )+

SourceElement ::= FunctionDeclaration | Statement
FunctionDeclaration ::= "function" Identifier ( "(" ( FormalParameterList )? ")" ) FunctionBody
FormalParameterList ::= Identifier ( "," Identifier )*
FunctionBody ::= "{" ( SourceElements )? "}"

Statement ::= ExpressionStatement | VariableStatement | ReturnStatement

VariableStatement ::= "var" VariableDeclaration ( ";" )?
VariableDeclaration ::= Identifier ( Initializer )?
Initializer ::= "=" AssignmentExpression
ExpressionStatement ::= AssignmentExpression ( ";" )?
AssignmentExpression ::= AdditiveExpression ( "=" AdditiveExpression )*
AdditiveExpression  ::= LeftHandSideExpression ( AdditiveOperator AssignmentExpression )*
AdditiveOperator ::= <"+"> | <"-">

LeftHandSideExpression ::= CallExpression | MemberExpression
CallExpression ::= MemberExpression Arguments
Arguments ::= "(" ( ArgumentList )? ")"
ArgumentList ::= AssignmentExpression ( "," AssignmentExpression )*

MemberExpression  ::= PrimaryExpression ( "." Identifier )*

PrimaryExpression ::= Identifier | Literal
Identifier ::= <identifier name>
<identifier name> ::= (& | _ | a-z | A-Z) (& | a-z | A-Z)*
Literal ::= <digit>+
<digit> ::= 0-9
```

## おまけメモ

### 1. EBNFでの基本的な記号の意味

- `::=` : "次のように定義される"という意味
- `|` : 選択（または）を表す
- `( )` : グループ化
- `?` : 直前の要素が0回か1回出現（任意）
- `+` : 直前の要素が1回以上出現
- `*` : 直前の要素が0回以上出現

#### 例1 変数定義

```
VariableStatement ::= "var" VariableDeclaration ( ";" )?
VariableDeclaration ::= Identifier ( Initializer )?
Initializer ::= "=" AssignmentExpression
```

↓

```js
var x;          // 初期化なしの宣言
var y = 10;     // 初期化付きの宣言
```

※なお、JavaScriptでは`自動セミコロン挿入`が行われるため、セミコロンが省略可。0回も許可されるのでEBNFでは`?`を付ける。

#### 例2 関数定義

```
FunctionDeclaration ::= "function" Identifier ( "(" ( FormalParameterList )? ")" ) FunctionBody
FormalParameterList ::= Identifier ( "," Identifier )*
FunctionBody ::= "{" ( SourceElements )? "}"
```

↓

```js
// パラメータなしの関数
function hello() {
    var x = 10;
}

// 複数パラメータの関数
function add(a, b) {
    return a + b;
}
```
