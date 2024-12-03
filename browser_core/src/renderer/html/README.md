# HTMLトークナイザー

ステートマシン図は以下。

```mermaid
stateDiagram-v2
    [*] --> Data

    Data --> TagOpen: <
    Data --> [*]: EOF
    Data --> Data: char

    TagOpen --> EndTagOpen: /
    TagOpen --> TagName: ascii_alphabetic
    TagOpen --> Data: EOF/other

    EndTagOpen --> TagName: ascii_alphabetic
    EndTagOpen --> [*]: EOF

    TagName --> BeforeAttributeName: space
    TagName --> SelfClosingStartTag: /
    TagName --> Data: >
    TagName --> TagName: ascii_char
    TagName --> [*]: EOF

    BeforeAttributeName --> AfterAttributeName: / or > or EOF
    BeforeAttributeName --> AttributeName: other

    AttributeName --> AttributeName: / or > or EOF
    AttributeName --> BeforeAttributeValue: =
    AttributeName --> AttributeName: ascii_char

    AfterAttributeName --> AfterAttributeName: space
    AfterAttributeName --> SelfClosingStartTag: /
    AfterAttributeName --> BeforeAttributeValue: =
    AfterAttributeName --> Data: >
    AfterAttributeName --> AttributeName: other
    AfterAttributeName --> [*]: EOF

    BeforeAttributeValue --> BeforeAttributeValue: space
    BeforeAttributeValue --> AttributeValueDoubleQuoted: "
    BeforeAttributeValue --> AttributeValueSingleQuoted: '
    BeforeAttributeValue --> AttributeValueUnquoted: other

    AttributeValueDoubleQuoted --> AfterAttributeValueQuoted: "
    AttributeValueDoubleQuoted --> AttributeValueDoubleQuoted: char
    AttributeValueDoubleQuoted --> [*]: EOF

    AttributeValueSingleQuoted --> AfterAttributeValueQuoted: '
    AttributeValueSingleQuoted --> AttributeValueSingleQuoted: char
    AttributeValueSingleQuoted --> [*]: EOF

    AttributeValueUnquoted --> BeforeAttributeName: space
    AttributeValueUnquoted --> Data: >
    AttributeValueUnquoted --> AttributeValueUnquoted: char
    AttributeValueUnquoted --> [*]: EOF

    AfterAttributeValueQuoted --> BeforeAttributeName: space
    AfterAttributeValueQuoted --> SelfClosingStartTag: /
    AfterAttributeValueQuoted --> Data: >
    AfterAttributeValueQuoted --> BeforeAttributeValue: other
    AfterAttributeValueQuoted --> [*]: EOF

    SelfClosingStartTag --> Data: >
    SelfClosingStartTag --> [*]: EOF

    Data --> ScriptData: script tag

    ScriptData --> ScriptDataLessThanSign: <
    ScriptData --> ScriptData: char
    ScriptData --> [*]: EOF

    ScriptDataLessThanSign --> ScriptDataEndTagOpen: /
    ScriptDataLessThanSign --> ScriptData: other

    ScriptDataEndTagOpen --> ScriptDataEndTagName: ascii_alphabetic
    ScriptDataEndTagOpen --> ScriptData: other

    ScriptDataEndTagName --> Data: >
    ScriptDataEndTagName --> TemporaryBuffer: other
    ScriptDataEndTagName --> ScriptDataEndTagName: alphabetic

    TemporaryBuffer --> ScriptData: buffer empty
    TemporaryBuffer --> TemporaryBuffer: buffer has chars
```

## HTMLパーサーの流れ

<https://html.spec.whatwg.org/multipage/parsing.html#tree-construction>

### Insertion Mode

一部のみ。

- Initial → BeforeHTML: 文書の開始処理
- BeforeHead → InHead: head要素の処理開始
- InHead → AfterHead: メタデータ処理の完了
- AfterHead → InBody: メインコンテンツの処理開始
- InBody → AfterBody: 本文処理の完了
- AfterAfterBody → [*]: 文書処理の完了

### エッジケース

- 予期しないコンテンツによる状態の巻き戻し（例：AfterAfterBody → InBody）
- 必須要素の自動生成
- クリーンアップ処理

## ステートマシン図

```mermaid
stateDiagram-v2
    [*] --> Initial: Document Start
    Initial --> BeforeHtml: Initial Token

    BeforeHtml --> BeforeHead: html tag
    BeforeHtml --> BeforeHead: other token
    note right of BeforeHtml
        Creates html element
        if not exists
    end note

    BeforeHead --> InHead: head tag
    BeforeHead --> InHead: other token
    note right of BeforeHead
        Creates head element
        if not exists
    end note

    InHead --> AfterHead: head closing tag
    InHead --> AfterHead: non-head element
    note right of InHead
        Processes metadata,
        title, scripts, etc.
    end note

    AfterHead --> InBody: body tag
    AfterHead --> InBody: other token
    note right of AfterHead
        Creates body element
        if not exists
    end note

    InBody --> AfterBody: body closing tag
    note right of InBody
        Main content processing
        - Elements
        - Text nodes
        - Formatting elements
    end note

    AfterBody --> AfterAfterBody: html closing tag
    AfterBody --> InBody: non-html end tag/content
    note left of AfterBody
        Handles trailing content
        after body
    end note

    AfterAfterBody --> [*]: EOF
    AfterAfterBody --> InBody: unexpected content
    note right of AfterAfterBody
        Final cleanup and
        document completion
    end note
```
